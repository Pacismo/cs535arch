//! Memory stage

use super::execute::ExecuteResult;
use crate::{Clock, Locks, PipelineStage, Registers, Status};
use libmem::module::{MemoryModule, Status as MemStatus};
use libseis::{
    registers::{get_name, RegisterFlags, BP, EPS, INF, LP, NAN, OF, PC, SP, ZF},
    types::{Byte, Register, Short, Word},
};
use serde::Serialize;
use std::fmt::Display;

/// Represents the kind of read being done
#[derive(Debug, Clone, Copy)]
pub enum ReadMode {
    /// Reading a byte from memory
    ReadByte {
        /// Where to read the value from
        address: Word,
        /// Whether to skip the cache
        volatile: bool,
    },
    /// Reading a short from memory
    ReadShort {
        /// Where to read the value from
        address: Word,
        /// Whether to skip the cache
        volatile: bool,
    },
    /// Reading a word from memory
    ReadWord {
        /// Where to read the value from
        address: Word,
        /// Whether to skip the cache
        volatile: bool,
    },
}

impl Serialize for ReadMode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(3))?;

        match self {
            ReadByte { address, volatile } => {
                map.serialize_entry("mode", "byte")?;
                map.serialize_entry("address", address)?;
                map.serialize_entry("volatile", volatile)?;
            }
            ReadShort { address, volatile } => {
                map.serialize_entry("mode", "short")?;
                map.serialize_entry("address", address)?;
                map.serialize_entry("volatile", volatile)?;
            }
            ReadWord { address, volatile } => {
                map.serialize_entry("mode", "word")?;
                map.serialize_entry("address", address)?;
                map.serialize_entry("volatile", volatile)?;
            }
        }

        map.end()
    }
}
use ReadMode::*;

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

/// Represents the kind of write being done
#[derive(Debug, Clone, Copy)]
pub enum WriteMode {
    /// Writing a byte to memory
    WriteByte {
        /// Where to write to
        address: Word,
        /// What to write
        value: Byte,
        /// Whether to skip the cache
        volatile: bool,
    },
    /// Writing a short to memory
    WriteShort {
        /// Where to write to
        address: Word,
        /// What to write
        value: Short,
        /// Whether to skip the cache
        volatile: bool,
    },
    /// Writing a word to memory
    WriteWord {
        /// Where to write to
        address: Word,
        /// What to write
        value: Word,
        /// Whether to skip the cache
        volatile: bool,
    },
}
use WriteMode::*;

impl Serialize for WriteMode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(4))?;

        match self {
            WriteByte {
                address,
                value,
                volatile,
            } => {
                map.serialize_entry("mode", "byte")?;
                map.serialize_entry("value", value)?;
                map.serialize_entry("address", address)?;
                map.serialize_entry("volatile", volatile)?;
            }
            WriteShort {
                address,
                value,
                volatile,
            } => {
                map.serialize_entry("mode", "short")?;
                map.serialize_entry("value", value)?;
                map.serialize_entry("address", address)?;
                map.serialize_entry("volatile", volatile)?;
            }
            WriteWord {
                address,
                value,
                volatile,
            } => {
                map.serialize_entry("mode", "word")?;
                map.serialize_entry("value", value)?;
                map.serialize_entry("address", address)?;
                map.serialize_entry("volatile", volatile)?;
            }
        }

        map.end()
    }
}

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

/// Represents the state of the JSR instruction
#[derive(Debug, Clone, Copy)]
pub enum JsrPrepState {
    /// We are writing the current value of the link pointer
    WritingLp,
    /// We are writing the current value of the stack base pointer
    WritingBp,
}
use JsrPrepState::*;

impl Serialize for JsrPrepState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            WritingLp => serializer.serialize_str("writing_lp"),
            WritingBp => serializer.serialize_str("writing_bp"),
        }
    }
}

impl Display for JsrPrepState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WritingLp => write!(f, "Writing the LP"),
            WritingBp => write!(f, "Writing the BP"),
        }
    }
}

/// Represents the state of the RET instruction
#[derive(Debug, Clone, Copy)]
pub enum RetPrepState {
    /// We are reading the base pointer from the stack
    ReadingBp,
    /// We are reading the link pointer from the stack
    ReadingLp(Word),
}
use RetPrepState::*;

impl Serialize for RetPrepState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            ReadingBp => serializer.serialize_str("reading_bp"),
            ReadingLp(..) => serializer.serialize_str("reading_lp"),
        }
    }
}

impl Display for RetPrepState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadingLp(_) => write!(f, "Reading the LP"),
            ReadingBp => write!(f, "Reading the BP"),
        }
    }
}

/// Represents the state of the memory stage
#[derive(Debug, Clone, Copy, Default)]
pub enum State {
    /// Nothing is happening
    #[default]
    Idle,
    /// This stage is reading memory
    Reading {
        /// Read mode
        mode: ReadMode,
        /// Destination register
        destination: Register,
        /// Clock requirement metadata
        clocks: usize,
    },
    /// This stage is writing to memory
    Writing {
        /// Write mode
        mode: WriteMode,
        /// Clock requirement metadata
        clocks: usize,
    },
    /// This stage is pushing data to the stack
    Pushing {
        /// What is being written to the stack
        value: Word,
        /// Where it is being written to
        sp: Word,
        /// Clock requirement metadata
        clocks: usize,
    },
    /// This stage is removing data from the stack
    Popping {
        /// Where the read data is being written to
        destination: Register,
        /// Where the data is being read from
        sp: Word,
        /// Clock requirement metadata
        clocks: usize,
    },
    /// A pop to an invalid register
    DummyPop {
        /// Current stack pointer value
        sp: Word,
        /// Clock requirement metadata
        clocks: usize,
    },
    /// Preparing for a jump to a subroutine
    JsrPrep {
        /// Where to jump
        address: Word,
        /// Where to return
        link: Word,
        /// The current SP value
        sp: Word,
        /// The current BP value
        bp: Word,
        /// The curren LP value
        lp: Word,
        /// Where in the preparation we are
        state: JsrPrepState,
        /// Clock requirement metadata
        clocks: usize,
    },
    /// Write the current BP value to the SP, read the BP back in
    RetPrep {
        /// Where to return to
        link: Word,
        /// Current BP value
        bp: Word,
        /// Return preparation state
        state: RetPrepState,
        /// Clock requirement metadata
        clocks: usize,
    },
    /// Ready to forward the result
    Ready {
        /// The result to forward
        result: MemoryResult,
    },
    /// This stage was squashed
    Squashed {
        /// The values of the write registers
        wregs: RegisterFlags,
    },
    /// This stage is halting (and writing data back to memory)
    Halting(usize),
    /// This stage is halted
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
            Idle => serializer.collect_map([("state", "idle")]),
            Reading {
                mode,
                destination,
                clocks,
            } => {
                let mut map = serializer.serialize_map(Some(4))?;
                map.serialize_entry("state", "reading")?;
                map.serialize_entry("mode", mode)?;
                map.serialize_entry("destination", destination)?;
                map.serialize_entry("clocks", clocks)?;
                map.end()
            }
            Writing { mode, clocks } => {
                let mut map = serializer.serialize_map(Some(3))?;
                map.serialize_entry("state", "writing")?;
                map.serialize_entry("mode", mode)?;
                map.serialize_entry("clocks", clocks)?;
                map.end()
            }
            Pushing { value, sp, clocks } => {
                let mut map = serializer.serialize_map(Some(4))?;
                map.serialize_entry("state", "pushing")?;
                map.serialize_entry("value", value)?;
                map.serialize_entry("sp", sp)?;
                map.serialize_entry("clocks", clocks)?;
                map.end()
            }
            Popping {
                destination,
                sp,
                clocks,
            } => {
                let mut map = serializer.serialize_map(Some(4))?;
                map.serialize_entry("state", "popping")?;
                map.serialize_entry("destination", destination)?;
                map.serialize_entry("sp", sp)?;
                map.serialize_entry("clocks", clocks)?;
                map.end()
            }
            DummyPop { sp, clocks } => {
                let mut map = serializer.serialize_map(Some(3))?;
                map.serialize_entry("state", "dummy_pop")?;
                map.serialize_entry("sp", sp)?;
                map.serialize_entry("clocks", clocks)?;
                map.end()
            }
            JsrPrep {
                address,
                link,
                sp,
                bp,
                lp,
                state,
                clocks,
            } => {
                let mut map = serializer.serialize_map(Some(8))?;
                map.serialize_entry("state", "jsr_prep")?;
                map.serialize_entry("address", address)?;
                map.serialize_entry("link", link)?;
                map.serialize_entry("bp", bp)?;
                map.serialize_entry("sp", sp)?;
                map.serialize_entry("lp", lp)?;
                map.serialize_entry("state", state)?;
                map.serialize_entry("clocks", clocks)?;
                map.end()
            }
            RetPrep {
                link,
                bp,
                state,
                clocks,
            } => {
                let mut map = serializer.serialize_map(Some(5))?;
                map.serialize_entry("state", "return_prep")?;
                map.serialize_entry("link", link)?;
                map.serialize_entry("bp", bp)?;
                map.serialize_entry("state", state)?;
                map.serialize_entry("clocks", clocks)?;
                map.end()
            }
            Ready { .. } => serializer.collect_map([("state", "ready")]),
            Squashed { wregs } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("state", "squashed")?;
                map.serialize_entry("write_regs", wregs)?;
                map.end()
            }
            Halting(..) => serializer.collect_map([("state", "halting")]),
            Halted => serializer.collect_map([("state", "halted")]),
        }
    }
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
                | Halting(..)
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
            | RetPrep { clocks, .. }
            | Halting(clocks) => clocks,
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
                MemoryResult::WriteReg2 {
                    destination: register,
                    ..
                } => [register, SP, ZF, OF, EPS, NAN, INF].into(),
                MemoryResult::JumpSubroutine { .. } => [PC, SP, BP, LP].into(),
                MemoryResult::Jump { .. } => [PC].into(),
                MemoryResult::Return { .. } => [SP, BP, PC].into(),
                MemoryResult::Halt => [].into(),
                MemoryResult::WriteStatus { .. } => [ZF, OF, EPS, NAN, INF].into(),
                _ => [].into(),
            },
            Squashed { wregs } => wregs,
            Halting(..) | Halted => [].into(),
        }
    }
}

/// The result of the memory stage
#[derive(Debug, Clone, Copy)]
pub enum MemoryResult {
    /// Nothing
    Nop,
    /// Squashed instruction
    Squashed {
        /// The held register locks
        wregs: RegisterFlags,
    },
    /// Write data back to a register without any status information
    WriteRegNoStatus {
        /// Where to write the value
        destination: Register,
        /// What to write
        value: Word,
    },
    /// Write data back to a register
    WriteReg1 {
        /// Where to write the value
        destination: Register,
        /// What to write
        value: Word,

        /// Zero flag state
        zf: bool,
        /// Overflow flag state
        of: bool,
        /// Epsilon-equality flag state
        eps: bool,
        /// NaN flag state
        nan: bool,
        /// Infinity flag state
        inf: bool,
    },
    /// Write data back to a register, but also update the stack pointer
    WriteReg2 {
        /// Where to write the value
        destination: Register,
        /// What to write
        value: Word,
        /// What to set the stack pointer to
        sp: Word,

        /// Zero flag state
        zf: bool,
        /// Overflow flag state
        of: bool,
        /// Epsilon-equality flag state
        eps: bool,
        /// NaN flag state
        nan: bool,
        /// Infinity flag state
        inf: bool,
    },
    /// Jump to a subroutine
    JumpSubroutine {
        /// Where to jump
        address: Word,
        /// Where to return
        link: Word,
        /// The new SP value
        sp: Word,
        /// The new BP value
        bp: Word,
    },
    /// Jump to a location in memory
    Jump {
        /// Where to jump
        address: Word,
    },
    /// Return from a subroutine
    Return {
        /// Where to jump
        address: Word,
        /// The new BP value
        bp: Word,
        /// The new SP value
        sp: Word,
        /// The new LP value
        lp: Word,
    },
    /// Stop execution
    Halt,
    /// The last instruction is to be ignored
    Ignore {
        /// Currently-held locks
        wregs: RegisterFlags,
    },
    /// Write to the status registers
    WriteStatus {
        /// Zero flag state
        zf: bool,
        /// Overflow flag state
        of: bool,
        /// Epsilon-equality flag state
        eps: bool,
        /// NaN flag state
        nan: bool,
        /// Infinity flag state
        inf: bool,
    },
}

impl Serialize for MemoryResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        match self {
            MemoryResult::Nop => serializer.collect_map([("job", "Nop")]),
            MemoryResult::Squashed { wregs } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("job", "squashed")?;
                map.serialize_entry("wregs", wregs)?;
                map.end()
            }
            MemoryResult::WriteRegNoStatus { destination, value } => serializer.collect_map([
                ("job", "write register without status"),
                ("destination", get_name(*destination).unwrap_or("<?>")),
                ("value", value.to_string().as_str()),
            ]),
            MemoryResult::WriteReg1 {
                destination,
                value,
                zf,
                of,
                eps,
                nan,
                inf,
            } => serializer.collect_map([
                ("job", "write register"),
                ("destination", get_name(*destination).unwrap_or("<?>")),
                ("value", value.to_string().as_str()),
                ("zf", zf.to_string().as_str()),
                ("of", of.to_string().as_str()),
                ("eps", eps.to_string().as_str()),
                ("nan", nan.to_string().as_str()),
                ("inf", inf.to_string().as_str()),
            ]),
            MemoryResult::WriteReg2 {
                destination,
                value,
                sp,
                zf,
                of,
                eps,
                nan,
                inf,
            } => serializer.collect_map([
                ("job", "write register and stack pointer"),
                ("destination", get_name(*destination).unwrap_or("<?>")),
                ("sp", sp.to_string().as_str()),
                ("value", value.to_string().as_str()),
                ("zf", zf.to_string().as_str()),
                ("of", of.to_string().as_str()),
                ("eps", eps.to_string().as_str()),
                ("nan", nan.to_string().as_str()),
                ("inf", inf.to_string().as_str()),
            ]),
            MemoryResult::JumpSubroutine {
                address,
                link,
                sp,
                bp,
            } => {
                let mut map = serializer.serialize_map(Some(5))?;
                map.serialize_entry("job", "jump to a subroutine")?;
                map.serialize_entry("address", &format!("{address:#010X}"))?;
                map.serialize_entry("link", link)?;
                map.serialize_entry("sp", sp)?;
                map.serialize_entry("bp", bp)?;
                map.end()
            }
            MemoryResult::Jump { address } => {
                serializer.collect_map([("job", "jump"), ("address", &format!("{address:#010X}"))])
            }
            MemoryResult::Return {
                address,
                bp,
                sp,
                lp,
            } => serializer.collect_map([
                ("job", "return"),
                ("address", &format!("{address:#010X}")),
                ("bp", bp.to_string().as_str()),
                ("sp", sp.to_string().as_str()),
                ("lp", lp.to_string().as_str()),
            ]),
            MemoryResult::Halt => serializer.collect_map([("job", "halt")]),
            MemoryResult::Ignore { wregs } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("job", "ignore")?;
                map.serialize_entry("wregs", wregs)?;
                map.end()
            }
            MemoryResult::WriteStatus {
                zf,
                of,
                eps,
                nan,
                inf,
            } => serializer.collect_map([
                ("job", "write status registers"),
                ("zf", zf.to_string().as_str()),
                ("of", of.to_string().as_str()),
                ("eps", eps.to_string().as_str()),
                ("nan", nan.to_string().as_str()),
                ("inf", inf.to_string().as_str()),
            ]),
        }
    }
}

/// Represents the memory pipeline stage
#[derive(Debug, Default)]
pub struct Memory {
    state: State,
    forward: Option<MemoryResult>,
}

impl Serialize for Memory {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.state.serialize(serializer)
    }
}

macro_rules! stack_address {
    ($a:ident + $o:literal) => {
        libseis::pages::STACK_PAGE | ($a.wrapping_add($o) & 0xFFFF)
    };
    ($a:ident - $o:literal) => {
        libseis::pages::STACK_PAGE | ($a.wrapping_sub($o) & 0xFFFF)
    };
    ($a:ident) => {
        libseis::pages::STACK_PAGE | ($a & 0xFFFF)
    };
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
                    Pushing { value, sp, .. } => match memory.write_word(stack_address!(sp), value)
                    {
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
                    } => match memory.read_word(stack_address!(sp - 4)) {
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
                                    destination,
                                    value,
                                    sp: sp.wrapping_sub(4),
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
                                        destination,
                                        value,
                                        sp: sp.wrapping_sub(4),
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
                    DummyPop { sp, .. } => match memory.read_word(stack_address!(sp - 4)) {
                        Ok(_) => {
                            if clock.is_ready() {
                                self.state = Idle;
                                self.forward = Some(MemoryResult::WriteRegNoStatus {
                                    destination: SP,
                                    value: sp.wrapping_sub(4),
                                });
                                clock.to_ready()
                            } else {
                                self.state = Ready {
                                    result: MemoryResult::WriteRegNoStatus {
                                        destination: SP,
                                        value: sp.wrapping_sub(4),
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
                        WritingLp => match memory.write_word(stack_address!(sp), link) {
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
                        WritingBp => match memory.write_word(stack_address!(sp + 4), bp) {
                            MemStatus::Idle => {
                                if clock.is_ready() {
                                    self.state = Idle;
                                    self.forward = Some(MemoryResult::JumpSubroutine {
                                        address,
                                        link,
                                        sp: stack_address!(sp + 8),
                                        bp: stack_address!(sp + 8),
                                    });
                                    clock.to_ready()
                                } else {
                                    self.state = Ready {
                                        result: MemoryResult::JumpSubroutine {
                                            address,
                                            link,
                                            sp: stack_address!(sp + 8),
                                            bp: stack_address!(sp + 8),
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
                        ReadingBp => match memory.read_word(stack_address!(bp - 4)) {
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
                        ReadingLp(bpval) => match memory.read_word(stack_address!(bp - 8)) {
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
                                self.forward = Some(result);
                                Clock::Halt
                            } else {
                                self.state = Idle;
                                self.forward = Some(result);
                                clock.to_ready()
                            }
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
                    Halting(..) => match memory.immediate_writeback() {
                        MemStatus::Idle => {
                            if clock.is_ready() {
                                self.state = Halted;
                                self.forward = Some(MemoryResult::Halt);
                                Clock::Halt
                            } else {
                                self.state = Ready {
                                    result: MemoryResult::Halt,
                                };
                                Clock::Halt
                            }
                        }
                        MemStatus::Busy(clocks) => {
                            self.state = Halting(clocks);
                            clock.to_block()
                        }
                    },
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
            let (clocks, rix) = match input {
                Status::Stall(clocks) => (clocks, 0),
                Status::Flow(input) => match input {
                    ExecuteResult::Nop => {
                        self.state = Ready {
                            result: MemoryResult::Nop,
                        };
                        (1, 0)
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
                        (1, 0)
                    }
                    ExecuteResult::JumpTo { address } => {
                        self.state = Ready {
                            result: MemoryResult::Jump { address },
                        };
                        (1, 0)
                    }
                    ExecuteResult::Return { link, bp } => {
                        self.state = RetPrep {
                            bp,
                            link,
                            state: ReadingBp,
                            clocks: 1,
                        };
                        (1, 0)
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
                        (1, 0)
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
                        (1, 0)
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
                        (1, 0)
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
                        (1, 0)
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
                        (1, 0)
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
                        (1, 0)
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
                        (1, 0)
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
                        (1, 0)
                    }
                    ExecuteResult::ReadRegStack { register, sp } => {
                        self.state = Popping {
                            destination: register,
                            sp,
                            clocks: 1,
                        };
                        (1, 0)
                    }
                    ExecuteResult::WriteRegStack { value, sp } => {
                        self.state = Pushing {
                            value,
                            sp,
                            clocks: 1,
                        };
                        (1, 0)
                    }
                    ExecuteResult::Squash { regs } => {
                        self.state = Ready {
                            result: MemoryResult::Squashed { wregs: regs },
                        };
                        (1, 0)
                    }
                    ExecuteResult::Ignore { regs } => {
                        self.state = Ready {
                            result: MemoryResult::Ignore { wregs: regs },
                        };
                        (1, 0)
                    }
                    ExecuteResult::Halt => {
                        self.state = Halting(1);
                        (1, 0)
                    }
                    ExecuteResult::PopStack { sp } => {
                        self.state = DummyPop { sp, clocks: 1 };
                        (1, 0)
                    }
                },
                Status::Ready(r) => (1, r),
                Status::Squashed => (1, 0),
                Status::Dry => (1, 0),
            };

            match take(&mut self.forward) {
                Some(result) => Status::Flow(result),
                None if self.state.is_waiting() && rix == 3 => {
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
