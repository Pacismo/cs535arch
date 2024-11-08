//! Decode stage

use crate::{reg_locks::Locks, regmap::RegMap, Clock, PipelineStage, Registers, Status};
use libmem::module::MemoryModule;
use libseis::{
    instruction_set::{decode, Info, Instruction},
    registers::{RegisterFlags, BP, PC, SP},
    types::Word,
};
use serde::Serialize;

/// The state of the [`Decode`] object
#[derive(Debug, Clone, Copy, Default)]
pub enum State {
    /// This stage is decoding an instruction
    Decoding {
        /// The word value being decoded
        word: Word,
        /// The location from which the value was fetched
        pc: Word,
    },
    /// This stage is ready to forward a decoded instruction
    Ready {
        /// The word value that was decoded
        word: Word,
        /// The location from which the value was fetched
        pc: Word,
    },
    /// This stage is awaiting the next instruction
    #[default]
    Idle,
    /// This stage is squashed
    Squashed,
    /// The last stage was a squash
    PrevSquash,
    /// This stage has ceased execution
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
            Decoding { word, .. } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("state", "decoding")?;
                map.serialize_entry("word", word)?;
                map.end()
            }
            Ready { word, .. } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("state", "ready")?;
                map.serialize_entry(
                    "instruction",
                    &decode::<Instruction>(*word).unwrap_or_default().to_string(),
                )?;
                map.end()
            }
            Idle => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("state", "idle")?;
                map.end()
            }
            Squashed => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("state", "squashed")?;
                map.end()
            }
            PrevSquash => {
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

use super::fetch::FetchResult;

impl State {
    fn is_ready(self) -> bool {
        matches!(self, Ready { .. })
    }

    fn is_idle(self) -> bool {
        matches!(self, Idle)
    }

    fn is_squashed(self) -> bool {
        matches!(self, Squashed | PrevSquash)
    }

    fn is_halted(&self) -> bool {
        matches!(self, Halted)
    }
}

/// The result of a decode operation
#[derive(Debug, Clone)]
pub enum DecodeResult {
    /// This stage is forwarding an instruction
    Forward {
        /// The instruction that was decoded
        instruction: Instruction,
        /// The values of the registers
        regvals: RegMap,
        /// The registers that got locked
        reglocks: RegisterFlags,
    },
    /// This stage is forwarding a squashed instruction
    Squashed,
}

/// The decode stage
#[derive(Debug, Default)]
pub struct Decode {
    state: State,
    forward: Option<DecodeResult>,
}

impl Serialize for Decode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.state.serialize(serializer)
    }
}

impl Decode {
    /// Gets the state of this stage
    pub fn get_state(&self) -> &State {
        &self.state
    }
}

impl PipelineStage for Decode {
    type Prev = FetchResult;
    type Next = DecodeResult;
    type State = State;

    fn clock(
        &mut self,
        clock: Clock,
        registers: &mut Registers,
        reg_locks: &mut Locks,
        _: &mut dyn MemoryModule,
    ) -> crate::Clock {
        if clock.is_halt() {
            self.state = Halted;
            self.forward = None;
            clock
        } else if clock.is_squash() {
            self.forward = None;
            self.state = Squashed;
            clock
        } else if self.state.is_halted() {
            Clock::Halt
        } else {
            if let Decoding { word, pc } = self.state {
                self.state = Ready { word, pc }
            }

            match self.state {
                Ready { word, pc } => {
                    if clock.is_ready() {
                        let instruction: Instruction = decode(word).unwrap_or_default();

                        let write = instruction.get_write_regs();
                        let reads = instruction.get_read_regs();

                        if reads.registers().all(|reg| reg_locks.is_unlocked(reg)) {
                            for reg in write {
                                reg_locks[reg] += 1;
                            }

                            self.forward = Some(DecodeResult::Forward {
                                instruction,
                                regvals: reads
                                    .registers()
                                    .map(|r| {
                                        if r == PC {
                                            // PC must equal location of where instruction was fetched -- always one word behind
                                            (PC, pc)
                                        } else if r == SP || r == BP {
                                            (r, (registers[r] & 0x0000_FFFF) | 0x0001_0000)
                                        } else {
                                            (r, registers[r])
                                        }
                                    })
                                    .collect(),
                                reglocks: write,
                            });

                            self.state = Idle;

                            clock.to_ready()
                        } else {
                            clock.to_block()
                        }
                    } else {
                        if self.state.is_idle() {
                            clock.to_ready()
                        } else {
                            clock
                        }
                    }
                }
                Idle => clock.to_ready(),
                Squashed | PrevSquash => {
                    if clock.is_ready() {
                        self.forward = Some(DecodeResult::Squashed);
                        self.state = Idle;
                        clock.to_ready()
                    } else {
                        clock
                    }
                }
                _ => unreachable!(),
            }
        }
    }

    fn forward(&mut self, input: Status<Self::Prev>) -> Status<Self::Next> {
        use std::mem::take;

        if self.state.is_halted() {
            Status::Dry
        } else {
            let (clocks, bubbles) = match input {
                Status::Flow(FetchResult::Ready { word, pc }, b) => {
                    self.state = Decoding { word, pc };
                    (1, b)
                }
                Status::Flow(FetchResult::Squashed, b) => {
                    self.state = PrevSquash;
                    (1, b)
                }
                Status::Stall(clocks) => (clocks, true),
                Status::Ready(n, b) => (n, b),
                Status::Squashed(n) => (n, true),
                Status::Dry => (1, true),
            };

            match take(&mut self.forward) {
                Some(v) => Status::Flow(v, bubbles),
                None if self.state.is_ready() => Status::Ready(clocks, bubbles),
                None if self.state.is_squashed() => Status::Squashed(clocks),
                None if self.state.is_idle() => Status::Stall(clocks),
                None => Status::Stall(1),
            }
        }
    }

    fn get_state(&self) -> &State {
        &self.state
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use libmem::{cache::Associative, memory::Memory, module::SingleLevel};
    use libseis::{instruction_set::ControlOp, pages::PAGE_SIZE, types::Byte};
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
    fn clock_basic() {
        let (mut mem, mut reg, mut lock) = basic_setup();
        let mut decode = Decode::default();

        // Clock it once

        assert!(matches!(
            decode.clock(Clock::Ready(1), &mut reg, &mut lock, &mut mem),
            Clock::Ready(1)
        ));

        // No state change

        assert!(matches!(
            decode,
            Decode {
                forward: None,
                state: State::Idle
            }
        ));

        // Forward a NOP word

        assert!(matches!(
            decode.forward(Status::Flow(
                FetchResult::Ready {
                    word: 0x0000_0000,
                    pc: 0
                },
                false
            )),
            Status::Stall(1)
        ));

        // Check that it changed state

        assert!(matches!(
            decode,
            Decode {
                forward: None,
                state: State::Decoding {
                    word: 0x0000_0000,
                    ..
                }
            }
        ));

        // Clock it once

        assert!(matches!(
            decode.clock(Clock::Ready(1), &mut reg, &mut lock, &mut mem),
            Clock::Ready(1)
        ));

        // Check that its state changed to idle and that it finished decoding

        assert!(matches!(
            decode,
            Decode {
                forward: Some(_),
                state: State::Idle,
            }
        ));

        // Forward the value and make sure it is what we expect

        assert!(matches!(
            decode.forward(Status::Flow(FetchResult::Ready { word: 0x0000_0000, pc: 0 }, false)),
            Status::Flow(DecodeResult::Forward {
                instruction: Instruction::Control(ControlOp::Nop), // Nop
                regvals, // No register values
                reglocks: RegisterFlags(0) // No register locks
            }, _) if regvals.len() == 0
        ));
    }

    #[test]
    fn clock_block() {
        let (mut mem, mut reg, mut lock) = basic_setup();
        let mut decode = Decode::default();

        // without data, the decode step should simply report that it is ready to receive data

        assert!(matches!(
            decode.clock(Clock::Block(1), &mut reg, &mut lock, &mut mem),
            Clock::Ready(1)
        ));
    }

    #[test]
    fn clock_squash() {
        let (mut mem, mut reg, mut lock) = basic_setup();
        let mut decode = Decode::default();

        // We need to forward the Squash clock

        assert!(matches!(
            decode.clock(Clock::Squash(1), &mut reg, &mut lock, &mut mem),
            Clock::Squash(1)
        ));
    }
}
