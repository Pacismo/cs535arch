use crate::{reg_locks::Locks, Clock, PipelineStage, Status};
use libmem::module::Status::Busy;
use libseis::types::Word;
use serde::Serialize;

/// The state of the [`Fetch`] object
#[derive(Debug, Clone, Copy, Serialize, Default)]
enum State {
    /// The stage is waiting for the next job
    #[default]
    Idle,
    /// The next instruction word is being fetched
    Waiting { clocks: usize },
    /// The next instruction word is available
    Ready { instruction: Word },
    /// The stage has been squashed
    Squashed { clocks: usize },
    /// The stage has been halted
    Halted,
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

    fn is_waiting(&self) -> bool {
        matches!(self, Waiting { .. })
    }

    fn is_squashed(&self) -> bool {
        matches!(self, Squashed { .. })
    }

    fn is_halted(&self) -> bool {
        matches!(self, Halted)
    }

    fn is_idle(&self) -> bool {
        matches!(self, Idle)
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum FetchResult {
    Ready { word: Word },
    Squashed,
}

#[derive(Debug, Serialize)]
pub struct Fetch {
    state: State,
    forward: Option<FetchResult>,
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

    type Next = FetchResult;

    fn clock(
        &mut self,
        clock: Clock,
        registers: &mut crate::Registers,
        _: &mut Locks,
        memory: &mut dyn libmem::module::MemoryModule,
    ) -> Clock {
        if clock.is_halt() {
            self.state = Halted;
            self.forward = None;
            return Clock::Halt;
        } else if clock.is_squash() {
            self.state = Squashed { clocks: 2 };
            self.forward = None;
            return clock;
        } else if let Waiting { ref mut clocks } = self.state {
            *clocks = clocks.saturating_sub(clock.clocks());
            if *clocks == 0 {
                self.state = Idle;
            } else {
                return clock.to_block();
            }
        } else if let Squashed { ref mut clocks } = self.state {
            if clock.is_ready() {
                *clocks = clocks.saturating_sub(clock.clocks());
                if *clocks == 0 {
                    self.state = Idle;
                }
                self.forward = Some(FetchResult::Squashed);
            }
            return clock;
        }

        if self.state.is_idle() {
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
        }

        match self.state {
            Ready { instruction } if !clock.is_block() => {
                self.forward = Some(FetchResult::Ready { word: instruction });
                self.state = Idle;
                clock.to_ready()
            }
            Ready { .. } => clock.to_block(),
            Waiting { .. } => clock.to_block(),
            Halted => Clock::Halt,

            Squashed { .. } => {
                unreachable!("The squashed state can never be the result of a clock")
            }

            Idle => unreachable!("The idle state can never be the result of a clock"),
        }
    }

    fn forward(&mut self, _: crate::Status<Self::Prev>) -> crate::Status<Self::Next> {
        match self.forward {
            Some(instruction) => {
                self.forward = None;
                Status::Flow(instruction)
            }
            None if self.state.is_waiting() => Status::Stall(self.state.clocks()),
            None if self.state.is_squashed() => Status::Squashed,
            None if self.state.is_halted() => Status::Dry,
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

    fn basic_setup() -> (SingleLevel, Registers, Locks) {
        // Create a memoryspace where every byte is the index modulo 256
        let mut memory = Memory::new(4);
        memory.set_page::<PAGE_SIZE>(0x0000_0000, from_fn(|i| (i & 0xFF) as u8));
        memory.set_page::<PAGE_SIZE>(0x0001_0000, from_fn(|i| (i & 0xFF) as u8));
        memory.set_page::<PAGE_SIZE>(0x0002_0000, from_fn(|i| (i & 0xFF) as u8));
        memory.set_page::<PAGE_SIZE>(0x0003_0000, from_fn(|i| (i & 0xFF) as u8));

        (
            SingleLevel::new(
                Box::new(Associative::new(3, 2)),
                Box::new(Associative::new(3, 2)),
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
                assert!(matches!(fetch.forward, Some(FetchResult::Ready { word }) if word == v[0]));
                assert!(matches!(fetch.state, State::Idle));

                state = fetch.forward(Status::default());
                assert!(matches!(
                    state,
                    Status::Flow(FetchResult::Ready { word }) if word == v[0]
                ));

                mem.clock(1);
                result = fetch.clock(Clock::Ready(1), &mut reg, &mut lock, &mut mem);
                assert!(matches!(result, Clock::Ready(1)));
                assert!(matches!(fetch.forward, Some(FetchResult::Ready { word }) if word == v[1]));
                assert!(matches!(fetch.state, State::Idle));

                state = fetch.forward(Status::default());
                assert!(matches!(
                    state,
                    Status::Flow(FetchResult::Ready { word }) if word == v[1]
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
        assert!(matches!(fetch.state, State::Squashed { .. }));
        assert!(matches!(state, Status::Squashed));
    }
}
