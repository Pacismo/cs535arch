//! Floating-point operations
use super::{Decode, Encode, Info};
use crate::{
    instruction_set::{decode, error::DecodeError},
    registers::{RegisterFlags, EPS, INF, NAN, OF, ZF},
    types::{Register, Word},
};
use std::fmt::Display;

/// Binary floating-point operation
#[derive(Debug, Clone, Copy)]
pub struct BinaryOp {
    /// Left register
    pub left: Register,
    /// Right register
    pub right: Register,
    /// Destination register
    pub destination: Register,
}

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
        Ok(Self {
            left: ((word & Self::REG_MASK[0]) >> Self::REG_SHIFT[0]) as Register,
            right: ((word & Self::REG_MASK[1]) >> Self::REG_SHIFT[1]) as Register,
            destination: ((word & Self::REG_MASK[2]) >> Self::REG_SHIFT[2]) as Register,
        })
    }
}

impl Encode for BinaryOp {
    fn encode(self) -> Word {
        ((self.left as Word) << Self::REG_SHIFT[0])
            | ((self.right as Word) << Self::REG_SHIFT[1])
            | ((self.destination as Word) << Self::REG_SHIFT[2])
    }
}

impl Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "V{:X}, V{:X} => V{:X}",
            self.left, self.right, self.destination
        )
    }
}

/// Unary floating-point operation
#[derive(Debug, Clone, Copy)]
pub struct UnaryOp {
    /// Source register
    pub source: Register,
    /// Destination register
    pub destination: Register,
}

impl UnaryOp {
    const REG_MASK: [Word; 2] = [
        0b0000_0000_0000_0000_0000_0000_1111_0000,
        0b0000_0000_0000_0000_0000_0000_0000_1111,
    ];
    const REG_SHIFT: [Word; 2] = [4, 0];
}

impl Decode for UnaryOp {
    fn decode(word: Word) -> super::error::DecodeResult<Self> {
        Ok(Self {
            source: ((word & Self::REG_MASK[0]) >> Self::REG_SHIFT[0]) as Register,
            destination: ((word & Self::REG_MASK[1]) >> Self::REG_SHIFT[1]) as Register,
        })
    }
}

impl Encode for UnaryOp {
    fn encode(self) -> Word {
        ((self.source as Word) << Self::REG_SHIFT[0])
            | ((self.destination as Word) << Self::REG_SHIFT[1])
    }
}

impl Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "V{:X} => V{:X}", self.source, self.destination)
    }
}

/// floating-point conversion operation
#[derive(Debug, Clone, Copy)]
pub struct ConversionOp {
    /// Source register
    pub source: Register,
    /// Destination register
    pub destination: Register,
}

impl ConversionOp {
    const REG_MASK: [Word; 2] = [
        0b0000_0000_0000_0000_0000_0000_1111_0000,
        0b0000_0000_0000_0000_0000_0000_0000_1111,
    ];
    const REG_SHIFT: [Word; 2] = [4, 0];
}

impl Decode for ConversionOp {
    fn decode(word: Word) -> super::error::DecodeResult<Self> {
        Ok(Self {
            source: ((word & Self::REG_MASK[0]) >> Self::REG_SHIFT[0]) as Register,
            destination: ((word & Self::REG_MASK[1]) >> Self::REG_SHIFT[1]) as Register,
        })
    }
}

impl Encode for ConversionOp {
    fn encode(self) -> Word {
        ((self.source as Word) << Self::REG_SHIFT[0])
            | ((self.destination as Word) << Self::REG_SHIFT[1])
    }
}

impl Display for ConversionOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "V{:X} => V{:X}", self.source, self.destination)
    }
}

/// Binary floating-point comparison
#[derive(Debug, Clone, Copy)]
pub struct CompOp {
    /// Left register
    pub left: Register,
    /// Right register
    pub right: Register,
}

impl CompOp {
    const REG_MASK: [Word; 2] = [
        0b0000_0000_0000_0000_0000_0000_1111_0000,
        0b0000_0000_0000_0000_0000_1111_0000_0000,
    ];
    const REG_SHIFT: [Word; 2] = [4, 8];
}

impl Decode for CompOp {
    fn decode(word: Word) -> super::error::DecodeResult<Self> {
        Ok(Self {
            left: ((word & Self::REG_MASK[0]) >> Self::REG_SHIFT[0]) as Register,
            right: ((word & Self::REG_MASK[1]) >> Self::REG_SHIFT[1]) as Register,
        })
    }
}

impl Encode for CompOp {
    fn encode(self) -> Word {
        ((self.left as Word) << Self::REG_SHIFT[0]) | ((self.right as Word) << Self::REG_SHIFT[1])
    }
}

impl Display for CompOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "V{:X}, V{:X}", self.left, self.right)
    }
}

/// Floating-point check operation (NAN/INF)
#[derive(Debug, Clone, Copy)]
pub struct CheckOp(pub Register);

impl CheckOp {
    const REG_MASK: Word = 0b0000_0000_0000_0000_0000_0000_0000_1111;
}

impl Decode for CheckOp {
    fn decode(word: Word) -> super::error::DecodeResult<Self> {
        Ok(Self((word & Self::REG_MASK) as Register))
    }
}

impl Encode for CheckOp {
    fn encode(self) -> Word {
        self.0 as Word
    }
}

impl Display for CheckOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "V{:x}", self.0)
    }
}

/// Floating point instructions
#[derive(Debug, Clone, Copy)]
pub enum FloatingPointOp {
    /// Floating-point addition
    ///
    /// ```seis
    /// FADD Vx, Vy, Vz
    /// ```
    Fadd(BinaryOp),
    /// Floating-point subtraction
    ///
    /// ```seis
    /// FSUB Vx, Vy, Vz
    /// ```
    Fsub(BinaryOp),
    /// Floating-point multiplication
    ///
    /// ```seis
    /// FMUL Vx, Vy, Vz
    /// ```
    Fmul(BinaryOp),
    /// Floating-point division
    ///
    /// ```seis
    /// FDIV Vx, Vy, Vz
    /// ```
    Fdiv(BinaryOp),
    /// Floating-point modulo
    ///
    /// ```seis
    /// FMOD Vx, Vy, Vz
    /// ```
    Fmod(BinaryOp),
    /// Floating-point comparison
    ///
    /// ```seis
    /// FCMP Vx, Vy
    /// ```
    Fcmp(CompOp),
    /// Floating-point negation
    ///
    /// ```seis
    /// FNEG Vx, Vy
    /// ```
    Fneg(UnaryOp),
    /// Floating-point reciporacle
    ///
    /// ```seis
    /// FREC Vx, Vy
    /// ```
    Frec(UnaryOp),
    /// Integer-to-float
    ///
    /// ```seis
    /// ITOF Vx, Vy
    /// ```
    Itof(ConversionOp),
    /// Float-to-integer
    ///
    /// ```seis
    /// FTOI Vx, Vy
    /// ```
    Ftoi(ConversionOp),
    /// Floating-point check
    ///
    /// ```seis
    /// FCHK Vx
    /// ```
    Fchk(CheckOp),
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

impl Info for FloatingPointOp {
    fn get_write_regs(self) -> RegisterFlags {
        use FloatingPointOp::*;

        match self {
            Fadd(BinaryOp {
                left: _,
                right: _,
                destination: r,
            })
            | Fsub(BinaryOp {
                left: _,
                right: _,
                destination: r,
            })
            | Fmul(BinaryOp {
                left: _,
                right: _,
                destination: r,
            })
            | Fdiv(BinaryOp {
                left: _,
                right: _,
                destination: r,
            })
            | Fmod(BinaryOp {
                left: _,
                right: _,
                destination: r,
            })
            | Fneg(UnaryOp {
                source: _,
                destination: r,
            })
            | Frec(UnaryOp {
                source: _,
                destination: r,
            })
            | Itof(ConversionOp {
                source: _,
                destination: r,
            })
            | Ftoi(ConversionOp {
                source: _,
                destination: r,
            }) => [r, ZF, OF, EPS, NAN, INF].into(),

            _ => [].into(),
        }
    }

    fn get_read_regs(self) -> RegisterFlags {
        use FloatingPointOp::*;

        match self {
            Fadd(BinaryOp {
                left: r0,
                right: r1,
                destination: _,
            })
            | Fsub(BinaryOp {
                left: r0,
                right: r1,
                destination: _,
            })
            | Fmul(BinaryOp {
                left: r0,
                right: r1,
                destination: _,
            })
            | Fdiv(BinaryOp {
                left: r0,
                right: r1,
                destination: _,
            })
            | Fmod(BinaryOp {
                left: r0,
                right: r1,
                destination: _,
            })
            | Fcmp(CompOp {
                left: r0,
                right: r1,
            }) => [r0, r1].into(),

            Fneg(UnaryOp {
                source: r,
                destination: _,
            })
            | Frec(UnaryOp {
                source: r,
                destination: _,
            })
            | Itof(ConversionOp {
                source: r,
                destination: _,
            })
            | Ftoi(ConversionOp {
                source: r,
                destination: _,
            })
            | Fchk(CheckOp(r)) => [r].into(),
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
