use crate::{reg_locks::Locks, regmap::RegMap, Clock, PipelineStage, Registers, Status};
use libmem::module::MemoryModule;
use libseis::{
    instruction_set::{decode, Info, Instruction},
    registers::{RegisterFlags, PC},
    types::Word,
};
use serde::Serialize;

/// The state of the [`Decode`] object
#[derive(Debug, Clone, Copy, Serialize, Default)]
enum State {
    Decoding {
        word: Word,
    },
    Ready {
        word: Word,
    },
    #[default]
    Idle,
    Squashed,
}
use State::*;

impl State {
    fn is_ready(self) -> bool {
        matches!(self, Ready { .. })
    }

    fn is_idle(self) -> bool {
        matches!(self, Idle)
    }

    fn is_squashed(self) -> bool {
        matches!(self, Squashed)
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum DecodeResult {
    Forward {
        instruction: Instruction,
        regvals: RegMap,
        reglocks: RegisterFlags,
    },
    Squashed,
}

#[derive(Debug, Serialize, Default)]
pub struct Decode {
    forward: Option<DecodeResult>,
    state: State,
}

impl PipelineStage for Decode {
    type Prev = Word;
    type Next = DecodeResult;

    fn clock(
        &mut self,
        clock: Clock,
        registers: &mut Registers,
        reg_locks: &mut Locks,
        _: &mut dyn MemoryModule,
    ) -> crate::Clock {
        if clock.is_squash() {
            self.forward = None;
            self.state = Squashed;
            clock
        } else {
            if let Decoding { word } = self.state {
                self.state = Ready { word }
            }

            match self.state {
                Ready { word } => {
                    if clock.is_ready() {
                        let instruction: Instruction = decode(word).unwrap_or_default();

                        let write = instruction.get_write_regs();
                        let reads = instruction.get_read_regs();

                        if reads.iter().all(|&reg| reg_locks.is_unlocked(reg)) {
                            for reg in write {
                                reg_locks[reg] += 1;
                            }

                            self.forward = Some(DecodeResult::Forward {
                                instruction,
                                regvals: reads
                                    .into_iter()
                                    .map(|r| {
                                        // PC must equal location of where instruction was fetched -- always one word behind
                                        if r == PC {
                                            (PC, registers[r] - 4)
                                        } else {
                                            (r, registers[r])
                                        }
                                    })
                                    .collect(),
                                reglocks: write,
                            });

                            self.state = Idle;

                            clock
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
                Squashed => {
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

        let clocks = match input {
            Status::Flow(word) => {
                self.state = Decoding { word };
                1
            }
            Status::Stall(clocks) => clocks,
            _ => 1,
        };

        match take(&mut self.forward) {
            Some(v) => Status::Flow(v),
            None if self.state.is_ready() => Status::Ready,
            None if input.is_dry() => Status::Dry,
            None => Status::Stall(clocks),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use libmem::{cache::Associative, memory::Memory, module::SingleLevel};
    use libseis::{instruction_set::ControlOp, pages::PAGE_SIZE};
    use std::array::from_fn;

    fn basic_setup<'a>() -> (SingleLevel<'a>, Registers, Locks) {
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
            decode.forward(Status::Flow(0x0000_0000)),
            Status::Stall(1)
        ));

        // Check that it changed state

        assert!(matches!(
            decode,
            Decode {
                forward: None,
                state: State::Decoding { word: 0x0000_0000 }
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
            decode.forward(Status::Flow(0x0000_0000)),
            Status::Flow(DecodeResult::Forward {
                instruction: Instruction::Control(ControlOp::Nop), // Nop
                regvals, // No register values
                reglocks: RegisterFlags(0) // No register locks
            }) if regvals.len() == 0
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
