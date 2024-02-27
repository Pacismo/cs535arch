use std::fmt::Display;

use crate::{
    instruction_set::{decode, error::DecodeError},
    types::{Register, Word},
};

use super::{Decode, Encode};

#[derive(Debug, Clone, Copy)]
pub struct BinaryOp(Register, Register, Register);

impl BinaryOp {
    /// Masks for the source, option, and destination registers
    const REG_MASK: [Word; 3] = [
        0b0000_0000_0000_0000_0000_1111_0000_0000,
        0b0000_0000_0000_0000_0000_0000_1111_0000,
        0b0000_0000_0000_0000_0000_0000_0000_1111,
    ];
    /// Shifts for the source, option, and destination registers
    const REG_SHIFT: [Word; 3] = [8, 4, 0];
}

impl Decode for BinaryOp {
    fn decode(word: Word) -> super::error::DecodeResult<Self> {
        Ok(Self(
            (word & Self::REG_MASK[0]) >> Self::REG_SHIFT[0],
            (word & Self::REG_MASK[1]) >> Self::REG_SHIFT[1],
            (word & Self::REG_MASK[2]) >> Self::REG_SHIFT[2],
        ))
    }
}

impl Encode for BinaryOp {
    fn encode(self) -> Word {
        (self.0 << Self::REG_SHIFT[0])
            | (self.1 << Self::REG_SHIFT[1])
            | (self.2 << Self::REG_SHIFT[2])
    }
}

impl Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "V{:X}, V{:X} => V{:X}", self.0, self.1, self.2)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UnaryOp(Register, Register);

impl UnaryOp {
    const REG_MASK: [Word; 2] = [
        0b0000_0000_0000_0000_0000_0000_1111_0000,
        0b0000_0000_0000_0000_0000_0000_0000_1111,
    ];
    const REG_SHIFT: [Word; 2] = [4, 0];
}

impl Decode for UnaryOp {
    fn decode(word: Word) -> super::error::DecodeResult<Self> {
        Ok(Self(
            (word & Self::REG_MASK[0]) >> Self::REG_SHIFT[0],
            (word & Self::REG_MASK[1]) >> Self::REG_SHIFT[1],
        ))
    }
}

impl Encode for UnaryOp {
    fn encode(self) -> Word {
        (self.0 << Self::REG_SHIFT[0]) | (self.1 << Self::REG_SHIFT[1])
    }
}

impl Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "V{:X} => V{:X}", self.0, self.1)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CompOp(Register, Register);

impl CompOp {
    const REG_MASK: [Word; 2] = [
        0b0000_0000_0000_0000_0000_0000_1111_0000,
        0b0000_0000_0000_0000_0000_0000_0000_1111,
    ];
    const REG_SHIFT: [Word; 2] = [4, 0];
}

impl Decode for CompOp {
    fn decode(word: Word) -> super::error::DecodeResult<Self> {
        Ok(Self(
            (word & Self::REG_MASK[0]) >> Self::REG_SHIFT[0],
            (word & Self::REG_MASK[1]) >> Self::REG_SHIFT[1],
        ))
    }
}

impl Encode for CompOp {
    fn encode(self) -> Word {
        (self.0 << Self::REG_SHIFT[0]) | (self.1 << Self::REG_SHIFT[1])
    }
}

impl Display for CompOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "V{:X}, V{:X}", self.0, self.1)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FloatingPointOp {
    /// Floating-point addition
    Fadd(BinaryOp),
    /// Floating-point subtraction
    Fsub(BinaryOp),
    /// Floating-point multiplication
    Fmul(BinaryOp),
    /// Floating-point division
    Fdiv(BinaryOp),
    /// Floating-point modulo
    Fmod(BinaryOp),
    /// Floating-point comparison
    Fcmp(BinaryOp),
    /// Floating-point negation
    Fneg(UnaryOp),
    /// Floating-point reciporacle
    Frec(UnaryOp),
    /// Integer-to-float
    Itof(UnaryOp),
    /// Float-to-integer
    Ftoi(UnaryOp),
    /// Floating-point check
    Fchk(BinaryOp),
}

impl FloatingPointOp {
    const MASK: Word = 0b0001_1111_0000_0000_0000_0000_0000_0000;
    const SHIFT: Word = 24;

    /// [`FloatingPointOp::Fadd`]
    const FADD: Word = 0b0_0000;
    /// [`FloatingPointOp::Fsub`]
    const FSUB: Word = 0b0_0001;
    /// [`FloatingPointOp::Fmul`]
    const FMUL: Word = 0b0_0010;
    /// [`FloatingPointOp::Fdiv`]
    const FDIV: Word = 0b0_0011;
    /// [`FloatingPointOp::Fmod`]
    const FMOD: Word = 0b0_0100;
    /// [`FloatingPointOp::Fcmp`]
    const FCMP: Word = 0b0_0101;
    /// [`FloatingPointOp::Fneg`]
    const FNEG: Word = 0b0_0110;
    /// [`FloatingPointOp::Frec`]
    const FREC: Word = 0b0_0111;
    /// [`FloatingPointOp::Itof`]
    const ITOF: Word = 0b0_1000;
    /// [`FloatingPointOp::Ftoi`]
    const FTOI: Word = 0b0_1001;
    /// [`FloatingPointOp::Fchk`]
    const FCHK: Word = 0b0_1010;
}

impl Decode for FloatingPointOp {
    fn decode(word: Word) -> super::error::DecodeResult<Self> {
        use FloatingPointOp::*;
        let fp_op = (word & Self::MASK) >> Self::SHIFT;

        match fp_op {
            Self::FADD => Ok(Fadd(decode(word)?)),
            Self::FSUB => Ok(Fsub(decode(word)?)),
            Self::FMUL => Ok(Fmul(decode(word)?)),
            Self::FDIV => Ok(Fdiv(decode(word)?)),
            Self::FMOD => Ok(Fmod(decode(word)?)),
            Self::FCMP => Ok(Fcmp(decode(word)?)),
            Self::FNEG => Ok(Fneg(decode(word)?)),
            Self::FREC => Ok(Frec(decode(word)?)),
            Self::ITOF => Ok(Itof(decode(word)?)),
            Self::FTOI => Ok(Ftoi(decode(word)?)),
            Self::FCHK => Ok(Fchk(decode(word)?)),
            _ => Err(DecodeError::InvalidFloatingPointOp(word)),
        }
    }
}

impl Encode for FloatingPointOp {
    fn encode(self) -> Word {
        use FloatingPointOp::*;

        match self {
            Fadd(b) => (Self::FADD << Self::SHIFT) | b.encode(),
            Fsub(b) => (Self::FSUB << Self::SHIFT) | b.encode(),
            Fmul(b) => (Self::FMUL << Self::SHIFT) | b.encode(),
            Fdiv(b) => (Self::FDIV << Self::SHIFT) | b.encode(),
            Fmod(b) => (Self::FMOD << Self::SHIFT) | b.encode(),
            Fcmp(b) => (Self::FCMP << Self::SHIFT) | b.encode(),
            Fneg(u) => (Self::FNEG << Self::SHIFT) | u.encode(),
            Frec(u) => (Self::FREC << Self::SHIFT) | u.encode(),
            Itof(u) => (Self::ITOF << Self::SHIFT) | u.encode(),
            Ftoi(u) => (Self::FTOI << Self::SHIFT) | u.encode(),
            Fchk(b) => (Self::FCHK << Self::SHIFT) | b.encode(),
        }
    }
}

impl Display for FloatingPointOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use FloatingPointOp::*;

        match self {
            Fadd(b) => write!(f, "FADD {b}"),
            Fsub(b) => write!(f, "FSUB {b}"),
            Fmul(b) => write!(f, "FMUL {b}"),
            Fdiv(b) => write!(f, "FDIV {b}"),
            Fmod(b) => write!(f, "FMOD {b}"),
            Fcmp(b) => write!(f, "FCMP {b}"),
            Fneg(u) => write!(f, "FNEG {u}"),
            Frec(u) => write!(f, "FREC {u}"),
            Itof(u) => write!(f, "ITOF {u}"),
            Ftoi(u) => write!(f, "FTOI {u}"),
            Fchk(b) => write!(f, "FCHK {b}"),
        }
    }
}
