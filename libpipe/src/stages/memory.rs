use super::execute::ExecuteResult;
use crate::{Clock, Locks, PipelineStage, Registers, Status};
use libmem::module::MemoryModule;
use libseis::{
    registers::RegisterFlags,
    types::{Byte, Register, Short, Word},
};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
enum ReadMode {
    /// Reading a byte from memory
    ReadByte {
        address: Word,
        destination: Register,
        volatile: bool,
    },
    /// Reading a short from memory
    ReadShort {
        address: Word,
        destination: Register,
        volatile: bool,
    },
    /// Reading a word from memory
    ReadWord {
        address: Word,
        destination: Register,
        volatile: bool,
    },
}
use ReadMode::*;

#[derive(Debug, Clone, Serialize)]
enum WriteMode {
    /// Writing a byte to memory
    WriteByte {
        address: Word,
        value: Byte,
        volatile: bool,
    },
    /// Writing a short to memory
    WriteShort {
        address: Word,
        value: Short,
        volatile: bool,
    },
    /// Writing a word to memory
    WriteWord {
        address: Word,
        value: Word,
        volatile: bool,
    },
}
use WriteMode::*;

#[derive(Debug, Clone, Serialize, Default)]
enum State {
    #[default]
    Idle,
    Reading {
        mode: ReadMode,
        clocks: usize,
    },
    Writing {
        mode: WriteMode,
        clocks: usize,
    },
    Pushing {
        value: Word,
        sp: Word,
        clocks: usize,
    },
    Popping {
        register: Register,
        sp: Word,
        clocks: usize,
    },
    JsrPrep {
        address: Word,
        link: Word,
        sp: Word,
        bp: Word,
        clocks: usize,
    },
    RetPrep {
        link: Word,
        bp: Word,
        clocks: usize,
    },
    Ready {
        result: MemoryResult,
    },
    Squashed {
        wregs: RegisterFlags,
    },
    Halted,
}

impl State {
    fn is_halted(&self) -> bool {
        matches!(self, Halted)
    }

    fn is_waiting(&self) -> bool {
        matches!(
            self,
            Reading { .. }
                | Writing { .. }
                | Pushing { .. }
                | Popping { .. }
                | JsrPrep { .. }
                | RetPrep { .. }
        )
    }

    fn wait_time(&self) -> usize {
        match self {
            &Reading { clocks, .. }
            | &Writing { clocks, .. }
            | &Pushing { clocks, .. }
            | &Popping { clocks, .. }
            | &JsrPrep { clocks, .. }
            | &RetPrep { clocks, .. } => clocks,
            _ => 1,
        }
    }

    fn is_squashed(&self) -> bool {
        matches!(self, Squashed { .. })
    }

    fn is_idle(&self) -> bool {
        matches!(self, Idle)
    }
}
use State::*;

#[derive(Debug, Clone, Serialize)]
pub enum MemoryResult {
    /// Nothing
    Nop,
    /// Squashed instruction
    Squashed {
        wregs: RegisterFlags,
    },
    /// Write data back to a register
    WriteReg1 {
        destination: Register,
        value: Word,

        zf: bool,
        of: bool,
        eps: bool,
        nan: bool,
        inf: bool,
    },
    /// Write data back to a register, but also update the stack pointer
    WriteReg2 {
        register: Register,
        value: Word,
        sp: Word,

        zf: bool,
        of: bool,
        eps: bool,
        nan: bool,
        inf: bool,
    },
    /// Jump to a subroutine
    ///
    /// Write the current BP value to where the SP is, copy SP to BP,
    /// and increment the SP by 4
    JumpSubroutine {
        address: Word,
        link: Word,
        sp: Word,
        bp: Word,
    },
    /// Jump to a location in memory
    Jump {
        address: Word,
    },
    /// Return from a subroutine
    ///
    /// Write the current BP value to the SP, read the BP back in
    Return {
        address: Word,
        bp: Word,
        sp: Word,
    },
    /// Stop execution
    Halt,
    Ignore {
        wregs: RegisterFlags,
    },
    WriteStatus {
        zf: bool,
        of: bool,
        eps: bool,
        nan: bool,
        inf: bool,
    },
}

#[derive(Debug, Serialize, Default)]
pub struct Memory {
    state: State,
    forward: Option<MemoryResult>,
}

impl PipelineStage for Memory {
    type Prev = ExecuteResult;

    type Next = MemoryResult;

    fn clock(
        &mut self,
        _: Clock,
        _: &mut Registers,
        _: &mut Locks,
        memory: &mut dyn MemoryModule,
    ) -> Clock {
        if self.state.is_halted() {
            Clock::Halt
        } else {
            todo!()
        }
    }

    fn forward(&mut self, input: Status<Self::Prev>) -> Status<Self::Next> {
        use std::mem::take;
        if self.state.is_halted() {
            Status::Dry
        } else {
            let clocks = match input {
                Status::Stall(clocks) => clocks,
                Status::Flow(input) => match input {
                    ExecuteResult::Nop => {
                        self.state = Ready {
                            result: MemoryResult::Nop,
                        };
                        1
                    }
                    ExecuteResult::Subroutine {
                        address,
                        link,
                        sp,
                        bp,
                    } => {
                        self.state = JsrPrep {
                            address,
                            link,
                            sp,
                            bp,
                            clocks: 0,
                        };
                        1
                    }
                    ExecuteResult::JumpTo { address } => {
                        self.state = Ready {
                            result: MemoryResult::Jump { address },
                        };
                        1
                    }
                    ExecuteResult::Return { link, bp } => {
                        self.state = RetPrep {
                            bp,
                            link,
                            clocks: 0,
                        };
                        1
                    }
                    ExecuteResult::WriteReg {
                        destination,
                        value,
                        zf,
                        of,
                        eps,
                        nan,
                        inf,
                    } => {
                        self.state = Ready {
                            result: MemoryResult::WriteReg1 {
                                destination,
                                value,
                                zf,
                                of,
                                eps,
                                nan,
                                inf,
                            },
                        };
                        1
                    }
                    ExecuteResult::WriteStatus {
                        zf,
                        of,
                        eps,
                        nan,
                        inf,
                    } => {
                        self.state = Ready {
                            result: MemoryResult::WriteStatus {
                                zf,
                                of,
                                eps,
                                nan,
                                inf,
                            },
                        };
                        1
                    }
                    ExecuteResult::WriteMemByte {
                        address,
                        value,
                        volatile,
                    } => {
                        self.state = Writing {
                            mode: WriteByte {
                                address,
                                value,
                                volatile,
                            },
                            clocks: 0,
                        };
                        1
                    }
                    ExecuteResult::WriteMemShort {
                        address,
                        value,
                        volatile,
                    } => {
                        self.state = Writing {
                            mode: WriteShort {
                                address,
                                value,
                                volatile,
                            },
                            clocks: 0,
                        };
                        1
                    }
                    ExecuteResult::WriteMemWord {
                        address,
                        value,
                        volatile,
                    } => {
                        self.state = Writing {
                            mode: WriteWord {
                                address,
                                value,
                                volatile,
                            },
                            clocks: 0,
                        };
                        1
                    }
                    ExecuteResult::ReadMemByte {
                        address,
                        destination,
                        volatile,
                    } => {
                        self.state = Reading {
                            mode: ReadByte {
                                address,
                                destination,
                                volatile,
                            },
                            clocks: 0,
                        };
                        1
                    }
                    ExecuteResult::ReadMemShort {
                        address,
                        destination,
                        volatile,
                    } => {
                        self.state = Reading {
                            mode: ReadShort {
                                address,
                                destination,
                                volatile,
                            },
                            clocks: 0,
                        };
                        1
                    }
                    ExecuteResult::ReadMemWord {
                        address,
                        destination,
                        volatile,
                    } => {
                        self.state = Reading {
                            mode: ReadWord {
                                address,
                                destination,
                                volatile,
                            },
                            clocks: 0,
                        };
                        1
                    }
                    ExecuteResult::ReadRegStack { register, sp } => {
                        self.state = Popping {
                            register,
                            sp,
                            clocks: 0,
                        };
                        1
                    }
                    ExecuteResult::WriteRegStack { value, sp } => {
                        self.state = Pushing {
                            value,
                            sp,
                            clocks: 0,
                        };
                        1
                    }
                    ExecuteResult::Squash { regs } => {
                        self.state = Ready {
                            result: MemoryResult::Squashed { wregs: regs },
                        };
                        1
                    }
                    ExecuteResult::Ignore { regs } => {
                        self.state = Ready {
                            result: MemoryResult::Ignore { wregs: regs },
                        };
                        1
                    }
                    ExecuteResult::Halt => {
                        self.state = Halted;
                        1
                    }
                },
                Status::Ready => 1,
                Status::Squashed => 1,
                Status::Dry => unreachable!(),
            };

            match take(&mut self.forward) {
                Some(result) => Status::Flow(result),
                None if self.state.is_waiting() => {
                    Status::Stall(clocks.min(self.state.wait_time()))
                }
                None if self.state.is_squashed() => Status::Squashed,
                None if self.state.is_idle() => Status::Stall(clocks),
                None => Status::Ready,
            }
        }
    }
}
