use crate::{reg_locks::Locks, Clock, PipelineStage, Status};
use libmem::module::Status::Busy;
use libseis::types::Word;
use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, Default)]
enum State {
    Ready {
        instruction: Word,
    },
    Waiting {
        clocks: usize,
    },
    Squashed,
    #[default]
    Idle,
}

use State::*;

#[derive(Debug, Serialize)]
pub struct Fetch {
    state: State,
    forward: bool,
}

impl Default for Fetch {
    fn default() -> Self {
        Self {
            state: Idle,
            forward: false,
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
                    registers.pc += 4;
                }
                Err(Busy(clocks)) => {
                    self.state = Waiting { clocks };
                }
                _ => unreachable!("read_instruction should never return Idle"),
            }
        } else {
            if clock.is_squash() {
                self.state = Squashed;
                self.forward = false;
                return Clock::Squash(clock.clocks());
            }
        }

        self.forward = !clock.is_block();

        match self.state {
            Ready { .. } if !clock.is_block() => clock.to_block(),
            Waiting { .. } => clock.to_block(),
            _ => clock,
        }
    }

    fn forward(&mut self, _: crate::Status<Self::Prev>) -> crate::Status<Self::Next> {
        let state = match self.state {
            Ready { instruction } if self.forward => {
                self.state = Idle;
                Status::Flow(instruction)
            }
            Ready { .. } => Status::Ready,
            Waiting { clocks } => Status::Stall(clocks),
            Squashed => Status::Squashed,
            Idle => Status::Dry,
        };

        state
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Registers;
    use libmem::cache::Associative;
    use libmem::memory::Memory;
    use libmem::module::SingleLevel;

    fn basic_setup<'a>() -> (SingleLevel<'a>, Registers, Locks) {
        (
            SingleLevel::new(
                Associative::new(2, 2),
                Associative::new(2, 2),
                Memory::new(4),
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

        let result = fetch.clock(Clock::Ready(1), &mut reg, &mut lock, &mut mem);

        assert!(matches!(result, Clock::Block(1)));
        assert_eq!(fetch.forward, true);
        assert!(matches!(fetch.state, State::Waiting { clocks: 10 }));
    }

    #[test]
    fn clock_block() {
        let (mut mem, mut reg, mut lock) = basic_setup();
        let mut fetch = Fetch::default();

        let result = fetch.clock(Clock::Block(1), &mut reg, &mut lock, &mut mem);

        assert!(matches!(result, Clock::Block(1)));
        assert_eq!(fetch.forward, false);
        assert!(matches!(fetch.state, State::Waiting { clocks: 10 }));
    }

    #[test]
    fn clock_squash() {
        let (mut mem, mut reg, mut lock) = basic_setup();
        let mut fetch = Fetch::default();

        let result = fetch.clock(Clock::Squash(1), &mut reg, &mut lock, &mut mem);

        assert!(matches!(result, Clock::Squash(1)));
        assert_eq!(fetch.forward, false);
        assert!(matches!(fetch.state, State::Squashed));
    }
}
