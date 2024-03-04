use super::{error::DecodeResult, Decode, Encode};
use crate::{
    instruction_set::{decode, error::DecodeError},
    registers::{self, BP, LP, SP, V},
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
pub enum ImmOp {
    Immediate {
        zero: bool,
        shift: Byte,
        immediate: Short,
        destination: Register,
    },
    ZeroPageTranslate {
        address: Short,
        destination: Register,
    },
}

impl ImmOp {
    const ZPG_TRANSLATE: Word = 0b0000_0000_1000_0000_0000_0000_0000_0000;
    const ZERO_FLAG: Word = 0b0000_0000_0100_0000_0000_0000_0000_0000;
    const DEST_REG_MASK: Word = 0b0000_0000_0000_0000_0000_0000_0000_1111;
    const IMM_MASK: Word = 0b0000_0000_0000_1111_1111_1111_1111_0000;
    const IMM_SHIFT: Word = 4;
    const IMM_BSHIFT_MASK: Word = 0b0000_0000_0011_0000_0000_0000_0000_0000;
    const IMM_BSHIFT_SHIFT: Word = 20;
}

impl Decode for ImmOp {
    fn decode(word: Word) -> DecodeResult<Self> {
        use ImmOp::*;

        if word & Self::ZPG_TRANSLATE == 0 {
            let zero = (word & Self::ZERO_FLAG) == 0;
            let destination = (word & Self::DEST_REG_MASK) as Register;
            let immediate = ((word & Self::IMM_MASK) >> Self::IMM_SHIFT) as Short;
            let shift = ((word & Self::IMM_BSHIFT_MASK) >> Self::IMM_BSHIFT_SHIFT) as Byte;

            Ok(Immediate {
                zero,
                shift,
                immediate,
                destination,
            })
        } else {
            let address = ((word & Self::IMM_MASK) >> Self::IMM_SHIFT) as Short;
            let destination = (word & Self::DEST_REG_MASK) as Register;

            Ok(ZeroPageTranslate {
                address,
                destination,
            })
        }
    }
}

impl Encode for ImmOp {
    fn encode(self) -> Word {
        use ImmOp::*;

        match self {
            Immediate {
                zero,
                shift,
                immediate,
                destination,
            } => {
                let zero = if zero { Self::ZERO_FLAG } else { 0 };
                let destination = destination as Word;
                let immediate = (immediate as Word) << Self::IMM_SHIFT;
                let shift = (shift as Word) << Self::IMM_BSHIFT_SHIFT;

                zero | destination | immediate | shift
            }
            ZeroPageTranslate {
                address,
                destination,
            } => {
                let address = (address as Word) << Self::IMM_SHIFT;
                let destination = destination as Word;

                address | destination
            }
        }
    }
}

impl Display for ImmOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ImmOp::*;

        match self {
            Immediate {
                zero,
                shift,
                immediate,
                destination,
            } => {
                if *shift == 0 {
                    if *zero {
                        write!(f, "#{immediate} => V{destination:X}")
                    } else {
                        write!(f, "#{immediate} => V{destination:X}.0")
                    }
                } else {
                    write!(f, "#{immediate} => V{destination:X}.{shift}")
                }
            }
            ZeroPageTranslate {
                address,
                destination,
            } => {
                write!(f, "&{address} => V{destination:X}")
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RegOp {
    pub source: Register,
    pub destination: Register,
}

impl RegOp {
    const SRC_REGISTER_MASK: Word = 0b0000_0000_0000_0000_1111_1111_0000_0000;
    const DST_REGISTER_MASK: Word = 0b0000_0000_0000_0000_0000_0000_1111_1111;
}

impl Decode for RegOp {
    fn decode(word: Word) -> DecodeResult<Self> {
        let source = ((word & Self::SRC_REGISTER_MASK) >> 8) as Register;
        let destination = (word & Self::DST_REGISTER_MASK) as Register;

        if (source as usize) < registers::COUNT && (destination as usize) < registers::COUNT {
            Ok(Self {
                source,
                destination,
            })
        } else {
            Err(DecodeError::InvalidRegister(source, destination))
        }
    }
}

impl Encode for RegOp {
    fn encode(self) -> Word {
        ((self.source as Word) << 8) | (self.destination as Word)
    }
}

impl Display for RegOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} => {}",
            registers::get_name(self.source).ok_or(std::fmt::Error)?,
            registers::get_name(self.destination).ok_or(std::fmt::Error)?
        )
    }
}

/// Represents addressing modes.
/// Includes translation for the zero-page.
#[derive(Debug, Clone, Copy)]
pub enum MemOp {
    /// Address with the zero-page ID in the upper bytes
    ZeroPage {
        address: Short,
        destination: Register,
    },
    /// Data from an address
    Indirect {
        volatile: bool,
        address: Register,
        destination: Register,
    },
    /// Data from an address at an offset (times the width of the read)
    OffsetIndirect {
        volatile: bool,
        address: Register,
        offset: Short,
        destination: Register,
    },
    /// Data from an address at an index (times the width of the read)
    IndexedIndirect {
        volatile: bool,
        address: Register,
        index: Register,
        destination: Register,
    },
    /// Data from the stack, offset (multiplied by the width of the read) from the stack pointer
    StackOffset {
        offset: Short,
        destination: Register,
    },
}

impl MemOp {
    const DST_REG_MASK: Word = 0b0000_0000_0000_0000_0000_0000_0000_1111;

    const ZPG_ADDR_MASK: Word = 0b0000_0000_0000_1111_1111_1111_1111_0000;
    const ZPG_ADDR_SHIFT: Word = 4;

    const ADDR_MODE_MASK: Word = 0b0000_0000_1110_0000_0000_0000_0000_0000;
    const ADDR_MODE_SHIFT: Word = 21;

    const ADDRESS_REG_MASK: Word = 0b0000_0000_0000_0000_0000_0000_1111_0000;
    const ADDRESS_REG_SHIFT: Word = 4;
    const INDEX_REG_MASK: Word = 0b0000_0000_0000_0000_0000_1111_0000_0000;
    const INDEX_REG_SHIFT: Word = 12;
    const OFFSET_MASK: Word = 0b0000_0000_0000_1111_1111_1111_0000_0000;
    const OFFSET_SHIFT: Word = 8;

    const VOLATILE_BIT: Word = 0b100;
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
                volatile: (word & Self::VOLATILE_BIT) != 0,
                address: ((word & Self::ADDRESS_REG_MASK) >> Self::ADDRESS_REG_SHIFT) as Register,
                destination,
            }),
            Self::OFFSET_MODE => Ok(OffsetIndirect {
                volatile: (word & Self::VOLATILE_BIT) != 0,
                address: ((word & Self::ADDRESS_REG_MASK) >> Self::ADDRESS_REG_SHIFT) as Register,
                offset: ((word & Self::OFFSET_MASK) >> Self::OFFSET_SHIFT) as Short,
                destination,
            }),
            Self::INDEXED_MODE => Ok(IndexedIndirect {
                volatile: (word & Self::VOLATILE_BIT) != 0,
                address: ((word & Self::ADDRESS_REG_MASK) >> Self::ADDRESS_REG_SHIFT) as Register,
                index: ((word & Self::INDEX_REG_MASK) >> Self::INDEX_REG_SHIFT) as Register,
                destination,
            }),
            Self::STACK_OFFSET_MODE => Ok(StackOffset {
                offset: ((word & Self::OFFSET_MASK) >> Self::OFFSET_SHIFT) as Short,
                destination,
            }),
            Self::ZERO_PAGE_MODE => Ok(ZeroPage {
                address: ((word & Self::ZPG_ADDR_MASK) >> Self::ZPG_ADDR_SHIFT) as Short,
                destination,
            }),
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
            Indirect {
                volatile,
                address,
                destination,
            } => {
                (Self::INDIRECT_MODE << Self::ADDR_MODE_SHIFT)
                    | if volatile { Self::VOLATILE_BIT } else { 0 }
                    | ((address as Word) << Self::ADDRESS_REG_SHIFT)
                    | (destination as Word)
            }
            OffsetIndirect {
                volatile,
                address,
                offset,
                destination,
            } => {
                (Self::OFFSET_MODE << Self::ADDR_MODE_SHIFT)
                    | if volatile { Self::VOLATILE_BIT } else { 0 }
                    | ((address as Word) << Self::ADDRESS_REG_SHIFT)
                    | ((offset as Word) << Self::OFFSET_SHIFT)
                    | (destination as Word)
            }
            IndexedIndirect {
                volatile,
                address,
                index,
                destination,
            } => {
                (Self::INDEXED_MODE << Self::ADDR_MODE_SHIFT)
                    | if volatile { Self::VOLATILE_BIT } else { 0 }
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
            Indirect {
                volatile,
                address,
                destination,
            } => write!(
                f,
                "V{address:X} {assign} V{destination:X}",
                assign = if *volatile { "=>>" } else { "=>" }
            ),
            OffsetIndirect {
                volatile,
                address,
                offset,
                destination,
            } => write!(
                f,
                "V{address:X} + #{offset} {assign} V{destination:X}",
                assign = if *volatile { "=>>" } else { "=>" }
            ),
            IndexedIndirect {
                volatile,
                address,
                index,
                destination,
            } => write!(
                f,
                "V{address:X}[V{index:X}] {assign} V{destination:X}",
                assign = if *volatile { "=>>" } else { "=>" }
            ),
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
    const EXTEND_MODE: Word = 0b0000_0001_0000_0000_0000_0000_0000_0000;
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

        if word & Self::EXTEND_MODE == 0 {
            Ok(Registers(decode(word)?))
        } else {
            Ok(Extend((word & Self::EXTEND_MASK) as Short))
        }
    }
}

impl Encode for PushOp {
    fn encode(self) -> Word {
        use PushOp::*;

        match self {
            Registers(reg) => reg.encode(),
            Extend(size) => Self::EXTEND_MASK | size as Word,
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
    const SHRINK_MODE: Word = 0b0000_0001_0000_0000_0000_0000_0000_0000;
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
        if word & Self::SHRINK_MODE == 0 {
            Ok(Registers(decode(word)?))
        } else {
            Ok(Shrink((word & Self::SHRINK_MASK) as Short))
        }
    }
}

impl Encode for PopOp {
    fn encode(self) -> Word {
        use PopOp::*;

        match self {
            Registers(reg) => reg.encode(),
            Shrink(size) => Self::SHRINK_MODE | size as Word,
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
    Ldr(ImmOp),
}

impl RegisterOp {
    const MASK: Word = 0b0001_1110_0000_0000_0000_0000_0000_0000;
    const SHIFT: Word = 25;

    const PUSH: Word = 0b0000;
    const POP: Word = 0b0001;
    const LBR: Word = 0b0010;
    const SBR: Word = 0b0011;
    const LSR: Word = 0b0100;
    const SSR: Word = 0b0101;
    const LLR: Word = 0b0110;
    const SLR: Word = 0b0111;
    const TFR: Word = 0b1000;
    const LDR: Word = 0b1001;
}

impl Decode for RegisterOp {
    fn decode(word: Word) -> DecodeResult<Self> {
        use RegisterOp::*;
        let reg_op = (word & Self::MASK) >> Self::SHIFT;

        match reg_op {
            Self::PUSH => Ok(Push(decode(word)?)),
            Self::POP => Ok(Pop(decode(word)?)),
            Self::LBR => Ok(Lbr(decode(word)?)),
            Self::SBR => Ok(Sbr(decode(word)?)),
            Self::LSR => Ok(Lsr(decode(word)?)),
            Self::SSR => Ok(Ssr(decode(word)?)),
            Self::LLR => Ok(Llr(decode(word)?)),
            Self::SLR => Ok(Slr(decode(word)?)),
            Self::TFR => Ok(Tfr(decode(word)?)),
            Self::LDR => Ok(Ldr(decode(word)?)),
            _ => Err(DecodeError::InvalidRegisterOp(reg_op)),
        }
    }
}

impl Encode for RegisterOp {
    fn encode(self) -> Word {
        use RegisterOp::*;

        match self {
            Lbr(m) => m.encode(),
            Lsr(m) => m.encode(),
            Llr(m) => m.encode(),
            Sbr(m) => m.encode(),
            Ssr(m) => m.encode(),
            Slr(m) => m.encode(),
            Tfr(r) => r.encode(),
            Push(p) => p.encode(),
            Pop(p) => p.encode(),
            Ldr(i) => i.encode(),
        }
    }
}

impl Display for RegisterOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use RegisterOp::*;

        match self {
            Lbr(m) => write!(f, "LBR {m}"),
            Lsr(m) => write!(f, "LSR {m}"),
            Llr(m) => write!(f, "LLR {m}"),
            Sbr(m) => write!(f, "SBR {m}"),
            Ssr(m) => write!(f, "SSR {m}"),
            Slr(m) => write!(f, "SLR {m}"),
            Tfr(r) => write!(f, "TFR {r}"),
            Push(p) => write!(f, "PUSH {p}"),
            Pop(p) => write!(f, "POP {p}"),
            Ldr(i) => write!(f, "LDR {i}"),
        }
    }
}
