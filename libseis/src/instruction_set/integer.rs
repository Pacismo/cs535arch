use std::fmt::Display;

use super::{error::DecodeResult, Decode, Encode};
use crate::{
    instruction_set::{decode, error::DecodeError},
    types::{Register, Word},
};

/// Binary operation (two parameters)
#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
    /// Source register, 13-bit immediate parameter, and destination register
    Immediate(Register, Word, Register),
    /// Source and parameter registers and destination register
    Registers(Register, Register, Register),
}

impl BinaryOp {
    /// Immediate value provided if set
    const IMM_FLAG_MASK: Word = 0b0000_0000_1000_0000_0000_0000_0000_0000;

    /// Source register mask
    const SRC_REG_MASK: Word = 0b0000_0000_0000_0000_0000_0000_1111_0000;
    /// Source register shift
    const SRC_REG_SHIFT: Word = 4;
    /// Destination register mask
    const DST_REG_MASK: Word = 0b0000_0000_0000_0000_0000_0000_0000_1111;
    /// Destination register shift
    const DST_REG_SHIFT: Word = 0;

    /// Immediate parameter mask
    const IMM_CONST_MASK: Word = 0b0000_0000_0111_1111_1111_1111_0000_0000;
    /// Register parameter mask
    const REG_PARAM_MASK: Word = 0b0000_0000_0000_0000_0000_1111_0000_0000;
    /// Parameter shift
    const PARAM_SHIFT: Word = 8;
}

impl Decode for BinaryOp {
    fn decode(word: Word) -> DecodeResult<Self> {
        use BinaryOp::*;
        let dest = (word & Self::DST_REG_MASK) >> Self::DST_REG_SHIFT;
        let src = (word & Self::SRC_REG_MASK) >> Self::SRC_REG_SHIFT;

        if (word & Self::IMM_FLAG_MASK) == Self::IMM_FLAG_MASK {
            let param = (word & Self::REG_PARAM_MASK) >> Self::PARAM_SHIFT;
            Ok(Immediate(src as Register, param, dest as Register))
        } else {
            let param = (word & Self::IMM_CONST_MASK) >> Self::PARAM_SHIFT;
            Ok(Registers(
                src as Register,
                param as Register,
                dest as Register,
            ))
        }
    }
}

impl Encode for BinaryOp {
    fn encode(self) -> Word {
        use BinaryOp::*;

        match self {
            Immediate(src, opt, dst) => {
                Self::IMM_CONST_MASK
                    | ((src as Word) << Self::SRC_REG_SHIFT)
                    | (opt << Self::PARAM_SHIFT)
                    | ((dst as Word) << Self::DST_REG_SHIFT)
            }
            Registers(src, opt, dst) => {
                ((src as Word) << Self::SRC_REG_SHIFT)
                    | ((opt as Word) << Self::PARAM_SHIFT)
                    | ((dst as Word) << Self::DST_REG_SHIFT)
            }
        }
    }
}

impl Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use BinaryOp::*;

        match self {
            Immediate(src, opt, dst) => write!(f, "V{src:X}, #{opt} => V{dst:X}"),
            Registers(src, opt, dst) => write!(f, "V{src:X}, V{opt:X} => V{dst:X}"),
        }
    }
}

/// Unary operation (one parameter)
#[derive(Debug, Clone, Copy)]
pub struct UnaryOp(pub Register, pub Register);

impl UnaryOp {
    /// Source register mask
    const SRC_REG_MASK: Word = 0b0000_0000_0000_0000_0000_0000_1111_0000;
    /// Source register shift
    const SRC_REG_SHIFT: Word = 4;
    /// Destination register mask
    const DST_REG_MASK: Word = 0b0000_0000_0000_0000_0000_0000_0000_1111;
    /// Destination register shift
    const DST_REG_SHIFT: Word = 0;
}

impl Decode for UnaryOp {
    fn decode(word: Word) -> DecodeResult<Self> {
        let src = (word & Self::SRC_REG_MASK) >> Self::SRC_REG_SHIFT;
        let dst = (word & Self::DST_REG_MASK) >> Self::DST_REG_SHIFT;

        Ok(Self(src as Register, dst as Register))
    }
}

impl Encode for UnaryOp {
    fn encode(self) -> Word {
        ((self.0 as Word) << Self::SRC_REG_MASK) | ((self.1 as Word) << Self::DST_REG_MASK)
    }
}

impl Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "V{:X} => V{:X}", self.0, self.1)
    }
}

/// Comparison operation (explicitly different from [`UnaryOp`])
#[derive(Debug, Clone, Copy)]
pub enum CompOp {
    Registers(Register, Register),
    Immediate(Register, Word),
}

impl CompOp {
    /// Immediate flag mask
    const IMMEDIATE_MASK: Word = 0b0000_0000_1000_0000_0000_0000_0000_0000;

    /// Right register mask
    const RIGHT_REG_MASK: Word = 0b0000_0000_0000_0000_0000_0000_1111_0000;
    /// Right immediate mask
    const RIGHT_IMM_MASK: Word = 0b0000_0000_0111_1111_1111_1111_1111_0000;
    /// Right register shift
    const RIGHT_PARAM_SHIFT: Word = 4;

    /// Left register mask
    const LEFT_REG_MASK: Word = 0b0000_0000_0000_0000_0000_0000_0000_1111;
    /// Left register shift
    const LEFT_REG_SHIFT: Word = 0;
}

impl Decode for CompOp {
    fn decode(word: Word) -> DecodeResult<Self> {
        use CompOp::*;

        if word & Self::IMMEDIATE_MASK == 0 {
            let left = (word & Self::LEFT_REG_MASK) >> Self::LEFT_REG_SHIFT;
            let right = (word & Self::RIGHT_REG_MASK) >> Self::RIGHT_PARAM_SHIFT;

            Ok(Registers(left as Register, right as Register))
        } else {
            let left = (word & Self::LEFT_REG_MASK) >> Self::LEFT_REG_SHIFT;
            let right = (word & Self::RIGHT_IMM_MASK) >> Self::RIGHT_PARAM_SHIFT;

            Ok(Immediate(left as Register, right))
        }
    }
}

impl Encode for CompOp {
    fn encode(self) -> Word {
        use CompOp::*;

        match self {
            Registers(left, right) => {
                ((left as Word) << Self::LEFT_REG_SHIFT)
                    | ((right as Word) << Self::RIGHT_PARAM_SHIFT)
            }
            Immediate(left, right) => {
                ((left as Word) << Self::LEFT_REG_MASK) | (right << Self::RIGHT_PARAM_SHIFT)
            }
        }
    }
}

impl Display for CompOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use CompOp::*;

        match self {
            Registers(left, right) => write!(f, "V{left:X}, V{right:X}"),
            Immediate(left, right) => write!(f, "V{left:X}, #{right}"),
        }
    }
}

/// Integer operations
#[derive(Debug, Clone, Copy)]
pub enum IntegerOp {
    /// Add
    Add(BinaryOp),
    /// Subtract
    Sub(BinaryOp),
    /// Multiply
    Mul(BinaryOp),
    /// Divide unsigned
    Dvu(BinaryOp),
    /// Divide signed
    Dvs(BinaryOp),
    /// Modulo
    Mod(BinaryOp),
    /// Compare
    Cmp(CompOp),
    /// Test
    Tst(BinaryOp),
    /// And
    And(BinaryOp),
    /// Inclusive or
    Ior(BinaryOp),
    /// Exclusive or
    Xor(BinaryOp),
    /// Bitwise not
    Not(UnaryOp),
    /// Sign-extend byte
    Seb(UnaryOp),
    /// Sign-extend short
    Ses(UnaryOp),
    /// Logical-shift left
    Lsl(BinaryOp),
    /// Logical-shift right
    Lsr(BinaryOp),
    /// Arithmetic shift right
    Asr(BinaryOp),
    /// Rotate left
    Rol(BinaryOp),
    /// Rotate right
    Ror(BinaryOp),
}

impl IntegerOp {
    /// Mask to extract operation code
    const MASK: Word = 0b0001_1111_0000_0000_0000_0000_0000_0000;
    /// Operation code shift
    const SHIFT: Word = 24;

    /// Add bits
    ///
    /// [`IntegerOp::Add`]
    const ADD: Word = 0b0_0000;
    /// Subtract bits
    ///
    /// [`IntegerOp::Sub`]
    const SUB: Word = 0b0_0001;
    /// Multiply bits
    ///
    /// [`IntegerOp::Mul`]
    const MUL: Word = 0b0_0010;
    /// Divide unsigned bits
    ///
    /// [`IntegerOp::Dvu`]
    const DVU: Word = 0b0_0011;
    /// Divide signed bits
    ///
    /// [`IntegerOp::Dvs`]
    const DVS: Word = 0b0_0100;
    /// Modulo bits
    ///
    /// [`IntegerOp::Mod`]
    const MOD: Word = 0b0_0101;

    /// And bits
    ///
    /// [`IntegerOp::And`]
    const AND: Word = 0b0_0110;
    /// Inclusive or bits
    ///
    /// [`IntegerOp::Ior`]
    const IOR: Word = 0b0_0111;
    /// Exclusive or bits
    ///
    /// [`IntegerOp::Xor`]
    const XOR: Word = 0b0_1000;
    /// Not bits
    ///
    /// [`IntegerOp::Not`]
    const NOT: Word = 0b0_1001;

    /// Sign-extend byte bits
    ///
    /// [`IntegerOp::Seb`]
    const SEB: Word = 0b0_1100;
    /// Sign-extend short bits
    ///
    /// [`IntegerOp::Ses`]
    const SES: Word = 0b0_1101;

    /// Logical-shift left bits
    ///
    /// [`IntegerOp::Lsl`]
    const LSL: Word = 0b1_0000;
    /// Logical-shift right bits
    ///
    /// [`IntegerOp::Lsr`]
    const LSR: Word = 0b1_0001;
    /// Arithmetic-shift right bits
    ///
    /// [`IntegerOp::Asr`]
    const ASR: Word = 0b1_0010;

    /// Rotate left bits
    ///
    /// [`IntegerOp::Rol`]
    const ROL: Word = 0b1_0100;
    /// Rotate right bits
    ///
    /// [`IntegerOp::Ror`]
    const ROR: Word = 0b1_0101;

    /// Compare bits
    ///
    /// [`IntegerOp::Cmp`]
    const CMP: Word = 0b1_1110;
    /// Test bits
    ///
    /// [`IntegerOp::Tst`]
    const TST: Word = 0b1_1111;
}

impl Decode for IntegerOp {
    fn decode(word: Word) -> DecodeResult<Self> {
        use IntegerOp::*;
        let int_op = (word & Self::MASK) >> Self::SHIFT;

        match int_op {
            Self::ADD => Ok(Add(decode(word)?)),
            Self::SUB => Ok(Sub(decode(word)?)),
            Self::MUL => Ok(Mul(decode(word)?)),
            Self::DVU => Ok(Dvu(decode(word)?)),
            Self::DVS => Ok(Dvs(decode(word)?)),
            Self::MOD => Ok(Mod(decode(word)?)),
            Self::AND => Ok(And(decode(word)?)),
            Self::IOR => Ok(Ior(decode(word)?)),
            Self::XOR => Ok(Xor(decode(word)?)),
            Self::NOT => Ok(Not(decode(word)?)),
            Self::SEB => Ok(Seb(decode(word)?)),
            Self::SES => Ok(Ses(decode(word)?)),
            Self::LSL => Ok(Lsl(decode(word)?)),
            Self::LSR => Ok(Lsr(decode(word)?)),
            Self::ASR => Ok(Asr(decode(word)?)),
            Self::ROL => Ok(Rol(decode(word)?)),
            Self::ROR => Ok(Ror(decode(word)?)),
            Self::CMP => Ok(Cmp(decode(word)?)),
            Self::TST => Ok(Tst(decode(word)?)),
            _ => Err(DecodeError::InvalidIntegerOp(int_op)),
        }
    }
}

impl Encode for IntegerOp {
    fn encode(self) -> Word {
        use IntegerOp::*;

        match self {
            Add(b) => (Self::ADD << Self::SHIFT) | b.encode(),
            Sub(b) => (Self::SUB << Self::SHIFT) | b.encode(),
            Mul(b) => (Self::MUL << Self::SHIFT) | b.encode(),
            Dvu(b) => (Self::DVU << Self::SHIFT) | b.encode(),
            Dvs(b) => (Self::DVS << Self::SHIFT) | b.encode(),
            Mod(b) => (Self::MOD << Self::SHIFT) | b.encode(),
            Cmp(b) => (Self::CMP << Self::SHIFT) | b.encode(),
            Tst(b) => (Self::TST << Self::SHIFT) | b.encode(),
            And(b) => (Self::AND << Self::SHIFT) | b.encode(),
            Ior(b) => (Self::IOR << Self::SHIFT) | b.encode(),
            Xor(b) => (Self::XOR << Self::SHIFT) | b.encode(),
            Not(u) => (Self::NOT << Self::SHIFT) | u.encode(),
            Seb(u) => (Self::SEB << Self::SHIFT) | u.encode(),
            Ses(u) => (Self::SES << Self::SHIFT) | u.encode(),
            Lsl(b) => (Self::LSL << Self::SHIFT) | b.encode(),
            Lsr(b) => (Self::LSR << Self::SHIFT) | b.encode(),
            Asr(b) => (Self::ASR << Self::SHIFT) | b.encode(),
            Rol(b) => (Self::ROL << Self::SHIFT) | b.encode(),
            Ror(b) => (Self::ROR << Self::SHIFT) | b.encode(),
        }
    }
}

impl Display for IntegerOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use IntegerOp::*;

        match self {
            Add(b) => write!(f, "ADD {b}"),
            Sub(b) => write!(f, "SUB {b}"),
            Mul(b) => write!(f, "MUL {b}"),
            Dvu(b) => write!(f, "DVU {b}"),
            Dvs(b) => write!(f, "DVS {b}"),
            Mod(b) => write!(f, "MOD {b}"),
            Cmp(b) => write!(f, "CMP {b}"),
            Tst(b) => write!(f, "TST {b}"),
            And(b) => write!(f, "AND {b}"),
            Ior(b) => write!(f, "IOR {b}"),
            Xor(b) => write!(f, "XOR {b}"),
            Not(o) => write!(f, "NOT {o}"),
            Seb(o) => write!(f, "SEB {o}"),
            Ses(o) => write!(f, "SES {o}"),
            Lsl(b) => write!(f, "LSL {b}"),
            Lsr(b) => write!(f, "LSR {b}"),
            Asr(b) => write!(f, "ASR {b}"),
            Rol(b) => write!(f, "ROL {b}"),
            Ror(b) => write!(f, "ROR {b}"),
        }
    }
}
