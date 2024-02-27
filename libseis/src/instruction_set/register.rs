use super::{Decode, Encode};
use crate::{
    instruction_set::{decode, error::DecodeError},
    registers::{BP, LP, SP, V},
    types::{Byte, Register, Short, Word},
};
use std::fmt::{Display, Write};

#[derive(Debug, Clone, Copy)]
pub struct ImmOp(pub Word, pub Register);

impl ImmOp {
    // TODO: fill
}

impl Decode for ImmOp {
    fn decode(word: Word) -> super::error::DecodeResult<Self> {
        todo!()
    }
}

impl Encode for ImmOp {
    fn encode(self) -> Word {
        todo!()
    }
}

impl Display for ImmOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RegOp(pub Register, pub Register);

impl RegOp {
    // TODO: fill
}

impl Decode for RegOp {
    fn decode(word: Word) -> super::error::DecodeResult<Self> {
        todo!()
    }
}

impl Encode for RegOp {
    fn encode(self) -> Word {
        todo!()
    }
}

impl Display for RegOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MemOp {
    ZeroPage {
        address: Word,
        destination: Register,
    },
    Indirect {
        address: Register,
        destination: Register,
    },
    ImmIndexedIndirect {
        address: Register,
        offset: Short,
        destination: Register,
    },
    RegIndexedIndirect {
        address: Register,
        index: Register,
        destination: Register,
    },
    OffIndexedIndirect {
        address: Register,
        index: Register,
        offset: Register,
        destination: Register,
    },
}

impl MemOp {
    const DST_REG_MASK: Word = 0b0000_0000_0000_0000_0000_0000_0000_1111;

    /// A flag signifying to use a 16-bit zero-page address
    const ZPG_MASK: Word = 0b0000_0001_0000_0000_0000_0000_0000_0000;
    const ZPG_ADDR: Word = 0b0000_0000_0000_1111_1111_1111_1111_0000;
    const ZPG_SHIFT: Word = 4;

    const ADDR_MODE_MASK: Word = 0b0000_1100_0000_0000_0000_0000_0000_0000;
    const ADDR_MODE_SHIFT: Word = 26;
    const INDIRECT_MODE: Word = 0b00;
    const IMM_INDEXED_MODE: Word = 0b01;
    const REG_INDEXED_MODE: Word = 0b10;
    const OFF_INDEXED_MODE: Word = 0b11;
}

impl Decode for MemOp {
    fn decode(word: Word) -> super::error::DecodeResult<Self> {
        todo!()
    }
}

impl Encode for MemOp {
    fn encode(self) -> Word {
        todo!()
    }
}

impl Display for MemOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StackOp(Word);

impl StackOp {
    const MASK: Word = 0b0000_0000_1111_1111_1111_0000_0000_0000;
    const SHIFT: Word = 12;

    pub fn has_register(self, reg_id: Register) -> bool {
        self.0 & (1 << reg_id as Word) != 0
    }

    pub fn registers(self) -> Vec<Register> {
        let mut registers: Vec<Register> =
            V.into_iter().filter(|&v| self.has_register(v)).collect();

        if self.has_register(SP) {
            registers.push(SP)
        }

        if self.has_register(BP) {
            registers.push(BP)
        }

        if self.has_register(LP) {
            registers.push(LP)
        }

        registers
    }
}

impl Decode for StackOp {
    fn decode(word: Word) -> super::error::DecodeResult<Self> {
        let register_flags = (word & Self::MASK) >> Self::SHIFT;

        Ok(Self(register_flags))
    }
}

impl Encode for StackOp {
    fn encode(self) -> Word {
        self.0 << Self::SHIFT
    }
}

impl Display for StackOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut string = "".to_owned();

        for v in V.into_iter().filter(|&b| self.has_register(b)) {
            if string.is_empty() {
                write!(string, "V{v:X}")?;
            } else {
                write!(string, ", V{v:X}")?;
            }
        }

        if self.has_register(SP) {
            if string.is_empty() {
                write!(string, "SP")?;
            } else {
                write!(string, ", SP")?;
            }
        }

        if self.has_register(BP) {
            if string.is_empty() {
                write!(string, "BP")?;
            } else {
                write!(string, ", BP")?;
            }
        }

        if self.has_register(LP) {
            if string.is_empty() {
                write!(string, "LP")?;
            } else {
                write!(string, ", LP")?;
            }
        }

        write!(f, "{{{string}}}")
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RegisterOp {
    Lbr(MemOp),
    Lsr(MemOp),
    Llr(MemOp),
    Sbr(MemOp),
    Ssr(MemOp),
    Slr(MemOp),
    Tfr(RegOp),
    Push(StackOp),
    Pop(StackOp),
    Lol(ImmOp),
    Llz(ImmOp),
    Loh(ImmOp),
}

impl RegisterOp {
    const MASK: Word = 0b0001_1110_0000_0000_0000_0000_0000_0000;
    const SHIFT: Word = 25;

    const PUSH: Word = 0b0000;
    const POP: Word = 0b0001;
    const LBR: Word = 0b0010;
    const LSR: Word = 0b0011;
    const LLR: Word = 0b0100;
    const SBR: Word = 0b0101;
    const SSR: Word = 0b0110;
    const SLR: Word = 0b0111;
    const TFR: Word = 0b1000;
    const LOL: Word = 0b1001;
    const LLZ: Word = 0b1010;
    const LOH: Word = 0b1011;
}

impl Decode for RegisterOp {
    fn decode(word: Word) -> super::error::DecodeResult<Self> {
        use RegisterOp::*;
        let reg_op = (word & Self::MASK) >> Self::SHIFT;

        match reg_op {
            Self::PUSH => Ok(Push(decode(word)?)),
            Self::POP => Ok(Pop(decode(word)?)),
            Self::LBR => Ok(Lbr(decode(word)?)),
            Self::LSR => Ok(Lsr(decode(word)?)),
            Self::LLR => Ok(Llr(decode(word)?)),
            Self::SBR => Ok(Sbr(decode(word)?)),
            Self::SSR => Ok(Ssr(decode(word)?)),
            Self::SLR => Ok(Slr(decode(word)?)),
            Self::TFR => Ok(Tfr(decode(word)?)),
            Self::LOL => Ok(Lol(decode(word)?)),
            Self::LLZ => Ok(Llz(decode(word)?)),
            Self::LOH => Ok(Loh(decode(word)?)),
            _ => Err(DecodeError::InvalidRegisterOp(reg_op)),
        }
    }
}
