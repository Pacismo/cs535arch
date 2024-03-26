mod resolver;

use super::decode::DecodeResult;
use crate::{reg_locks::Locks, regmap::RegMap, Clock, PipelineStage, Registers, Status};
use libmem::module::MemoryModule;
use libseis::{
    instruction_set::Instruction,
    registers::RegisterFlags,
    types::{Byte, Register, Short, Word},
};
use resolver::Resolver;
use serde::Serialize;
use std::mem::take;

/// Represents the steps that must be taken by the next stage of the pipeline
/// to complete an instruction
#[derive(Debug, Clone, Serialize)]
pub enum ExecuteResult {
    /// Nothing
    Nop,
    /// Stops execution
    Halt,
    /// Jump to a subroutine
    ///
    /// Store old BP and LP values
    Subroutine {
        /// Where to jump
        location: Word,
        /// Where to return to
        link: Word,
        /// The current value of the SP
        sp: Word,
        /// The current value of the BP
        bp: Word,
    },
    /// Jumps to a location
    JumpTo {
        /// Where to jump
        location: Word,
    },
    /// Return from a subroutine
    ///
    /// Restore old BP value
    Return {
        /// Where to jump back to
        link: Word,
        /// The current value of the SP
        sp: Word,
        /// The current value of the BP (from which the old BP is restored)
        bp: Word,
    },
    /// Write a value back to the register
    ///
    /// Sets the flag registers
    WriteReg {
        destination: Register,
        value: Word,

        zf: bool,
        of: bool,
        eps: bool,
        nan: bool,
        inf: bool,
    },
    /// Write back a sequence of bits to the status registers
    WriteStatus {
        zf: bool,
        of: bool,
        eps: bool,
        nan: bool,
        inf: bool,
    },
    /// Write a value from a register to a location in memory
    ///
    /// Only considers the least significant byte
    WriteMemByte {
        address: Word,
        value: Byte,
        volatile: bool,
    },
    /// Write a value from a register to a location in memory
    ///
    /// Only considers the least significant two bytes
    WriteMemShort {
        address: Word,
        value: Short,
        volatile: bool,
    },
    /// Write a value from a register to a location in memory
    WriteMemWord {
        address: Word,
        value: Word,
        volatile: bool,
    },
    /// Read a byte from a location in memory into a register
    ReadMemByte {
        address: Word,
        destination: Register,
        volatile: bool,
    },
    /// Read a short from a location in memory into a register
    ReadMemShort {
        address: Word,
        destination: Register,
        volatile: bool,
    },
    /// Read a word from a location in memory into a register
    ReadMemWord {
        address: Word,
        destination: Register,
        volatile: bool,
    },
    /// Read a register from the stack
    ReadRegStack { reg: Register, sp: Word },
    /// Write a register to the stack
    WriteRegStack { regval: Word, sp: Word },
    /// Squash all instructions in the pipeline
    Squash { regs: RegisterFlags },
}

impl ExecuteResult {
    #[inline]
    pub fn is_squash(&self) -> bool {
        matches!(self, ExecuteResult::Squash { .. })
    }
}

#[derive(Debug, Clone, Serialize, Default)]
enum State {
    #[default]
    Idle,
    Waiting {
        instruction: Instruction,
        wregs: RegisterFlags,
        rvals: RegMap,
        clocks: usize,
    },
    Ready {
        result: ExecuteResult,
        wregs: RegisterFlags,
    },
    Squashed {
        wregs: RegisterFlags,
    },
}

use State::*;

impl State {
    fn is_waiting(&self) -> bool {
        matches!(self, Waiting { .. })
    }

    fn is_squashed(&self) -> bool {
        matches!(self, Squashed { .. })
    }

    fn wait_time(&self) -> usize {
        if let Waiting { clocks, .. } = self {
            *clocks
        } else {
            1
        }
    }
}

#[derive(Debug, Serialize, Default)]
pub struct Execute {
    state: State,
    forward: Option<ExecuteResult>,
}

impl PipelineStage for Execute {
    type Prev = DecodeResult;
    type Next = ExecuteResult;

    fn clock(
        &mut self,
        clock: Clock,
        _: &mut Registers,
        _: &mut Locks,
        _: &mut dyn MemoryModule,
    ) -> Clock {
        match take(&mut self.state) {
            Idle => clock.to_ready(),
            Waiting {
                instruction,
                wregs,
                rvals,
                mut clocks,
            } => {
                if clock.is_squash() {
                    self.forward = None;
                    self.state = Squashed { wregs };
                    clock
                } else {
                    clocks = clocks.saturating_sub(clock.clocks());
                    if clocks == 0 {
                        let result = instruction.execute(rvals);
                        let squash = result.is_squash();

                        if clock.is_ready() {
                            self.forward = Some(result);
                            self.state = Idle;

                            if squash {
                                clock.to_squash()
                            } else {
                                clock.to_ready()
                            }
                        } else {
                            self.state = Ready { result, wregs };

                            if squash {
                                clock.to_squash()
                            } else {
                                clock.to_block()
                            }
                        }
                    } else {
                        self.state = Waiting {
                            instruction,
                            wregs,
                            rvals,
                            clocks,
                        };
                        clock.to_block()
                    }
                }
            }
            Ready { result, wregs } => {
                if clock.is_squash() {
                    self.forward = None;
                    self.state = Squashed { wregs };
                    clock
                } else if clock.is_ready() {
                    self.forward = Some(result);
                    self.state = Idle;
                    clock
                } else {
                    self.state = Ready { result, wregs };
                    clock.to_block()
                }
            }
            Squashed { wregs } => {
                if clock.is_ready() {
                    self.state = Idle;
                    self.forward = Some(ExecuteResult::Squash { regs: wregs });
                    clock
                } else {
                    self.state = Squashed { wregs };
                    clock.to_block()
                }
            }
        }
    }

    fn forward(&mut self, input: Status<Self::Prev>) -> Status<Self::Next> {
        let clk = match input {
            Status::Stall(clocks) => clocks,
            Status::Flow(DecodeResult::Forward {
                instruction,
                regvals,
                reglocks,
            }) => {
                let clocks = instruction.clock_requirement();
                self.state = State::Waiting {
                    instruction,
                    wregs: reglocks,
                    rvals: regvals,
                    clocks,
                };
                clocks
            }
            Status::Flow(DecodeResult::Squashed) => {
                self.state = Squashed {
                    wregs: Default::default(),
                };
                1
            }
            Status::Ready => 1,
            Status::Squashed => 1,
            Status::Dry => 0,
        };

        match take(&mut self.forward) {
            Some(xr) => Status::Flow(xr),
            None if self.state.is_waiting() => Status::Stall(clk.min(self.state.wait_time())),
            None if self.state.is_squashed() => Status::Squashed,
            None if clk == 0 => Status::Dry,
            None => Status::Ready,
        }
    }
}

// TODO: Write tests
