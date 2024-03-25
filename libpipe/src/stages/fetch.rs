use crate::{reg_locks::Locks, Clock, PipelineStage, Status};
use libmem::module::Status::Busy;
use libseis::types::Word;
use serde::Serialize;

/// The state of the [`Fetch`] object
#[derive(Debug, Clone, Copy, Serialize, Default)]
enum State {
    /// The next instruction word is available
    Ready { instruction: Word },
    /// The next instruction word is being fetched
    Waiting { clocks: usize },
    /// The stage has been squashed
    Squashed,
    /// The stage is waiting for the next job
    #[default]
    Idle,
}

use State::*;

impl State {
    /// The number of clocks required until the state can move on
    fn clocks(&self) -> usize {
        if let &Waiting { clocks } = self {
            clocks
        } else {
            1
        }
    }

    fn waiting(&self) -> bool {
        matches!(self, Waiting { .. })
    }

    fn squashed(&self) -> bool {
        matches!(self, Squashed)
    }
}

#[derive(Debug, Serialize)]
pub struct Fetch {
    state: State,
    forward: Option<Word>,
}

impl Default for Fetch {
    fn default() -> Self {
        Self {
            state: Idle,
            forward: None,
        }
    }
}

impl PipelineStage for Fetch {
    type Prev = ();

    type Next = Word;

    fn clock(
        &mut self,
        clock: Clock,
        registers: &mut crate::Registers,
        _: &mut Locks,
        memory: &mut dyn libmem::module::MemoryModule,
    ) -> Clock {
        if let Waiting { ref mut clocks } = self.state {
            *clocks = clocks.saturating_sub(clock.clocks());
            if *clocks == 0 {
                self.state = Idle;
            }
        }

        if matches!(self.state, Idle | Squashed) {
            match memory.read_instruction(registers.pc) {
                Ok(instruction) => {
                    self.state = Ready { instruction };
                    registers.pc = registers.pc.wrapping_add(4);
                }
                Err(Busy(clocks)) => {
                    self.state = Waiting { clocks };
                }
                _ => unreachable!("read_instruction should never return Idle"),
            }
        } else {
            if clock.is_squash() {
                self.state = Squashed;
                self.forward = None;
                return Clock::Squash(clock.clocks());
            }
        }

        match self.state {
            Ready { instruction } if !clock.is_block() => {
                self.forward = Some(instruction);
                self.state = Idle;
                clock.to_ready()
            }
            Ready { .. } => clock.to_block(),
            Waiting { .. } => clock.to_block(),
            Squashed => clock.to_ready(),
            Idle => unreachable!("The idle state can never be the result of a clock"),
        }
    }

    fn forward(&mut self, _: crate::Status<Self::Prev>) -> crate::Status<Self::Next> {
        match self.forward {
            Some(instruction) => {
                self.forward = None;
                Status::Flow(instruction)
            }
            None if self.state.waiting() => Status::Stall(self.state.clocks()),
            None if self.state.squashed() => Status::Squashed,
            None => Status::Ready,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Registers;
    use libmem::{
        cache::Associative,
        memory::Memory,
        module::{MemoryModule, SingleLevel},
    };
    use libseis::{pages::PAGE_SIZE, types::Word};
    use std::array::from_fn;

    fn basic_setup() -> (SingleLevel<Associative>, Registers, Locks) {
        // Create a memoryspace where every byte is the index modulo 256
        let mut memory = Memory::new(4);
        memory.set_page::<PAGE_SIZE>(0x0000_0000, from_fn(|i| (i & 0xFF) as u8));
        memory.set_page::<PAGE_SIZE>(0x0001_0000, from_fn(|i| (i & 0xFF) as u8));
        memory.set_page::<PAGE_SIZE>(0x0002_0000, from_fn(|i| (i & 0xFF) as u8));
        memory.set_page::<PAGE_SIZE>(0x0003_0000, from_fn(|i| (i & 0xFF) as u8));

        (
            SingleLevel::new(
                Associative::new(3, 2),
                Associative::new(3, 2),
                memory,
                10,
                2,
                false,
            ),
            Registers::default(),
            Locks::default(),
        )
    }

    #[test]
    fn clock_ready() {
        let (mut mem, mut reg, mut lock) = basic_setup();
        let mut fetch = Fetch::default();

        (0..)
            .step_by(8)
            .map(|i| {
                (
                    i / 8,
                    [
                        Word::from_be_bytes([i, i + 1, i + 2, i + 3]),
                        Word::from_be_bytes([i + 4, i + 5, i + 6, i + 7]),
                    ],
                )
            })
            .take(16)
            .for_each(|(i, v)| {
                let mut result = fetch.clock(Clock::Ready(1), &mut reg, &mut lock, &mut mem);
                let mut state = fetch.forward(Status::default());

                assert!(matches!(result, Clock::Block(1)));
                assert!(fetch.forward.is_none(), "Forward anew for {i}");
                assert!(matches!(fetch.state, State::Waiting { clocks: 10 }));
                assert!(matches!(state, Status::Stall(10)));

                mem.clock(10);
                result = fetch.clock(Clock::Ready(10), &mut reg, &mut lock, &mut mem);
                assert!(matches!(result, Clock::Ready(10)));
                assert!(matches!(fetch.forward, Some(x) if x == v[0]));
                assert!(matches!(fetch.state, State::Idle));

                state = fetch.forward(Status::default());
                assert!(matches!(
                    state,
                    Status::Flow(x) if x == v[0]
                ));

                mem.clock(1);
                result = fetch.clock(Clock::Ready(1), &mut reg, &mut lock, &mut mem);
                assert!(matches!(result, Clock::Ready(1)));
                assert!(matches!(fetch.forward, Some(x) if x == v[1]));
                assert!(matches!(fetch.state, State::Idle));

                state = fetch.forward(Status::default());
                assert!(matches!(
                    state,
                    Status::Flow(x) if x == v[1]
                ))
            })
    }

    #[test]
    fn clock_block() {
        let (mut mem, mut reg, mut lock) = basic_setup();
        let mut fetch = Fetch::default();

        let result = fetch.clock(Clock::Block(1), &mut reg, &mut lock, &mut mem);
        let state = fetch.forward(Status::default());

        assert!(matches!(result, Clock::Block(1)));
        assert!(fetch.forward.is_none());
        assert!(matches!(fetch.state, State::Waiting { clocks: 10 }));
        assert!(matches!(state, Status::Stall(10)));
    }

    #[test]
    fn clock_squash() {
        let (mut mem, mut reg, mut lock) = basic_setup();
        let mut fetch = Fetch::default();

        let result = fetch.clock(Clock::Squash(1), &mut reg, &mut lock, &mut mem);
        let state = fetch.forward(Status::default());

        assert!(matches!(result, Clock::Squash(1)));
        assert!(fetch.forward.is_none());
        assert!(matches!(fetch.state, State::Squashed));
        assert!(matches!(state, Status::Squashed));
    }
}
