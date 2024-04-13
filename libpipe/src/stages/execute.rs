mod resolver;
#[cfg(test)]
mod tests;

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
#[derive(Debug, Clone)]
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
        address: Word,
        /// Where to return to
        link: Word,
        /// The current value of the SP
        sp: Word,
        /// The current value of the BP
        bp: Word,
        /// The current value of the LP
        lp: Word,
    },
    /// Jumps to a location
    JumpTo {
        /// Where to jump
        address: Word,
    },
    /// Return from a subroutine
    ///
    /// Restore old BP value
    Return {
        /// Where to jump back to
        link: Word,
        /// The current value of the BP (from which the old BP is restored)
        bp: Word,
    },
    /// Write a value back to the register
    ///
    /// Sets the flag registers
    WriteReg {
        /// The destination register
        destination: Register,
        /// What to write to the destination register
        value: Word,

        /// ZF register state
        zf: bool,
        /// OF register state
        of: bool,
        /// EPS register state
        eps: bool,
        /// NAN register state
        nan: bool,
        /// INF register state
        inf: bool,
    },
    /// Write back a sequence of bits to the status registers
    WriteStatus {
        /// ZF register state
        zf: bool,
        /// OF register state
        of: bool,
        /// EPS register state
        eps: bool,
        /// NAN register state
        nan: bool,
        /// INF register state
        inf: bool,
    },
    /// Write a value from a register to a location in memory
    ///
    /// Only considers the least significant byte
    WriteMemByte {
        /// Where to write to
        address: Word,
        /// What to write
        value: Byte,
        /// Whether to skip the cache
        volatile: bool,
    },
    /// Write a value from a register to a location in memory
    ///
    /// Only considers the least significant two bytes
    WriteMemShort {
        /// Where to write to
        address: Word,
        /// What to write
        value: Short,
        /// Whether to skip the cache
        volatile: bool,
    },
    /// Write a value from a register to a location in memory
    WriteMemWord {
        /// Where to write to
        address: Word,
        /// What to write
        value: Word,
        /// Whether to skip the cache
        volatile: bool,
    },
    /// Read a byte from a location in memory into a register
    ReadMemByte {
        /// Where to read from
        address: Word,
        /// Where to write the read value
        destination: Register,
        /// Whether to skip the cache
        volatile: bool,
    },
    /// Read a short from a location in memory into a register
    ReadMemShort {
        /// Where to read from
        address: Word,
        /// Where to write the read value
        destination: Register,
        /// Whether to skip the cache
        volatile: bool,
    },
    /// Read a word from a location in memory into a register
    ReadMemWord {
        /// Where to read from
        address: Word,
        /// Where to write the read value
        destination: Register,
        /// Whether to skip the cache
        volatile: bool,
    },
    /// Read a register from the stack
    ReadRegStack {
        /// The register to write the value to
        register: Register,
        /// The current stack pointer value
        sp: Word,
    },
    /// Write a register to the stack
    WriteRegStack {
        /// The value to write to the stack
        value: Word,
        /// The current stack pointer value
        sp: Word,
    },
    /// Squash all instructions in the pipeline
    Squash {
        /// The set of all register locks owned by the squashed instruction
        regs: RegisterFlags,
    },
    /// Ignore a jump instruction
    Ignore {
        /// The set of all register locks owned by the squashed instruction
        regs: RegisterFlags,
    },
    /// Simply remove a word from the stack (since the destination was invalid)
    PopStack {
        /// The current stack pointer value
        sp: Word,
    },
}

impl ExecuteResult {
    /// Returns true if the result requires a squash signal to be sent
    #[inline]
    pub fn should_squash(&self) -> bool {
        matches!(
            self,
            Self::JumpTo { .. } | Self::Subroutine { .. } | Self::Return { .. }
        )
    }

    /// Returns true if the result is a halt
    #[inline]
    pub fn is_halt(&self) -> bool {
        matches!(self, ExecuteResult::Halt)
    }
}

/// Represents the current state of the execute stage
#[derive(Debug, Clone, Default)]
pub enum State {
    /// Awaiting the next instruction
    #[default]
    Idle,
    /// Executing the next instruction
    Executing {
        /// The instruction being executed
        instruction: Instruction,
        /// The registers owned by this instruction
        wregs: RegisterFlags,
        /// The values read by the decode stage
        rvals: RegMap,
        /// The number of clocks required before the instruction is finished executing
        clocks: usize,
    },
    /// This stage is ready to forward a result
    Ready {
        /// The result to be forwarded
        result: ExecuteResult,
        /// The registers owned by this instruction
        wregs: RegisterFlags,
    },
    /// This stage has been squashed and is waiting to send the signal forward
    Squashed {
        /// The register locks being held by this (squashed) instruction
        wregs: RegisterFlags,
    },
    /// This stage has been halted
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
            Executing {
                instruction,
                wregs,
                rvals,
                clocks,
            } => {
                let mut map = serializer.serialize_map(Some(5))?;
                map.serialize_entry("state", "executing")?;
                map.serialize_entry("instruction", &instruction.to_string())?;
                map.serialize_entry("write_regs", wregs)?;
                map.serialize_entry("reg_values", rvals)?;
                map.serialize_entry("clocks", clocks)?;
                map.end()
            }
            Ready { wregs, .. } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("state", "ready")?;
                map.serialize_entry("write_regs", wregs)?;
                map.end()
            }
            Squashed { wregs, .. } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("state", "squashed")?;
                map.serialize_entry("write_regs", wregs)?;
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
    fn is_waiting(&self) -> bool {
        matches!(self, Executing { .. })
    }

    fn is_squashed(&self) -> bool {
        matches!(self, Squashed { .. })
    }

    fn wait_time(&self) -> usize {
        if let Executing { clocks, .. } = self {
            *clocks
        } else {
            1
        }
    }

    fn is_idle(&self) -> bool {
        matches!(self, Idle)
    }

    fn is_halted(&self) -> bool {
        matches!(self, Halted)
    }
}

/// Represents the execute pipeline stage
#[derive(Debug, Default)]
pub struct Execute {
    state: State,
    forward: Option<ExecuteResult>,
}

impl Serialize for Execute {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.state.serialize(serializer)
    }
}

impl PipelineStage for Execute {
    type Prev = DecodeResult;
    type Next = ExecuteResult;
    type State = State;

    fn clock(
        &mut self,
        clock: Clock,
        _: &mut Registers,
        _: &mut Locks,
        _: &mut dyn MemoryModule,
    ) -> Clock {
        match take(&mut self.state) {
            Idle => clock.to_ready(),
            Executing {
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
                        let squash = result.should_squash();

                        if clock.is_ready() {
                            let is_halt = result.is_halt();

                            self.forward = Some(result);
                            if is_halt {
                                self.state = Halted;
                            } else {
                                self.state = Idle;
                            }

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
                        self.state = Executing {
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
                    let is_halt = result.is_halt();

                    self.forward = Some(result);
                    if is_halt {
                        self.state = Halted;
                    } else {
                        self.state = Idle;
                    }
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
            Halted => {
                self.state = Halted;
                Clock::Halt
            }
        }
    }

    fn forward(&mut self, input: Status<Self::Prev>) -> Status<Self::Next> {
        if self.state.is_halted() && self.forward.is_none() {
            Status::Dry
        } else {
            let (clocks, rix) = match input {
                Status::Stall(clocks) => (clocks, 0),
                Status::Flow(DecodeResult::Forward {
                    instruction,
                    regvals,
                    reglocks,
                }) => {
                    let clocks = instruction.clock_requirement();
                    self.state = State::Executing {
                        instruction,
                        wregs: reglocks,
                        rvals: regvals,
                        clocks,
                    };
                    (clocks, 0)
                }
                Status::Flow(DecodeResult::Squashed) => {
                    self.state = Squashed {
                        wregs: Default::default(),
                    };
                    (1, 0)
                }
                Status::Ready(r) => (1, r),
                Status::Squashed => (1, 0),
                Status::Dry => unreachable!(),
            };

            match take(&mut self.forward) {
                Some(xr) => Status::Flow(xr),
                None if self.state.is_waiting() && rix == 2 => {
                    Status::Stall(self.state.wait_time())
                }
                None if self.state.is_waiting() => {
                    Status::Stall(clocks.min(self.state.wait_time()))
                }
                None if self.state.is_squashed() => Status::Squashed,
                None if self.state.is_idle() => Status::Stall(clocks),
                None => Status::Ready(rix + 1),
            }
        }
    }

    fn get_state(&self) -> &State {
        &self.state
    }
}

// TODO: Write tests
