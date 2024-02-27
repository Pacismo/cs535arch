use std::fmt::Display;

use super::{
    error::{DecodeError, DecodeResult},
    Decode, Encode,
};
use crate::{
    instruction_set::decode,
    types::{Register, Word},
};

/// Jump operands
#[derive(Debug, Clone, Copy)]
pub enum Jump {
    /// Jumps to a location pointed to by a register
    Register(Register),
    /// Jumps to a location pointed to by PC + IMM
    Forward(Word),
    /// Jumps to a location pointed to by PC - IMM
    Reverse(Word),
}

impl Jump {
    /// Mask to extract Jump-mode bits
    pub const MASK: Word = 0b0000_0001_1000_0000_0000_0000_0000_0000;
    /// Amount to shift Jump-mode bits
    pub const SHIFT: Word = 23;
    /// Mask to extract relative address bits
    pub const RELATIVE_MASK: Word = 0b0000_0000_0111_1111_1111_1111_1111_1111;
    /// Mask to extract register bits
    pub const REGISTER_MASK: Word = 0b0000_0000_0111_1000_0000_0000_0000_0000;
    /// Amount to shift register bits
    pub const REGISTER_SHIFT: Word = 19;

    /// Register-mode bits
    pub const REGISTER: Word = 0b00;
    /// Forward-mode bits
    pub const FORWARD: Word = 0b10;
    /// Reverse-mode bits
    pub const REVERSE: Word = 0b11;
}

impl Decode for Jump {
    fn decode(word: Word) -> DecodeResult<Self> {
        let mode = (word & Self::MASK) >> Self::SHIFT;

        match mode {
            Self::REGISTER => Ok(Self::Register(
                (word & Self::REGISTER_MASK) >> Self::REGISTER_SHIFT,
            )),
            Self::FORWARD => Ok(Self::Forward(word & Self::RELATIVE_MASK)),
            Self::REVERSE => Ok(Self::Reverse(word & Self::RELATIVE_MASK)),
            _ => Err(DecodeError::InvalidJumpType(mode)),
        }
    }
}

impl Encode for Jump {
    fn encode(self) -> Word {
        use Jump::*;
        match self {
            Register(reg) => (Self::REGISTER << Self::SHIFT) | reg << Self::REGISTER_SHIFT,
            Forward(off) => (Self::FORWARD << Self::SHIFT) | off,
            Reverse(off) => (Self::REVERSE << Self::SHIFT) | off,
        }
    }
}

impl Display for Jump {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Jump::*;

        match self {
            Register(reg) => write!(f, "V{reg:X}"),
            Forward(off) => write!(f, "+{off}"),
            Reverse(off) => write!(f, "-{off}"),
        }
    }
}

/// Flow control operations
#[derive(Debug, Clone, Copy)]
pub enum ControlOp {
    /// Stops the processor
    Halt,
    /// Does nothing
    Nop,
    /// Sets the PC to the link register
    Ret,
    /// Unconditional jump
    Jmp(Jump),
    /// Jump to subroutine
    Jsr(Jump),
    /// Jump if equal (ZF = 1)
    Jeq(Jump),
    /// Jump if not equal (ZF = 0)
    Jne(Jump),
    /// Jump if greater than (ZF = 0 & OF = 1)
    Jgt(Jump),
    /// Jump if less than (ZF = 0 & OF = 0)
    Jlt(Jump),
    /// Jump if greater than or equal to (OF = 1 | ZF = 1)
    Jge(Jump),
    /// Jump if less than or equal to (OF = 0 | ZF = 1)
    Jle(Jump),
}

impl ControlOp {
    /// Mask to extract the control-op bits
    pub const MASK: Word = 0b0001_1110_0000_0000_0000_0000_0000_0000;
    /// Amount to shift the control-op bits
    pub const SHIFT: Word = 25;

    /// HALT bits
    ///
    /// [`ControlOp::Halt`]
    pub const HALT: Word = 0b0000;
    /// NOP bits
    ///
    /// [`ControlOp::Nop`]
    pub const NOP: Word = 0b0001;
    /// JMP bits
    ///
    /// [`ControlOp::Jmp`]
    pub const JMP: Word = 0b0010;
    /// JSR bits
    ///
    /// [`ControlOp::Jsr`]
    pub const JSR: Word = 0b0011;
    /// RET bits
    ///
    /// [`ControlOp::Ret`]
    pub const RET: Word = 0b0100;

    /// JEQ bits
    ///
    /// [`ControlOp::Jeq`]
    pub const JEQ: Word = 0b1000;
    /// JNE bits
    ///
    /// [`ControlOp::Jne`]
    pub const JNE: Word = 0b1100;
    /// JGT bits
    ///
    /// [`ControlOp::Jgt`]
    pub const JGT: Word = 0b1101;
    /// JLT bits
    ///
    /// [`ControlOp::Jlt`]
    pub const JLT: Word = 0b1110;
    /// JGE bits
    ///
    /// [`ControlOp::Jge`]
    pub const JGE: Word = 0b1001;
    /// JLE bits
    ///
    /// [`ControlOp::Jle`]
    pub const JLE: Word = 0b1010;
}

impl Decode for ControlOp {
    fn decode(word: Word) -> DecodeResult<Self> {
        use ControlOp::*;
        let op = (word & Self::MASK) >> Self::SHIFT;

        match op {
            Self::HALT => Ok(Halt),
            Self::NOP => Ok(Nop),
            Self::RET => Ok(Ret),
            Self::JMP => Ok(Jmp(decode(word)?)),
            Self::JSR => Ok(Jsr(decode(word)?)),
            Self::JEQ => Ok(Jeq(decode(word)?)),
            Self::JNE => Ok(Jne(decode(word)?)),
            Self::JGT => Ok(Jgt(decode(word)?)),
            Self::JLT => Ok(Jlt(decode(word)?)),
            Self::JGE => Ok(Jge(decode(word)?)),
            Self::JLE => Ok(Jle(decode(word)?)),
            _ => Err(DecodeError::InvalidControlOp(op)),
        }
    }
}

impl Encode for ControlOp {
    fn encode(self) -> Word {
        use ControlOp::*;

        match self {
            Halt => Self::HALT << Self::SHIFT,
            Nop => Self::NOP << Self::SHIFT,
            Ret => Self::RET << Self::SHIFT,
            Jmp(jump) => (Self::JMP << Self::SHIFT) | jump.encode(),
            Jsr(jump) => (Self::JSR << Self::SHIFT) | jump.encode(),
            Jeq(jump) => (Self::JEQ << Self::SHIFT) | jump.encode(),
            Jne(jump) => (Self::JNE << Self::SHIFT) | jump.encode(),
            Jgt(jump) => (Self::JGT << Self::SHIFT) | jump.encode(),
            Jlt(jump) => (Self::JLT << Self::SHIFT) | jump.encode(),
            Jge(jump) => (Self::JGE << Self::SHIFT) | jump.encode(),
            Jle(jump) => (Self::JLE << Self::SHIFT) | jump.encode(),
        }
    }
}

impl Display for ControlOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ControlOp::*;

        match self {
            Halt => write!(f, "HALT"),
            Nop => write!(f, "NOP"),
            Ret => write!(f, "RET"),
            Jmp(jump) => write!(f, "JMP {jump}"),
            Jsr(jump) => write!(f, "JSR {jump}"),
            Jeq(jump) => write!(f, "JEQ {jump}"),
            Jne(jump) => write!(f, "JNE {jump}"),
            Jgt(jump) => write!(f, "JGT {jump}"),
            Jlt(jump) => write!(f, "JLT {jump}"),
            Jge(jump) => write!(f, "JGE {jump}"),
            Jle(jump) => write!(f, "JLE {jump}"),
        }
    }
}
