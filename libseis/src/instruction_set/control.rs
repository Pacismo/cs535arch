//! Control instructions

use super::{
    error::{DecodeError, DecodeResult},
    Decode, Encode, Info,
};
use crate::{
    instruction_set::decode,
    registers::RegisterFlags,
    types::{self, Register, SWord, Word},
};
use std::fmt::Display;

/// Jump operands
#[derive(Debug, Clone, Copy)]
pub enum Jump {
    /// Jumps to a location pointed to by a register
    Register(Register),
    /// Jumps to a location pointed to by PC + IMM (sign extended)
    Relative(SWord),
}

impl Jump {
    /// Mask to extract Jump-mode bits
    const REL_MODE_MASK: Word = 0b0000_0001_0000_0000_0000_0000_0000_0000;
    /// Mask to extract relative address bits
    const RELATIVE_MASK: Word = 0b0000_0000_1111_1111_1111_1111_1111_1111;
    /// Sign bit (signify negative)
    const SIGN_BIT: Word = 0b0000_0000_1000_0000_0000_0000_0000_0000;
    /// Mask to extract register bits
    const REGISTER_MASK: Word = 0b0000_0000_0000_0000_0000_0000_1111_0000;
    /// Amount to shift register bits
    const REGISTER_SHIFT: Word = 4;
}

impl Decode for Jump {
    fn decode(word: Word) -> DecodeResult<Self> {
        use Jump::*;

        if word & Self::REL_MODE_MASK == 0 {
            Ok(Register(
                ((word & Self::REGISTER_MASK) >> Self::REGISTER_SHIFT) as types::Register,
            ))
        } else {
            let amount = word & Self::RELATIVE_MASK;
            // Sign-extend if sign bit is 1
            if amount & Self::SIGN_BIT == 0 {
                Ok(Relative((amount as SWord) << 2))
            } else {
                Ok(Relative(((!Self::RELATIVE_MASK | amount) as SWord) << 2))
            }
        }
    }
}

impl Encode for Jump {
    fn encode(self) -> Word {
        use Jump::*;
        match self {
            Register(reg) => (reg as Word) << Self::REGISTER_SHIFT,
            Relative(off) => Self::REL_MODE_MASK | (off >> 2) as Word & Self::RELATIVE_MASK,
        }
    }
}

impl Display for Jump {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Jump::*;

        match self {
            Register(reg) => write!(f, "V{reg:X}"),
            Relative(off) => write!(f, "{:+}", off),
        }
    }
}

/// Flow control operations
///
/// All relative jumps are in terms of words rather than bytes
///
/// Absolute addresses ignore the least significant 2 bits
#[derive(Debug, Clone, Copy)]
pub enum ControlOp {
    /// Does nothing
    ///
    /// ```seis
    /// NOP ; No operation
    /// ```
    Nop,
    /// Stops the processor
    ///
    /// ```seis
    /// HALT ; Stop the processor
    /// ```
    Halt,
    /// Returns from a subroutine
    ///
    /// Will restore old BP value from the stack
    /// and use LP to jump back to the caller's
    /// next instruction
    ///
    /// ```seis
    /// RET ; Return from subroutine (jump to LP)
    /// ```
    Ret,
    /// Unconditional jump
    ///
    /// ```seis
    /// JMP Vx     ; Absolute jump
    /// JMP N      ; Relative jump
    /// JMP label ; Expands to N
    /// ```
    Jmp(Jump),
    /// Jump to subroutine
    ///
    /// Will store the old BP address and set BP to
    /// the new base, SP will point ahead of new BP
    ///
    /// Will store the return address to the LP
    ///
    /// ```seis
    /// JSR Vx     ; Jump to subroutine (absolute)
    /// JSR N      ; Jump to subroutine (relative)
    /// JSR label ; Expands to Vx with load or N
    /// ```
    Jsr(Jump),
    /// Jump if equal (ZF = 1)
    ///
    /// ```seis
    /// JEQ Vx
    /// JEQ N
    /// JEQ label ; Expands to N
    /// ```
    Jeq(Jump),
    /// Jump if not equal (ZF = 0)
    ///
    /// ```seis
    /// JNE Vx
    /// JNE N
    /// JNE label ; Expands to N
    /// ```
    Jne(Jump),
    /// Jump if greater than (ZF = 0 & OF = 1)
    ///
    /// ```seis
    /// JGT Vx
    /// JGT N
    /// JGT label ; Expands to N
    /// ```
    Jgt(Jump),
    /// Jump if less than (ZF = 0 & OF = 0)
    ///
    /// ```seis
    /// JLT Vx
    /// JLT N
    /// JLT label ; Expands to N
    /// ```
    Jlt(Jump),
    /// Jump if greater than or equal to (OF = 1 | ZF = 1)
    ///
    /// ```seis
    /// JGE Vx
    /// JGE N
    /// JGE label ; Expands to N
    /// ```
    Jge(Jump),
    /// Jump if less than or equal to (OF = 0 | ZF = 1)
    ///
    /// ```seis
    /// JLE Vx
    /// JLE N
    /// JLE label ; Expands to N
    /// ```
    Jle(Jump),
}

impl ControlOp {
    /// Mask to extract the control-op bits
    const MASK: Word = 0b0001_1110_0000_0000_0000_0000_0000_0000;
    /// Amount to shift the control-op bits
    const SHIFT: Word = 25;

    /// NOP bits
    ///
    /// [`ControlOp::Nop`]
    const NOP: Word = 0b0000;
    /// HALT bits
    ///
    /// [`ControlOp::Halt`]
    const HALT: Word = 0b0001;
    /// JMP bits
    ///
    /// [`ControlOp::Jmp`]
    const JMP: Word = 0b0010;
    /// JSR bits
    ///
    /// [`ControlOp::Jsr`]
    const JSR: Word = 0b0011;
    /// RET bits
    ///
    /// [`ControlOp::Ret`]
    const RET: Word = 0b0100;

    /// JEQ bits
    ///
    /// [`ControlOp::Jeq`]
    const JEQ: Word = 0b1000;
    /// JNE bits
    ///
    /// [`ControlOp::Jne`]
    const JNE: Word = 0b1100;
    /// JGT bits
    ///
    /// [`ControlOp::Jgt`]
    const JGT: Word = 0b1101;
    /// JLT bits
    ///
    /// [`ControlOp::Jlt`]
    const JLT: Word = 0b1110;
    /// JGE bits
    ///
    /// [`ControlOp::Jge`]
    const JGE: Word = 0b1001;
    /// JLE bits
    ///
    /// [`ControlOp::Jle`]
    const JLE: Word = 0b1010;
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

impl Info for ControlOp {
    fn get_write_regs(self) -> RegisterFlags {
        use crate::registers::{BP, LP, PC, SP};
        use ControlOp::*;

        match self {
            Ret => [PC, LP, SP, BP].into(),
            Jmp(_) | Jeq(_) | Jne(_) | Jgt(_) | Jlt(_) | Jge(_) | Jle(_) => [PC].into(),
            Jsr(_) => [PC, LP, SP, BP].into(),

            _ => [].into(),
        }
    }

    fn get_read_regs(self) -> RegisterFlags {
        use crate::registers::{BP, LP, OF, PC, SP, ZF};
        use ControlOp::*;
        use Jump::*;

        match self {
            Jmp(Register(r)) => [r, PC].into(),
            Jmp(Relative(_)) => [PC].into(),

            Jeq(Register(r)) => [r, PC, ZF].into(),
            Jeq(Relative(_)) => [PC, ZF].into(),

            Jne(Register(r)) => [r, PC, ZF].into(),
            Jne(Relative(_)) => [PC, ZF].into(),

            Jgt(Register(r)) => [r, PC, ZF, OF].into(),
            Jgt(Relative(_)) => [PC, ZF, OF].into(),

            Jlt(Register(r)) => [r, PC, ZF, OF].into(),
            Jlt(Relative(_)) => [PC, ZF, OF].into(),

            Jge(Register(r)) => [r, PC, ZF, OF].into(),
            Jge(Relative(_)) => [PC, ZF, OF].into(),

            Jle(Register(r)) => [r, PC, ZF, OF].into(),
            Jle(Relative(_)) => [PC, ZF, OF].into(),

            Jsr(Register(r)) => [r, PC, LP, SP, BP].into(),
            Jsr(Relative(_)) => [PC, LP, SP, BP].into(),
            Ret => [LP, BP].into(),

            _ => [].into(),
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
