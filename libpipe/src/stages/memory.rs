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
        register: Register,
        volatile: bool,
    },
    /// Reading a short from memory
    ReadShort {
        address: Word,
        register: Register,
        volatile: bool,
    },
    /// Reading a word from memory
    ReadWord {
        address: Word,
        register: Register,
        volatile: bool,
    },
}

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
        sp: Word,
        bp: Word,
        pc: Word,
        lr: Word,
        clocks: usize,
    },
    RetPrep {
        bp: Word,
        pc: Word,
        lr: Word,
        clocks: Word,
    },
    Ready {
        result: MemoryResult,
    },
    Squashed {
        wregs: RegisterFlags,
    },
}

#[derive(Debug, Clone, Serialize)]
pub enum MemoryResult {
    /// Nothing
    Nop,
    /// Squashed instruction
    Squashed { wregs: RegisterFlags },
    /// Write data back to a register
    WriteReg1 {
        register: Register,
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
    Jump { address: Word },
    /// Return from a subroutine
    ///
    /// Write the current BP value to the SP, read the BP back in
    Return { address: Word, bp: Word, sp: Word },
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
        _: &mut dyn MemoryModule,
    ) -> Clock {
        todo!()
    }

    fn forward(&mut self, _: Status<Self::Prev>) -> Status<Self::Next> {
        todo!()
    }
}
