//! Enumerators representing the structure of an instruction.
//!
//! Each enumerator contains data representing the instruction and
//! its operands. Instructions here are represented as a tree, where
//! the value of the previous level dictates the structure of the next.
//!
//! This also contains a set of traits defining the interfaces for encoding
//! and decoding instructions.

pub mod control;
pub mod error;
pub mod floating_point;
pub mod integer;
pub mod register;

use crate::{registers::RegisterFlags, types::Word};
pub use control::ControlOp;
use error::{DecodeError, DecodeResult};
pub use floating_point::FloatingPointOp;
pub use integer::IntegerOp;
pub use register::RegisterOp;
use std::fmt::Display;

/// Represents a decodable type.
pub trait Decode: Sized {
    /// Decodes the word
    fn decode(word: Word) -> DecodeResult<Self>;
}

/// Represents an encodable type.
pub trait Encode: Sized {
    /// Encodes to a word
    fn encode(self) -> Word;
}

/// Represents an interface for getting information about the
/// registers involved in the instruction.
pub trait Info: Sized {
    /// Gets the registers being written to
    fn get_write_regs(self) -> RegisterFlags;

    /// Gets the registers being read from
    fn get_read_regs(self) -> RegisterFlags;
}

/// Alias for [`Decode::decode`]
#[inline]
pub fn decode<D: Decode>(word: Word) -> DecodeResult<D> {
    D::decode(word)
}

/// Alias for [`Encode::encode`]
#[inline]
pub fn encode<E: Encode>(e: E) -> Word {
    e.encode()
}

/// Instruction categories
#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    /// [Control operations](ControlOp)
    Control(ControlOp),
    /// [Integer arithmetic](IntegerOp)
    Integer(IntegerOp),
    /// [Floating-point arithmetic](FloatingPointOp)
    FloatingPoint(FloatingPointOp),
    /// [Register operations](RegisterOp)
    Register(RegisterOp),
}

impl Default for Instruction {
    fn default() -> Self {
        Self::Control(ControlOp::Nop)
    }
}

impl Instruction {
    /// Mask to extract op-type bits
    const MASK: Word = 0b1110_0000_0000_0000_0000_0000_0000_0000;
    /// Amount to shift op-type bits
    const SHIFT: Word = 29;

    /// Control bits
    const CONTROL: Word = 0b000;
    /// Integer bits
    const INTEGER: Word = 0b001;
    /// Floating-point bits
    const FLOATING_POINT: Word = 0b010;
    /// Register bits
    const REGISTER: Word = 0b011;
}

impl Decode for Instruction {
    fn decode(word: Word) -> DecodeResult<Instruction> {
        use Instruction::*;
        let op_type = (word & Self::MASK) >> Self::SHIFT;

        match op_type {
            Self::CONTROL => Ok(Control(decode(word)?)),
            Self::INTEGER => Ok(Integer(decode(word)?)),
            Self::FLOATING_POINT => Ok(FloatingPoint(decode(word)?)),
            Self::REGISTER => Ok(Register(decode(word)?)),
            _ => Err(DecodeError::InvalidOpType(op_type)),
        }
    }
}

impl Encode for Instruction {
    fn encode(self) -> Word {
        use Instruction::*;

        match self {
            Control(c) => (Self::CONTROL << Self::SHIFT) | c.encode(),
            Integer(i) => (Self::INTEGER << Self::SHIFT) | i.encode(),
            FloatingPoint(f) => (Self::FLOATING_POINT << Self::SHIFT) | f.encode(),
            Register(r) => (Self::REGISTER << Self::SHIFT) | r.encode(),
        }
    }
}

impl Info for Instruction {
    fn get_write_regs(self) -> RegisterFlags {
        use Instruction::*;

        match self {
            Control(c) => c.get_write_regs(),
            Integer(i) => i.get_write_regs(),
            FloatingPoint(f) => f.get_write_regs(),
            Register(r) => r.get_write_regs(),
        }
    }

    fn get_read_regs(self) -> RegisterFlags {
        use Instruction::*;

        match self {
            Control(c) => c.get_read_regs(),
            Integer(i) => i.get_read_regs(),
            FloatingPoint(f) => f.get_read_regs(),
            Register(r) => r.get_read_regs(),
        }
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Instruction::*;

        match self {
            Control(c) => write!(f, "{c}"),
            Integer(i) => write!(f, "{i}"),
            FloatingPoint(fp) => write!(f, "{fp}"),
            Register(r) => write!(f, "{r}"),
        }
    }
}

impl TryFrom<Word> for Instruction {
    type Error = DecodeError;

    fn try_from(value: Word) -> DecodeResult<Self> {
        decode(value)
    }
}

impl From<Instruction> for Word {
    fn from(value: Instruction) -> Self {
        encode(value)
    }
}

impl serde::Serialize for Instruction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let as_string = self.to_string();
        serializer.serialize_str(&as_string)
    }
}
