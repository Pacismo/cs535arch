use super::execute::ExecuteResult;
use crate::{Clock, Locks, PipelineStage, Registers, Status};
use libmem::module::{MemoryModule, Status as MemStatus};
use libseis::{
    registers::{RegisterFlags, BP, EPS, INF, LP, NAN, OF, PC, SP, ZF},
    types::{Byte, Register, Short, Word},
};
use serde::Serialize;
use std::fmt::Display;

#[derive(Debug, Clone, Copy, Serialize)]
pub enum ReadMode {
    /// Reading a byte from memory
    ReadByte { address: Word, volatile: bool },
    /// Reading a short from memory
    ReadShort { address: Word, volatile: bool },
    /// Reading a word from memory
    ReadWord { address: Word, volatile: bool },
}

impl ReadMode {
    fn execute(self, mem: &mut dyn MemoryModule) -> Result<Word, usize> {
        match self {
            ReadByte {
                address, volatile, ..
            } => {
                match if volatile {
                    mem.read_byte_volatile(address)
                } else {
                    mem.read_byte(address)
                } {
                    Ok(value) => Ok(value as Word),
                    Err(MemStatus::Busy(clocks)) => Err(clocks),
                    Err(MemStatus::Idle) => unreachable!(),
                }
            }
            ReadShort {
                address, volatile, ..
            } => match if volatile {
                mem.read_short_volatile(address)
            } else {
                mem.read_short(address)
            } {
                Ok(value) => Ok(value as Word),
                Err(MemStatus::Busy(clocks)) => Err(clocks),
                Err(MemStatus::Idle) => unreachable!(),
            },
            ReadWord {
                address, volatile, ..
            } => match if volatile {
                mem.read_word_volatile(address)
            } else {
                mem.read_word(address)
            } {
                Ok(value) => Ok(value),
                Err(MemStatus::Busy(clocks)) => Err(clocks),
                Err(MemStatus::Idle) => unreachable!(),
            },
        }
    }
}
use ReadMode::*;

#[derive(Debug, Clone, Copy, Serialize)]
pub enum WriteMode {
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

impl WriteMode {
    fn execute(self, mem: &mut dyn MemoryModule) -> Option<usize> {
        match self {
            WriteByte {
                address,
                value,
                volatile,
            } => match if volatile {
                mem.write_byte_volatile(address, value)
            } else {
                mem.write_byte(address, value)
            } {
                MemStatus::Busy(clocks) => Some(clocks),
                MemStatus::Idle => None,
            },
            WriteShort {
                address,
                value,
                volatile,
            } => match if volatile {
                mem.write_short_volatile(address, value)
            } else {
                mem.write_short(address, value)
            } {
                MemStatus::Busy(clocks) => Some(clocks),
                MemStatus::Idle => None,
            },
            WriteWord {
                address,
                value,
                volatile,
            } => match if volatile {
                mem.write_word_volatile(address, value)
            } else {
                mem.write_word(address, value)
            } {
                MemStatus::Busy(clocks) => Some(clocks),
                MemStatus::Idle => None,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum JsrPrepState {
    WritingLp,
    WritingBp,
}
use JsrPrepState::*;

impl Display for JsrPrepState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WritingLp => write!(f, "Writing the LP"),
            WritingBp => write!(f, "Writing the BP"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum RetPrepState {
    ReadingBp,
    ReadingLp(Word),
}
use RetPrepState::*;

impl Display for RetPrepState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadingLp(_) => write!(f, "Reading the LP"),
            ReadingBp => write!(f, "Reading the BP"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Default)]
pub enum State {
    #[default]
    Idle,
    Reading {
        mode: ReadMode,
        destination: Register,
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
        destination: Register,
        sp: Word,
        clocks: usize,
    },
    DummyPop {
        sp: Word,
        clocks: usize,
    },
    /// Write the current BP value to where the SP is, increment the
    /// SP by 4, and set BP to the current value of SP
    JsrPrep {
        address: Word,
        link: Word,
        sp: Word,
        bp: Word,
        lp: Word,
        state: JsrPrepState,
        clocks: usize,
    },
    /// Write the current BP value to the SP, read the BP back in
    RetPrep {
        link: Word,
        bp: Word,
        state: RetPrepState,
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
    fn is_halted(self) -> bool {
        matches!(self, Halted)
    }

    fn is_waiting(self) -> bool {
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

    fn wait_time(self) -> usize {
        match self {
            Reading { clocks, .. }
            | Writing { clocks, .. }
            | Pushing { clocks, .. }
            | Popping { clocks, .. }
            | DummyPop { clocks, .. }
            | JsrPrep { clocks, .. }
            | RetPrep { clocks, .. } => clocks,
            _ => 1,
        }
    }

    fn is_squashed(self) -> bool {
        matches!(self, Squashed { .. })
    }

    fn is_idle(self) -> bool {
        matches!(self, Idle)
    }

    fn get_wregs(self) -> RegisterFlags {
        match self {
            Idle => RegisterFlags::default(),
            Reading { destination, .. } => [destination, ZF, OF, EPS, NAN, INF].into(),
            Writing { .. } => [].into(),
            Pushing { .. } => [SP].into(),
            Popping { destination, .. } => [destination, SP, ZF, OF, EPS, NAN, INF].into(),
            DummyPop { .. } => [SP, ZF, OF, EPS, NAN, INF].into(),
            JsrPrep { .. } => [PC, SP, BP, LP].into(),
            RetPrep { .. } => [PC, BP, SP].into(),
            Ready { result } => match result {
                MemoryResult::Squashed { wregs } | MemoryResult::Ignore { wregs } => wregs,
                MemoryResult::WriteReg1 { destination, .. } => {
                    [destination, ZF, OF, EPS, NAN, INF].into()
                }
                MemoryResult::WriteReg2 { register, .. } => {
                    [register, SP, ZF, OF, EPS, NAN, INF].into()
                }
                MemoryResult::JumpSubroutine { .. } => [PC, SP, BP, LP].into(),
                MemoryResult::Jump { .. } => [PC].into(),
                MemoryResult::Return { .. } => [SP, BP, PC].into(),
                MemoryResult::Halt => [].into(),
                MemoryResult::WriteStatus { .. } => [ZF, OF, EPS, NAN, INF].into(),
                _ => [].into(),
            },
            Squashed { wregs } => wregs,
            Halted => [].into(),
        }
    }
}
use State::*;

#[derive(Debug, Clone, Copy, Serialize)]
pub enum MemoryResult {
    /// Nothing
    Nop,
    /// Squashed instruction
    Squashed {
        wregs: RegisterFlags,
    },
    /// Write data back to a register without any status information
    WriteRegNoStatus {
        destination: Register,
        value: Word,
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
    Return {
        address: Word,
        bp: Word,
        sp: Word,
        lp: Word,
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
    type State = State;

    fn clock(
        &mut self,
        clock: Clock,
        _: &mut Registers,
        _: &mut Locks,
        memory: &mut dyn MemoryModule,
    ) -> Clock {
        if self.state.is_halted() {
            Clock::Halt
        } else {
            if clock.is_squash() {
                self.state = Squashed {
                    wregs: self.state.get_wregs(),
                };
                self.forward = None;
                clock
            } else {
                match self.state {
                    Idle => clock.to_ready(),
                    Reading {
                        mode, destination, ..
                    } => match mode.execute(memory) {
                        Ok(value) => {
                            if clock.is_ready() {
                                self.forward = Some(MemoryResult::WriteReg1 {
                                    destination,
                                    value,
                                    zf: value == 0,
                                    of: false,
                                    eps: false,
                                    nan: false,
                                    inf: false,
                                });
                                self.state = Idle;
                                clock.to_ready()
                            } else {
                                self.state = Ready {
                                    result: MemoryResult::WriteReg1 {
                                        destination,
                                        value,
                                        zf: value == 0,
                                        of: false,
                                        eps: false,
                                        nan: false,
                                        inf: false,
                                    },
                                };
                                clock.to_block()
                            }
                        }
                        Err(clocks) => {
                            self.state = Reading {
                                mode,
                                destination,
                                clocks,
                            };
                            clock.to_block()
                        }
                    },
                    Writing { mode, .. } => match mode.execute(memory) {
                        Some(clocks) => {
                            self.state = Writing { mode, clocks };
                            clock.to_block()
                        }
                        None => {
                            if clock.is_ready() {
                                self.state = Idle;
                                self.forward = Some(MemoryResult::Nop);
                                clock.to_ready()
                            } else {
                                self.state = Ready {
                                    result: MemoryResult::Nop,
                                };
                                self.forward = None;
                                clock.to_ready()
                            }
                        }
                    },
                    Pushing { value, sp, .. } => match memory.write_word(sp, value) {
                        MemStatus::Busy(clocks) => {
                            self.state = Pushing { value, sp, clocks };
                            clock.to_block()
                        }
                        MemStatus::Idle => {
                            if clock.is_ready() {
                                self.state = Idle;
                                self.forward = Some(MemoryResult::WriteRegNoStatus {
                                    destination: SP,
                                    value: sp.wrapping_add(4),
                                });
                                clock.to_ready()
                            } else {
                                self.state = Ready {
                                    result: MemoryResult::WriteRegNoStatus {
                                        destination: SP,
                                        value: sp.wrapping_add(4),
                                    },
                                };
                                clock.to_block()
                            }
                        }
                    },
                    Popping {
                        destination, sp, ..
                    } => match memory.read_word(sp) {
                        Err(MemStatus::Busy(clocks)) => {
                            self.state = Popping {
                                destination,
                                sp,
                                clocks,
                            };
                            clock.to_block()
                        }
                        Err(MemStatus::Idle) => unreachable!(),
                        Ok(value) => {
                            if clock.is_ready() {
                                self.state = Idle;
                                self.forward = Some(MemoryResult::WriteReg2 {
                                    register: destination,
                                    value,
                                    sp,
                                    zf: value == 0,
                                    of: false,
                                    eps: false,
                                    nan: false,
                                    inf: false,
                                });
                                clock.to_ready()
                            } else {
                                self.state = Ready {
                                    result: MemoryResult::WriteReg2 {
                                        register: destination,
                                        value,
                                        sp,
                                        zf: value == 0,
                                        of: false,
                                        eps: false,
                                        nan: false,
                                        inf: false,
                                    },
                                };
                                clock.to_block()
                            }
                        }
                    },
                    DummyPop { sp, .. } => match memory.read_word(sp) {
                        Ok(_) => {
                            if clock.is_ready() {
                                self.state = Idle;
                                self.forward = Some(MemoryResult::WriteRegNoStatus {
                                    destination: SP,
                                    value: sp,
                                });
                                clock.to_ready()
                            } else {
                                self.state = Ready {
                                    result: MemoryResult::WriteRegNoStatus {
                                        destination: SP,
                                        value: sp,
                                    },
                                };
                                clock.to_block()
                            }
                        }
                        Err(MemStatus::Busy(clocks)) => {
                            self.state = DummyPop { sp, clocks };
                            clock.to_block()
                        }
                        Err(MemStatus::Idle) => unreachable!(),
                    },
                    JsrPrep {
                        address,
                        link,
                        sp,
                        bp,
                        lp,
                        state,
                        ..
                    } => match state {
                        WritingLp => match memory.write_word(sp, link) {
                            MemStatus::Idle => {
                                self.state = JsrPrep {
                                    address,
                                    link,
                                    sp,
                                    bp,
                                    lp,
                                    state: WritingBp,
                                    clocks: 1,
                                };
                                clock.to_block()
                            }
                            MemStatus::Busy(clocks) => {
                                self.state = JsrPrep {
                                    address,
                                    link,
                                    sp,
                                    bp,
                                    lp,
                                    state,
                                    clocks,
                                };
                                clock.to_block()
                            }
                        },
                        WritingBp => match memory.write_word(sp, bp) {
                            MemStatus::Idle => {
                                if clock.is_ready() {
                                    self.state = Idle;
                                    self.forward = Some(MemoryResult::JumpSubroutine {
                                        address,
                                        link,
                                        sp,
                                        bp,
                                    });
                                    clock.to_ready()
                                } else {
                                    self.state = Ready {
                                        result: MemoryResult::JumpSubroutine {
                                            address,
                                            link,
                                            sp,
                                            bp,
                                        },
                                    };
                                    clock.to_block()
                                }
                            }
                            MemStatus::Busy(clocks) => {
                                self.state = JsrPrep {
                                    address,
                                    link,
                                    sp,
                                    bp,
                                    lp,
                                    state,
                                    clocks,
                                };
                                clock.to_block()
                            }
                        },
                    },
                    RetPrep {
                        link, bp, state, ..
                    } => match state {
                        ReadingBp => match memory.read_word(bp.wrapping_sub(4)) {
                            Ok(value) => {
                                self.state = RetPrep {
                                    link,
                                    bp,
                                    state: ReadingLp(value),
                                    clocks: 1,
                                };
                                clock.to_block()
                            }
                            Err(MemStatus::Busy(clocks)) => {
                                self.state = RetPrep {
                                    link,
                                    bp,
                                    state,
                                    clocks,
                                };
                                clock.to_block()
                            }
                            Err(MemStatus::Idle) => unreachable!(),
                        },
                        ReadingLp(bpval) => match memory.read_word(bp.wrapping_sub(8)) {
                            Ok(value) => {
                                if clock.is_ready() {
                                    self.state = Idle;
                                    self.forward = Some(MemoryResult::Return {
                                        address: link,
                                        bp: bpval,
                                        sp: bp.wrapping_sub(8),
                                        lp: value,
                                    });
                                    clock.to_ready()
                                } else {
                                    self.state = Ready {
                                        result: MemoryResult::Return {
                                            address: link,
                                            bp: bpval,
                                            sp: bp.wrapping_sub(8),
                                            lp: value,
                                        },
                                    };
                                    clock.to_block()
                                }
                            }
                            Err(MemStatus::Busy(clocks)) => {
                                self.state = RetPrep {
                                    link,
                                    bp,
                                    state,
                                    clocks,
                                };
                                clock.to_block()
                            }
                            Err(MemStatus::Idle) => unreachable!(),
                        },
                    },
                    Ready { result } => {
                        if clock.is_ready() {
                            if matches!(result, MemoryResult::Halt) {
                                self.state = Halted;
                            } else {
                                self.state = Idle;
                            }
                            self.forward = Some(result);
                            clock.to_ready()
                        } else {
                            self.state = Ready { result };
                            clock.to_block()
                        }
                    }
                    Squashed { wregs } => {
                        if clock.is_ready() {
                            self.forward = Some(MemoryResult::Squashed { wregs });
                            self.state = Idle;
                            clock.to_ready()
                        } else {
                            self.state = Squashed { wregs };
                            clock.to_block()
                        }
                    }
                    Halted => unreachable!(),
                }
            }
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
                        lp,
                    } => {
                        self.state = JsrPrep {
                            address,
                            link,
                            sp,
                            bp,
                            lp,
                            state: WritingLp,
                            clocks: 1,
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
                            state: ReadingBp,
                            clocks: 1,
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
                            clocks: 1,
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
                            clocks: 1,
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
                            clocks: 1,
                        };
                        1
                    }
                    ExecuteResult::ReadMemByte {
                        address,
                        destination,
                        volatile,
                    } => {
                        self.state = Reading {
                            mode: ReadByte { address, volatile },
                            destination,
                            clocks: 1,
                        };
                        1
                    }
                    ExecuteResult::ReadMemShort {
                        address,
                        destination,
                        volatile,
                    } => {
                        self.state = Reading {
                            mode: ReadShort { address, volatile },
                            destination,
                            clocks: 1,
                        };
                        1
                    }
                    ExecuteResult::ReadMemWord {
                        address,
                        destination,
                        volatile,
                    } => {
                        self.state = Reading {
                            mode: ReadWord { address, volatile },
                            destination,
                            clocks: 1,
                        };
                        1
                    }
                    ExecuteResult::ReadRegStack { register, sp } => {
                        self.state = Popping {
                            destination: register,
                            sp,
                            clocks: 1,
                        };
                        1
                    }
                    ExecuteResult::WriteRegStack { value, sp } => {
                        self.state = Pushing {
                            value,
                            sp,
                            clocks: 1,
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
                        self.state = Ready {
                            result: MemoryResult::Halt,
                        };
                        1
                    }
                    ExecuteResult::PopStack { sp } => {
                        self.state = DummyPop { sp, clocks: 1 };
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

    fn get_state(&self) -> &State {
        &self.state
    }
}
