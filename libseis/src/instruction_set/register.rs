use super::{error::DecodeResult, Decode, Encode};
use crate::{
    instruction_set::{decode, error::DecodeError},
    registers::{BP, LP, SP, V},
    types::{Byte, Register, Short, Word},
};
use std::fmt::{Display, Write};

#[derive(Debug, Clone, Copy)]
pub struct RegisterFlags(Word);

impl RegisterFlags {
    const REG_MASK: Word = 0b0000_0000_0000_1111_1111_1111_1111_1111;

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

impl Decode for RegisterFlags {
    fn decode(word: Word) -> DecodeResult<Self> {
        let register_flags = word & Self::REG_MASK;

        Ok(Self(register_flags))
    }
}

impl Encode for RegisterFlags {
    fn encode(self) -> Word {
        self.0
    }
}

impl Display for RegisterFlags {
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
pub struct ImmOp(pub Word, pub Register);

impl ImmOp {
    // TODO: fill
}

impl Decode for ImmOp {
    fn decode(word: Word) -> DecodeResult<Self> {
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
    fn decode(word: Word) -> DecodeResult<Self> {
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
    /// Read from an address with the zero-page ID in the upper bytes
    ZeroPage {
        address: Short,
        destination: Register,
    },
    /// Create an address with the zero-page ID in the upper bytes
    ZeroPageTranslate {
        address: Short,
        destination: Register,
    },
    Indirect {
        address: Register,
        destination: Register,
    },
    OffsetIndirect {
        address: Register,
        offset: Short,
        destination: Register,
    },
    IndexedIndirect {
        address: Register,
        index: Register,
        destination: Register,
    },
    StackOffset {
        offset: Short,
        destination: Register,
    },
}

/// TODO: decode
impl MemOp {
    const DST_REG_MASK: Word = 0b0000_0000_0000_0000_0000_0000_0000_1111;

    /// A flag signifying to translate a 16-bit zero-page address into an absolute address
    /// The zero page can exist anywhere
    const ZPG_TRANSLATE: Word = 0b0000_0001_0000_0000_0000_0000_0000_0000;
    const ZPG_ADDR_MASK: Word = 0b0000_0000_0000_1111_1111_1111_1111_0000;
    const ZPG_ADDR_SHIFT: Word = 4;

    const ADDR_MODE_MASK: Word = 0b0000_1110_0000_0000_0000_0000_0000_0000;
    const ADDR_MODE_SHIFT: Word = 25;

    const ADDRESS_REG_MASK: Word = 0b0000_0000_0000_0000_0000_0000_1111_0000;
    const ADDRESS_REG_SHIFT: Word = 4;
    const INDEX_REG_MASK: Word = 0b0000_0000_0000_0000_0000_1111_0000_0000;
    const INDEX_REG_SHIFT: Word = 12;
    const OFFSET_MASK: Word = 0b0000_0000_1111_1111_1111_1111_0000_0000;
    const OFFSET_SHIFT: Word = 8;

    const INDIRECT_MODE: Word = 0b000;
    const OFFSET_MODE: Word = 0b001;
    const INDEXED_MODE: Word = 0b010;
    const STACK_OFFSET_MODE: Word = 0b011;

    const ZERO_PAGE_MODE: Word = 0b111;
}

impl Decode for MemOp {
    fn decode(word: Word) -> DecodeResult<Self> {
        use MemOp::*;
        let addr_mode = (word & Self::ADDR_MODE_MASK) >> Self::ADDR_MODE_SHIFT;
        let destination = (word & Self::DST_REG_MASK) as Register;

        match addr_mode {
            Self::INDIRECT_MODE => Ok(Indirect {
                address: ((word & Self::ADDRESS_REG_MASK) >> Self::ADDRESS_REG_SHIFT) as Register,
                destination,
            }),
            Self::OFFSET_MODE => Ok(OffsetIndirect {
                address: ((word & Self::ADDRESS_REG_MASK) >> Self::ADDRESS_REG_SHIFT) as Register,
                offset: ((word & Self::OFFSET_MASK) >> Self::OFFSET_SHIFT) as Short,
                destination,
            }),
            Self::INDEXED_MODE => Ok(IndexedIndirect {
                address: ((word & Self::ADDRESS_REG_MASK) >> Self::ADDRESS_REG_SHIFT) as Register,
                index: ((word & Self::INDEX_REG_MASK) >> Self::INDEX_REG_SHIFT) as Register,
                destination,
            }),
            Self::STACK_OFFSET_MODE => Ok(StackOffset {
                offset: ((word & Self::OFFSET_MASK) >> Self::OFFSET_SHIFT) as Short,
                destination,
            }),
            Self::ZERO_PAGE_MODE => {
                let address = ((word & Self::ZPG_ADDR_MASK) >> Self::ZPG_ADDR_SHIFT) as Short;

                if word & Self::ZPG_TRANSLATE == 0 {
                    Ok(ZeroPage {
                        address,
                        destination,
                    })
                } else {
                    Ok(ZeroPageTranslate {
                        address,
                        destination,
                    })
                }
            }
            _ => Err(DecodeError::InvalidAddressingMode(addr_mode)),
        }
    }
}

impl Encode for MemOp {
    fn encode(self) -> Word {
        use MemOp::*;

        match self {
            ZeroPage {
                address,
                destination,
            } => {
                (Self::ZERO_PAGE_MODE << Self::ADDR_MODE_SHIFT)
                    | ((address as Word) << Self::ZPG_ADDR_SHIFT)
                    | (destination as Word)
            }
            ZeroPageTranslate {
                address,
                destination,
            } => {
                (Self::ZERO_PAGE_MODE << Self::ADDR_MODE_SHIFT)
                    | Self::ZPG_TRANSLATE
                    | ((address as Word) << Self::ZPG_ADDR_SHIFT)
                    | (destination as Word)
            }
            Indirect {
                address,
                destination,
            } => {
                (Self::INDIRECT_MODE << Self::ADDR_MODE_SHIFT)
                    | ((address as Word) << Self::ADDRESS_REG_SHIFT)
                    | (destination as Word)
            }
            OffsetIndirect {
                address,
                offset,
                destination,
            } => {
                (Self::OFFSET_MODE << Self::ADDR_MODE_SHIFT)
                    | ((address as Word) << Self::ADDRESS_REG_SHIFT)
                    | ((offset as Word) << Self::OFFSET_SHIFT)
                    | (destination as Word)
            }
            IndexedIndirect {
                address,
                index,
                destination,
            } => {
                (Self::INDEXED_MODE << Self::ADDR_MODE_SHIFT)
                    | ((address as Word) << Self::ADDRESS_REG_SHIFT)
                    | ((index as Word) << Self::INDEX_REG_SHIFT)
                    | (destination as Word)
            }
            StackOffset {
                offset,
                destination,
            } => {
                (Self::STACK_OFFSET_MODE << Self::ADDR_MODE_SHIFT)
                    | ((offset as Word) << Self::OFFSET_SHIFT)
                    | (destination as Word)
            }
        }
    }
}

impl Display for MemOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use MemOp::*;

        match self {
            ZeroPage {
                address,
                destination,
            } => write!(f, "@{address:#x} => V{destination:X}"),
            ZeroPageTranslate {
                address,
                destination,
            } => write!(f, "&{address:#x} => V{destination:X}"),
            Indirect {
                address,
                destination,
            } => write!(f, "V{address:X} => V{destination:X}"),
            OffsetIndirect {
                address,
                offset,
                destination,
            } => write!(f, "V{address:X} + #{offset} => V{destination:X}"),
            IndexedIndirect {
                address,
                index,
                destination,
            } => write!(f, "V{address:X}[V{index:X}] => V{destination:X}"),
            StackOffset {
                offset,
                destination,
            } => write!(f, "%{offset} => V{destination:X}"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PushOp {
    Registers(RegisterFlags),
    Extend(Short),
}

impl PushOp {
    const MASK: Word = 0b0000_0001_0000_0000_0000_0000_0000_0000;
    const SHIFT: Word = 24;
    const REGISTERS: Word = 0b0;
    const EXTEND: Word = 0b1;

    const EXTEND_MASK: Word = 0b0000_0000_0000_0000_1111_1111_1111_1111;

    pub fn has_register(self, reg_id: Register) -> bool {
        use PushOp::*;

        match self {
            Registers(reg) => reg.has_register(reg_id),
            _ => false,
        }
    }

    pub fn registers(self) -> Vec<Register> {
        use PushOp::*;

        match self {
            Registers(reg) => reg.registers(),
            _ => vec![],
        }
    }
}

impl Decode for PushOp {
    fn decode(word: Word) -> DecodeResult<Self> {
        use PushOp::*;
        let op_mode = (word & Self::MASK) >> Self::SHIFT;

        match op_mode {
            Self::REGISTERS => Ok(Registers(decode(word)?)),
            Self::EXTEND => Ok(Extend((word & Self::EXTEND_MASK) as Short)),
            _ => Err(DecodeError::InvalidPushOp(op_mode)),
        }
    }
}

impl Encode for PushOp {
    fn encode(self) -> Word {
        use PushOp::*;

        match self {
            Registers(reg) => (Self::REGISTERS << Self::SHIFT) | reg.encode(),
            Extend(size) => (Self::EXTEND << Self::SHIFT) | size as Word,
        }
    }
}

impl Display for PushOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use PushOp::*;

        match self {
            Registers(reg) => write!(f, "PUSH {reg}"),
            Extend(ext) => write!(f, "PUSH {ext}"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PopOp {
    Registers(RegisterFlags),
    Shrink(Short),
}

impl PopOp {
    const MASK: Word = 0b0000_0001_0000_0000_0000_0000_0000_0000;
    const SHIFT: Word = 24;
    const REGISTERS: Word = 0b0;
    const SHRINK: Word = 0b1;

    const SHRINK_MASK: Word = 0b0000_0000_0000_0000_1111_1111_1111_1111;

    pub fn has_register(self, reg_id: Register) -> bool {
        use PopOp::*;

        match self {
            Registers(reg) => reg.has_register(reg_id),
            _ => false,
        }
    }

    pub fn registers(self) -> Vec<Register> {
        use PopOp::*;

        match self {
            Registers(reg) => reg.registers(),
            _ => vec![],
        }
    }
}

impl Decode for PopOp {
    fn decode(word: Word) -> DecodeResult<Self> {
        use PopOp::*;
        let op_mode = (word & Self::MASK) >> Self::SHIFT;

        match op_mode {
            Self::REGISTERS => Ok(Registers(decode(word)?)),
            Self::SHRINK => Ok(Shrink((word & Self::SHRINK_MASK) as Short)),
            _ => Err(DecodeError::InvalidPopOp(op_mode)),
        }
    }
}

impl Encode for PopOp {
    fn encode(self) -> Word {
        use PopOp::*;

        match self {
            Registers(reg) => (Self::REGISTERS << Self::SHIFT) | reg.encode(),
            Shrink(size) => (Self::SHRINK << Self::SHIFT) | size as Word,
        }
    }
}

impl Display for PopOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use PopOp::*;

        match self {
            Registers(reg) => write!(f, "POP {reg}"),
            Shrink(size) => write!(f, "POP {size}"),
        }
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
    Push(PushOp),
    Pop(PopOp),
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
    const LLZ: Word = 0b1001;
    const LOL: Word = 0b1010;
    const LOH: Word = 0b1011;
}

impl Decode for RegisterOp {
    fn decode(word: Word) -> DecodeResult<Self> {
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
