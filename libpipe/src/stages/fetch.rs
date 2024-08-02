//! Fetch stage

use crate::{reg_locks::Locks, Clock, PipelineStage, Status};
use libmem::module::Status::Busy;
use libseis::types::Word;
use serde::Serialize;

/// The state of the [`Fetch`] object
#[derive(Debug, Clone, Copy, Default)]
pub enum State {
    /// The stage is waiting for the next job
    #[default]
    Idle,
    /// The next instruction word is being fetched
    Waiting {
        /// Represents the number of clocks before the read is finished
        clocks: usize,
    },
    /// The next instruction word is available
    Ready {
        /// The word representing the next instruction
        word: Word,
    },
    /// The stage has been squashed
    Squashed {
        /// Represents the number of clocks before this stage will begin reading instructions again
        clocks: usize,
    },
    /// The stage has been halted
    Halted,
}
use State::*;

impl Serialize for State {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        match self {
            Idle => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("state", "idle")?;
                map.end()
            }
            Waiting { clocks } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("state", "waiting")?;
                map.serialize_entry("clocks", clocks)?;
                map.end()
            }
            Ready { word: instruction } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("state", "ready")?;
                map.serialize_entry("word", instruction)?;
                map.end()
            }
            Squashed { .. } => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("state", "squashed")?;
                map.end()
            }
            Halted => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("state", "halted")?;
                map.end()
            }
        }
    }
}

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
}

/// Represents the result of a fetch
#[derive(Debug, Clone, Copy)]
pub enum FetchResult {
    /// The next instruction is available
    Ready {
        /// The word to be decoded
        word: Word,
        /// Where this instruction was located
        pc: Word,
    },
    /// No new instructions are available
    Squashed,
}

/// Represents the fetch pipeline stage
#[derive(Debug)]
pub struct Fetch {
    state: State,
    forward: Option<FetchResult>,
}

impl Serialize for Fetch {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.state.serialize(serializer)
    }
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
    type State = State;

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
        }

        match self.state {
            Ready { word } if clock.is_ready() => {
                self.forward = Some(FetchResult::Ready {
                    word,
                    pc: registers.pc,
                });
                self.state = Idle;
                registers.pc = registers.pc.wrapping_add(4);
                clock.to_ready()
            }
            Ready { .. } => clock.to_block(),
            Waiting { .. } => match memory.read_instruction(registers.pc) {
                Ok(word) if clock.is_ready() => {
                    self.forward = Some(FetchResult::Ready {
                        word,
                        pc: registers.pc,
                    });
                    self.state = Idle;
                    registers.pc = registers.pc.wrapping_add(4);
                    clock.to_ready()
                }
                Ok(word) => {
                    self.state = Ready { word };
                    clock.to_block()
                }
                Err(Busy(clocks)) => {
                    self.state = Waiting { clocks };
                    clock.to_block()
                }
                Err(_) => unreachable!(),
            },
            Halted => Clock::Halt,

            Squashed { clocks } if clock.is_ready() => {
                if clocks > 0 {
                    self.state = Squashed { clocks: clocks - 1 };
                    clock.to_squash()
                } else {
                    self.state = Idle;
                    clock.to_ready()
                }
            }

            Squashed { .. } => clock.to_block(),

            Idle => match memory.read_instruction(registers.pc) {
                Ok(word) if clock.is_ready() => {
                    self.forward = Some(FetchResult::Ready {
                        word,
                        pc: registers.pc,
                    });
                    self.state = Idle;
                    registers.pc = registers.pc.wrapping_add(4);
                    clock.to_ready()
                }
                Ok(word) => {
                    self.state = Ready { word };
                    clock.to_block()
                }
                Err(Busy(clocks)) => {
                    self.state = Waiting { clocks };
                    clock.to_block()
                }
                Err(_) => unreachable!(),
            },
        }
    }

    fn forward(&mut self, _: crate::Status<Self::Prev>) -> crate::Status<Self::Next> {
        match self.forward {
            Some(instruction) => {
                self.forward = None;
                Status::Flow(instruction, false)
            }
            None if self.state.is_waiting() => Status::Stall(self.state.clocks()),
            None if self.state.is_squashed() => Status::Squashed(1),
            None if self.state.is_halted() => Status::Dry,
            None => Status::Ready(1, false),
        }
    }

    fn get_state(&self) -> &State {
        &self.state
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
    use libseis::{
        pages::PAGE_SIZE,
        types::{Byte, Word},
    };
    use std::array::from_fn;

    fn basic_setup() -> (SingleLevel, Registers, Locks) {
        // Create a memoryspace where every byte is the index modulo 256
        let mut memory = Memory::new(4);
        memory.set_page(
            0x0000_0000,
            &from_fn::<Byte, PAGE_SIZE, _>(|i| (i & 0xFF) as u8),
        );
        memory.set_page(
            0x0001_0000,
            &from_fn::<Byte, PAGE_SIZE, _>(|i| (i & 0xFF) as u8),
        );
        memory.set_page(
            0x0002_0000,
            &from_fn::<Byte, PAGE_SIZE, _>(|i| (i & 0xFF) as u8),
        );
        memory.set_page(
            0x0003_0000,
            &from_fn::<Byte, PAGE_SIZE, _>(|i| (i & 0xFF) as u8),
        );

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
                assert!(
                    matches!(fetch.forward, Some(FetchResult::Ready { word ,.. }) if word == v[0])
                );
                assert!(matches!(fetch.state, State::Idle));

                state = fetch.forward(Status::default());
                assert!(matches!(
                    state,
                    Status::Flow(FetchResult::Ready { word, .. }, _) if word == v[0]
                ));

                mem.clock(1);
                result = fetch.clock(Clock::Ready(1), &mut reg, &mut lock, &mut mem);
                assert!(matches!(result, Clock::Ready(1)));
                assert!(
                    matches!(fetch.forward, Some(FetchResult::Ready { word ,.. }) if word == v[1])
                );
                assert!(matches!(fetch.state, State::Idle));

                state = fetch.forward(Status::default());
                assert!(matches!(
                    state,
                    Status::Flow(FetchResult::Ready { word ,.. }, _) if word == v[1]
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
        assert!(matches!(state, Status::Squashed(_)));
    }
}
