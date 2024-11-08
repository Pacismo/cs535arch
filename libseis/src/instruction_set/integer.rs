//! Integer operations
use super::{error::DecodeResult, Decode, Encode, Info};
use crate::{
    instruction_set::{decode, error::DecodeError},
    registers::{RegisterFlags, EPS, INF, NAN, OF, ZF},
    types::{Register, Word},
};
use std::fmt::Display;

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
            let param = (word & Self::IMM_CONST_MASK) >> Self::PARAM_SHIFT;
            Ok(Immediate(src as Register, param, dest as Register))
        } else {
            let param = (word & Self::REG_PARAM_MASK) >> Self::PARAM_SHIFT;
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
                Self::IMM_FLAG_MASK
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
            Immediate(src, opt, dst) => write!(f, "V{src:X}, {opt} => V{dst:X}"),
            Registers(src, opt, dst) => write!(f, "V{src:X}, V{opt:X} => V{dst:X}"),
        }
    }
}

/// Sign extension operation
#[derive(Debug, Clone, Copy)]
pub struct SignExtendOp(pub Word, pub Register);

impl SignExtendOp {
    const TGT_REG_MASK: Word = 0b0000_0000_0000_0000_0000_0000_0000_1111;
    const SXT_BIT_MASK: Word = 0b0000_0000_0000_0000_0000_0000_0011_0000;
    const SXT_BIT_SHIFT: Word = 4;
}

impl Decode for SignExtendOp {
    fn decode(word: Word) -> DecodeResult<Self> {
        Ok(Self(
            (word & Self::SXT_BIT_MASK) >> Self::SXT_BIT_SHIFT,
            (word & Self::TGT_REG_MASK) as Register,
        ))
    }
}

impl Encode for SignExtendOp {
    fn encode(self) -> Word {
        (self.0 << Self::SXT_BIT_SHIFT) | (self.1 as Word)
    }
}

impl Display for SignExtendOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let width = if self.0 == 0 {
            "byte"
        } else if self.0 == 1 {
            "short"
        } else if self.0 == 2 {
            "word"
        } else {
            "<INVALID>"
        };

        write!(f, "{} V{:X}", width, self.1)
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
        ((self.0 as Word) << Self::SRC_REG_SHIFT) | ((self.1 as Word) << Self::DST_REG_MASK)
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
    /// Compare two registers
    Registers(Register, Register, bool),
    /// Compare a register value to an immediate
    Immediate(Register, Word, bool),
}

impl CompOp {
    /// Immediate flag mask
    const IMMEDIATE_MASK: Word = 0b0000_0000_1000_0000_0000_0000_0000_0000;

    /// Right register mask
    const RIGHT_REG_MASK: Word = 0b0000_0000_0000_0000_0000_1111_0000_0000;
    /// Right immediate mask
    const RIGHT_IMM_MASK: Word = 0b0000_0000_0111_1111_1111_1111_0000_0000;
    const RIGHT_IMM_SIGN: Word = 0b0000_0000_0100_0000_0000_0000_0000_0000;
    /// Right register shift
    const RIGHT_PARAM_SHIFT: Word = 8;

    /// Left register mask
    const LEFT_REG_MASK: Word = 0b0000_0000_0000_0000_0000_0000_1111_0000;
    /// Left register shift
    const LEFT_REG_SHIFT: Word = 4;
    /// Signed mode bit
    const SIGNED_MODE: Word = 0b0000_0000_0000_0000_0000_0000_0000_1000;
}

impl Decode for CompOp {
    fn decode(word: Word) -> DecodeResult<Self> {
        use CompOp::*;

        if word & Self::IMMEDIATE_MASK == 0 {
            let left = (word & Self::LEFT_REG_MASK) >> Self::LEFT_REG_SHIFT;
            let right = (word & Self::RIGHT_REG_MASK) >> Self::RIGHT_PARAM_SHIFT;

            Ok(Registers(
                left as Register,
                right as Register,
                word & Self::SIGNED_MODE != 0,
            ))
        } else {
            let left = (word & Self::LEFT_REG_MASK) >> Self::LEFT_REG_SHIFT;
            let mut right = (word & Self::RIGHT_IMM_MASK) >> Self::RIGHT_PARAM_SHIFT;
            if word & Self::RIGHT_IMM_SIGN != 0 {
                right |= 0b1111_1111_1000_0000_0000_0000_0000_0000;
            }

            Ok(Immediate(
                left as Register,
                right,
                word & Self::SIGNED_MODE != 0,
            ))
        }
    }
}

impl Encode for CompOp {
    fn encode(self) -> Word {
        use CompOp::*;

        match self {
            Registers(left, right, signed) => {
                ((left as Word) << Self::LEFT_REG_SHIFT)
                    | ((right as Word) << Self::RIGHT_PARAM_SHIFT)
                    | if signed { Self::SIGNED_MODE } else { 0 }
            }
            Immediate(left, right, signed) => {
                ((left as Word) << Self::LEFT_REG_SHIFT)
                    | ((right << Self::RIGHT_PARAM_SHIFT) & Self::RIGHT_IMM_MASK)
                    | if signed { Self::SIGNED_MODE } else { 0 }
                    | Self::IMMEDIATE_MASK
            }
        }
    }
}

impl Display for CompOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use CompOp::*;

        match self {
            &Registers(left, right, signed) => {
                write!(f, "{}V{left:X}, V{right:X}", if signed { "s " } else { "" })
            }
            &Immediate(left, right, signed) => {
                write!(f, "{}V{left:X}, {right}", if signed { "s " } else { "" })
            }
        }
    }
}

/// Comparison operation (explicitly different from [`UnaryOp`])
#[derive(Debug, Clone, Copy)]
pub enum TestOp {
    /// Test two registers
    Registers(Register, Register),
    /// Test a register against an immediate
    Immediate(Register, Word),
}

impl TestOp {
    /// Immediate flag mask
    const IMMEDIATE_MASK: Word = 0b0000_0000_1000_0000_0000_0000_0000_0000;

    /// Right register mask
    const RIGHT_REG_MASK: Word = 0b0000_0000_0000_0000_0000_1111_0000_0000;
    /// Right immediate mask
    const RIGHT_IMM_MASK: Word = 0b0000_0000_0111_1111_1111_1111_0000_0000;
    const RIGHT_IMM_SIGN: Word = 0b0000_0000_0100_0000_0000_0000_0000_0000;
    /// Right register shift
    const RIGHT_PARAM_SHIFT: Word = 8;

    /// Left register mask
    const LEFT_REG_MASK: Word = 0b0000_0000_0000_0000_0000_0000_1111_0000;
    /// Left register shift
    const LEFT_REG_SHIFT: Word = 4;
}

impl Decode for TestOp {
    fn decode(word: Word) -> DecodeResult<Self> {
        use TestOp::*;

        if word & Self::IMMEDIATE_MASK == 0 {
            let left = (word & Self::LEFT_REG_MASK) >> Self::LEFT_REG_SHIFT;
            let right = (word & Self::RIGHT_REG_MASK) >> Self::RIGHT_PARAM_SHIFT;

            Ok(Registers(left as Register, right as Register))
        } else {
            let left = (word & Self::LEFT_REG_MASK) >> Self::LEFT_REG_SHIFT;
            let mut right = (word & Self::RIGHT_IMM_MASK) >> Self::RIGHT_PARAM_SHIFT;
            if word & Self::RIGHT_IMM_SIGN != 0 {
                right |= 0b1111_1111_1000_0000_0000_0000_0000_0000;
            }

            Ok(Immediate(left as Register, right))
        }
    }
}

impl Encode for TestOp {
    fn encode(self) -> Word {
        use TestOp::*;

        match self {
            Registers(left, right) => {
                ((left as Word) << Self::LEFT_REG_SHIFT)
                    | ((right as Word) << Self::RIGHT_PARAM_SHIFT)
            }
            Immediate(left, right) => {
                ((left as Word) << Self::LEFT_REG_SHIFT)
                    | ((right << Self::RIGHT_PARAM_SHIFT) & Self::RIGHT_IMM_MASK)
                    | Self::IMMEDIATE_MASK
            }
        }
    }
}

impl Display for TestOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use TestOp::*;

        match self {
            Registers(left, right) => write!(f, "V{left:X}, V{right:X}"),
            Immediate(left, right) => write!(f, "V{left:X}, {right}"),
        }
    }
}

/// Integer operations
#[derive(Debug, Clone, Copy)]
pub enum IntegerOp {
    /// Add
    ///
    /// ```seis
    /// ADD Vx, Vy, Vz
    /// ```
    Add(BinaryOp),
    /// Subtract
    ///
    /// ```seis
    /// SUB Vx, Vy, Vz
    /// ```
    Sub(BinaryOp),
    /// Multiply
    ///
    /// ```seis
    /// MUL Vx, Vy, Vz
    /// ```
    Mul(BinaryOp),
    /// Divide unsigned
    ///
    /// ```seis
    /// DVU Vx, Vy, Vz
    /// ```
    Dvu(BinaryOp),
    /// Divide signed
    ///
    /// ```seis
    /// DVS Vx, Vy, Vz
    /// ```
    Dvs(BinaryOp),
    /// Modulo
    ///
    /// ```seis
    /// MOD Vx, Vy, Vz
    /// ```
    Mod(BinaryOp),
    /// Compare
    ///
    /// ```seis
    /// CMP Vx, Vy
    /// ```
    Cmp(CompOp),
    /// Test
    ///
    /// ```seis
    /// TST Vx, Vy
    /// ```
    Tst(TestOp),
    /// And
    ///
    /// ```seis
    /// AND Vx, Vy, Vz
    /// ```
    And(BinaryOp),
    /// Inclusive or
    ///
    /// ```seis
    /// IOR Vx, Vy, Vz
    /// ```
    Ior(BinaryOp),
    /// Exclusive or
    ///
    /// ```seis
    /// XOR Vx, Vy, Vz
    /// ```
    Xor(BinaryOp),
    /// Bitwise not
    ///
    /// ```seis
    /// NOT Vx, Vy
    /// ```
    Not(UnaryOp),
    /// Sign-extend
    ///
    /// ```seis
    /// SXT Vx, Vy
    /// ```
    Sxt(SignExtendOp),
    /// Logical-shift left
    ///
    /// ```seis
    /// BSL Vx, Vy, Vz
    /// ```
    Bsl(BinaryOp),
    /// Logical-shift right
    ///
    /// ```seis
    /// BSR Vx, Vy, Vz
    /// ```
    Bsr(BinaryOp),
    /// Arithmetic shift right
    ///
    /// ```seis
    /// ASR Vx, Vy, Vz
    /// ```
    Asr(BinaryOp),
    /// Rotate left
    ///
    /// ```seis
    /// ROL Vx, Vy, Vz
    /// ```
    Rol(BinaryOp),
    /// Rotate right
    ///
    /// ```seis
    /// ROR Vx, Vy, Vz
    /// ```
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
    /// [`IntegerOp::Sxt`]
    const SXT: Word = 0b0_1010;

    /// Logical-shift left bits
    ///
    /// [`IntegerOp::Bsl`]
    const BSL: Word = 0b0_1011;
    /// Logical-shift right bits
    ///
    /// [`IntegerOp::Bsr`]
    const BSR: Word = 0b0_1100;
    /// Arithmetic-shift right bits
    ///
    /// [`IntegerOp::Asr`]
    const ASR: Word = 0b0_1101;

    /// Rotate left bits
    ///
    /// [`IntegerOp::Rol`]
    const ROL: Word = 0b0_1110;
    /// Rotate right bits
    ///
    /// [`IntegerOp::Ror`]
    const ROR: Word = 0b0_1111;

    /// Compare bits
    ///
    /// [`IntegerOp::Cmp`]
    const CMP: Word = 0b1_0000;
    /// Test bits
    ///
    /// [`IntegerOp::Tst`]
    const TST: Word = 0b1_0001;
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
            Self::SXT => Ok(Sxt(decode(word)?)),
            Self::BSL => Ok(Bsl(decode(word)?)),
            Self::BSR => Ok(Bsr(decode(word)?)),
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
            Sxt(s) => (Self::SXT << Self::SHIFT) | s.encode(),
            Bsl(b) => (Self::BSL << Self::SHIFT) | b.encode(),
            Bsr(b) => (Self::BSR << Self::SHIFT) | b.encode(),
            Asr(b) => (Self::ASR << Self::SHIFT) | b.encode(),
            Rol(b) => (Self::ROL << Self::SHIFT) | b.encode(),
            Ror(b) => (Self::ROR << Self::SHIFT) | b.encode(),
        }
    }
}

impl Info for IntegerOp {
    fn get_write_regs(self) -> RegisterFlags {
        use IntegerOp::*;

        match self {
            Not(UnaryOp(_, r))
            | Sxt(SignExtendOp(_, r))
            | Add(BinaryOp::Registers(.., r) | BinaryOp::Immediate(.., r))
            | Sub(BinaryOp::Registers(.., r) | BinaryOp::Immediate(.., r))
            | Mul(BinaryOp::Registers(.., r) | BinaryOp::Immediate(.., r))
            | Dvu(BinaryOp::Registers(.., r) | BinaryOp::Immediate(.., r))
            | Dvs(BinaryOp::Registers(.., r) | BinaryOp::Immediate(.., r))
            | Mod(BinaryOp::Registers(.., r) | BinaryOp::Immediate(.., r))
            | Bsl(BinaryOp::Registers(.., r) | BinaryOp::Immediate(.., r))
            | Bsr(BinaryOp::Registers(.., r) | BinaryOp::Immediate(.., r))
            | Asr(BinaryOp::Registers(.., r) | BinaryOp::Immediate(.., r))
            | Rol(BinaryOp::Registers(.., r) | BinaryOp::Immediate(.., r))
            | Ror(BinaryOp::Registers(.., r) | BinaryOp::Immediate(.., r))
            | And(BinaryOp::Registers(.., r) | BinaryOp::Immediate(.., r))
            | Ior(BinaryOp::Registers(.., r) | BinaryOp::Immediate(.., r))
            | Xor(BinaryOp::Registers(.., r) | BinaryOp::Immediate(.., r)) => {
                [r, ZF, OF, EPS, NAN, INF].into()
            }

            Cmp(_) | Tst(_) => [ZF, OF, EPS, NAN, INF].into(),
        }
    }

    fn get_read_regs(self) -> RegisterFlags {
        use IntegerOp::*;

        match self {
            Add(BinaryOp::Registers(r0, r1, _))
            | Sub(BinaryOp::Registers(r0, r1, _))
            | Mul(BinaryOp::Registers(r0, r1, _))
            | Dvu(BinaryOp::Registers(r0, r1, _))
            | Dvs(BinaryOp::Registers(r0, r1, _))
            | Mod(BinaryOp::Registers(r0, r1, _))
            | And(BinaryOp::Registers(r0, r1, _))
            | Ior(BinaryOp::Registers(r0, r1, _))
            | Xor(BinaryOp::Registers(r0, r1, _))
            | Bsl(BinaryOp::Registers(r0, r1, _))
            | Bsr(BinaryOp::Registers(r0, r1, _))
            | Asr(BinaryOp::Registers(r0, r1, _))
            | Rol(BinaryOp::Registers(r0, r1, _))
            | Ror(BinaryOp::Registers(r0, r1, _))
            | Cmp(CompOp::Registers(r0, r1, _))
            | Tst(TestOp::Registers(r0, r1)) => [r0, r1].into(),

            Add(BinaryOp::Immediate(r, _, _))
            | Sub(BinaryOp::Immediate(r, _, _))
            | Mul(BinaryOp::Immediate(r, _, _))
            | Dvu(BinaryOp::Immediate(r, _, _))
            | Dvs(BinaryOp::Immediate(r, _, _))
            | Mod(BinaryOp::Immediate(r, _, _))
            | And(BinaryOp::Immediate(r, _, _))
            | Ior(BinaryOp::Immediate(r, _, _))
            | Xor(BinaryOp::Immediate(r, _, _))
            | Bsl(BinaryOp::Immediate(r, _, _))
            | Bsr(BinaryOp::Immediate(r, _, _))
            | Asr(BinaryOp::Immediate(r, _, _))
            | Rol(BinaryOp::Immediate(r, _, _))
            | Ror(BinaryOp::Immediate(r, _, _))
            | Cmp(CompOp::Immediate(r, _, _))
            | Tst(TestOp::Immediate(r, _))
            | Not(UnaryOp(r, _))
            | Sxt(SignExtendOp(_, r)) => [r].into(),
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
            Sxt(s) => write!(f, "SXT {s}"),
            Bsl(b) => write!(f, "LSL {b}"),
            Bsr(b) => write!(f, "LSR {b}"),
            Asr(b) => write!(f, "ASR {b}"),
            Rol(b) => write!(f, "ROL {b}"),
            Ror(b) => write!(f, "ROR {b}"),
        }
    }
}
