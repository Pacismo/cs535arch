use super::{error::DecodeResult, Decode, Encode, Info};
use crate::registers::{get_name, RegisterFlags, EPS, INF, NAN, OF, SP, ZF};
use crate::{
    instruction_set::{decode, error::DecodeError},
    registers,
    types::{Byte, Register, Short, Word},
};
use std::fmt::Display;

impl Decode for Register {
    fn decode(word: Word) -> DecodeResult<Self> {
        const REG_MASK: Word = 0b0000_0000_0000_0000_0000_0000_0001_1111;

        Ok((word & REG_MASK) as Register)
    }
}

impl Encode for Register {
    fn encode(self) -> Word {
        self as Word
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
                        write!(f, "{immediate} => V{destination:X}")
                    } else {
                        write!(f, "{immediate} => V{destination:X}.0")
                    }
                } else {
                    write!(f, "{immediate} => V{destination:X}.{shift}")
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
pub enum WriteOp {
    /// Address with the zero-page ID in the upper bytes
    ZeroPage { address: Short, source: Register },
    /// Data from an address
    Indirect {
        volatile: bool,
        address: Register,
        source: Register,
    },
    /// Data from an address at an offset (times the width of the read)
    OffsetIndirect {
        volatile: bool,
        address: Register,
        offset: Short,
        source: Register,
    },
    /// Data from an address at an index (times the width of the read)
    IndexedIndirect {
        volatile: bool,
        address: Register,
        index: Register,
        source: Register,
    },
    /// Data from the stack, offset (multiplied by the width of the read) from the stack pointer
    StackOffset { offset: Short, source: Register },
}

impl WriteOp {
    const SRC_REG_MASK: Word = 0b0000_0000_0000_0000_0000_0000_0000_1111;

    const ZPG_ADDR_MASK: Word = 0b0000_0000_0000_1111_1111_1111_1111_0000;
    const ZPG_ADDR_SHIFT: Word = 4;

    const ADDR_MODE_MASK: Word = 0b0000_0000_1110_0000_0000_0000_0000_0000;
    const ADDR_MODE_SHIFT: Word = 21;

    const ADDRESS_REG_MASK: Word = 0b0000_0000_0000_0000_0000_0000_1111_0000;
    const ADDRESS_REG_SHIFT: Word = 4;
    const INDEX_REG_MASK: Word = 0b0000_0000_0000_0000_0000_1111_0000_0000;
    const INDEX_REG_SHIFT: Word = 8;
    const OFFSET_MASK: Word = 0b0000_0000_0000_1111_1111_1111_0000_0000;
    const OFFSET_SHIFT: Word = 8;

    const VOLATILE_BIT: Word = 0b100;
    const INDIRECT_MODE: Word = 0b000;
    const OFFSET_MODE: Word = 0b001;
    const INDEXED_MODE: Word = 0b010;
    const STACK_OFFSET_MODE: Word = 0b011;

    const ZERO_PAGE_MODE: Word = 0b111;
}

impl Decode for WriteOp {
    fn decode(word: Word) -> DecodeResult<Self> {
        use WriteOp::*;
        let addr_mode = (word & Self::ADDR_MODE_MASK) >> Self::ADDR_MODE_SHIFT;
        let source = (word & Self::SRC_REG_MASK) as Register;

        match addr_mode {
            Self::INDIRECT_MODE => Ok(Indirect {
                volatile: (word & Self::VOLATILE_BIT) != 0,
                address: ((word & Self::ADDRESS_REG_MASK) >> Self::ADDRESS_REG_SHIFT) as Register,
                source,
            }),
            Self::OFFSET_MODE => Ok(OffsetIndirect {
                volatile: (word & Self::VOLATILE_BIT) != 0,
                address: ((word & Self::ADDRESS_REG_MASK) >> Self::ADDRESS_REG_SHIFT) as Register,
                offset: ((word & Self::OFFSET_MASK) >> Self::OFFSET_SHIFT) as Short,
                source,
            }),
            Self::INDEXED_MODE => Ok(IndexedIndirect {
                volatile: (word & Self::VOLATILE_BIT) != 0,
                address: ((word & Self::ADDRESS_REG_MASK) >> Self::ADDRESS_REG_SHIFT) as Register,
                index: ((word & Self::INDEX_REG_MASK) >> Self::INDEX_REG_SHIFT) as Register,
                source,
            }),
            Self::STACK_OFFSET_MODE => Ok(StackOffset {
                offset: ((word & Self::OFFSET_MASK) >> Self::OFFSET_SHIFT) as Short,
                source,
            }),
            Self::ZERO_PAGE_MODE => Ok(ZeroPage {
                address: ((word & Self::ZPG_ADDR_MASK) >> Self::ZPG_ADDR_SHIFT) as Short,
                source,
            }),
            _ => Err(DecodeError::InvalidAddressingMode(addr_mode)),
        }
    }
}

impl Encode for WriteOp {
    fn encode(self) -> Word {
        use WriteOp::*;

        match self {
            ZeroPage {
                address,
                source: destination,
            } => {
                (Self::ZERO_PAGE_MODE << Self::ADDR_MODE_SHIFT)
                    | ((address as Word) << Self::ZPG_ADDR_SHIFT)
                    | (destination as Word)
            }
            Indirect {
                volatile,
                address,
                source: destination,
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
                source: destination,
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
                source,
            } => {
                (Self::INDEXED_MODE << Self::ADDR_MODE_SHIFT)
                    | if volatile { Self::VOLATILE_BIT } else { 0 }
                    | ((address as Word) << Self::ADDRESS_REG_SHIFT)
                    | ((index as Word) << Self::INDEX_REG_SHIFT)
                    | (source as Word)
            }
            StackOffset {
                offset,
                source: destination,
            } => {
                (Self::STACK_OFFSET_MODE << Self::ADDR_MODE_SHIFT)
                    | ((offset as Word) << Self::OFFSET_SHIFT)
                    | (destination as Word)
            }
        }
    }
}

impl Display for WriteOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use WriteOp::*;

        match self {
            ZeroPage { address, source } => write!(f, "V{source:X} => @{address:#x}"),
            Indirect {
                volatile,
                address,
                source,
            } => write!(
                f,
                "V{source:X} {assign} V{address:X}",
                assign = if *volatile { "=>>" } else { "=>" }
            ),
            OffsetIndirect {
                volatile,
                address,
                offset,
                source,
            } => write!(
                f,
                "V{source:X} {assign} V{address:X} + {offset}",
                assign = if *volatile { "=>>" } else { "=>" }
            ),
            IndexedIndirect {
                volatile,
                address,
                index,
                source,
            } => write!(
                f,
                "V{source:X} {assign} V{address:X}[V{index:X}]",
                assign = if *volatile { "=>>" } else { "=>" }
            ),
            StackOffset { offset, source } => write!(f, "V{source:X} => %{offset}"),
        }
    }
}

/// Represents addressing modes.
/// Includes translation for the zero-page.
#[derive(Debug, Clone, Copy)]
pub enum ReadOp {
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
    /// Data from the stack, offset (multiplied by the width of the read) from the stack base
    StackOffset {
        offset: Short,
        destination: Register,
    },
}

impl ReadOp {
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

impl Decode for ReadOp {
    fn decode(word: Word) -> DecodeResult<Self> {
        use ReadOp::*;
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

impl Encode for ReadOp {
    fn encode(self) -> Word {
        use ReadOp::*;

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

impl Display for ReadOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ReadOp::*;

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
                "V{address:X} + {offset} {assign} V{destination:X}",
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
pub enum RegisterOp {
    Lbr(ReadOp),
    Lsr(ReadOp),
    Llr(ReadOp),
    Sbr(WriteOp),
    Ssr(WriteOp),
    Slr(WriteOp),
    Tfr(RegOp),
    Push(Register),
    Pop(Register),
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
            Lbr(m) => (Self::LBR << Self::SHIFT) | m.encode(),
            Lsr(m) => (Self::LSR << Self::SHIFT) | m.encode(),
            Llr(m) => (Self::LLR << Self::SHIFT) | m.encode(),
            Sbr(m) => (Self::SBR << Self::SHIFT) | m.encode(),
            Ssr(m) => (Self::SSR << Self::SHIFT) | m.encode(),
            Slr(m) => (Self::SLR << Self::SHIFT) | m.encode(),
            Tfr(r) => (Self::TFR << Self::SHIFT) | r.encode(),
            Push(p) => (Self::PUSH << Self::SHIFT) | p.encode(),
            Pop(p) => (Self::POP << Self::SHIFT) | p.encode(),
            Ldr(i) => (Self::LDR << Self::SHIFT) | i.encode(),
        }
    }
}

impl Info for RegisterOp {
    fn get_write_regs(self) -> RegisterFlags {
        use RegisterOp::*;

        match self {
            Lbr(
                ReadOp::IndexedIndirect { destination, .. }
                | ReadOp::Indirect { destination, .. }
                | ReadOp::OffsetIndirect { destination, .. }
                | ReadOp::StackOffset { destination, .. }
                | ReadOp::ZeroPage { destination, .. },
            )
            | Lsr(
                ReadOp::IndexedIndirect { destination, .. }
                | ReadOp::Indirect { destination, .. }
                | ReadOp::OffsetIndirect { destination, .. }
                | ReadOp::StackOffset { destination, .. }
                | ReadOp::ZeroPage { destination, .. },
            )
            | Llr(
                ReadOp::IndexedIndirect { destination, .. }
                | ReadOp::Indirect { destination, .. }
                | ReadOp::OffsetIndirect { destination, .. }
                | ReadOp::StackOffset { destination, .. }
                | ReadOp::ZeroPage { destination, .. },
            )
            | Tfr(RegOp { destination, .. })
            | Ldr(
                ImmOp::Immediate { destination, .. } | ImmOp::ZeroPageTranslate { destination, .. },
            ) => [destination, ZF, OF, EPS, NAN, INF].into(),

            Pop(reg) => [reg, SP, ZF, OF, EPS, NAN, INF].into(),
            Push(_) => [SP].into(),

            _ => [].into(),
        }
    }

    fn get_read_regs(self) -> RegisterFlags {
        use crate::registers::BP;
        use RegisterOp::*;

        match self {
            Lbr(r) | Lsr(r) | Llr(r) => match r {
                ReadOp::Indirect { address, .. } => [address].into(),
                ReadOp::OffsetIndirect { address, .. } => [address].into(),
                ReadOp::IndexedIndirect { address, index, .. } => [address, index].into(),
                ReadOp::StackOffset { .. } => [BP].into(),
                _ => [].into(),
            },
            Sbr(w) | Ssr(w) | Slr(w) => match w {
                WriteOp::ZeroPage { source, .. } => [source].into(),
                WriteOp::Indirect {
                    address, source, ..
                } => [address, source].into(),
                WriteOp::OffsetIndirect {
                    address, source, ..
                } => [address, source].into(),
                WriteOp::IndexedIndirect {
                    address,
                    index,
                    source,
                    ..
                } => [address, index, source].into(),
                WriteOp::StackOffset { source, .. } => [source, BP].into(),
            },
            Tfr(RegOp { source, .. }) => [source].into(),
            Push(reg) => [reg, SP].into(),
            Pop(_) => [SP].into(),
            Ldr(ImmOp::Immediate { destination, .. }) => [destination].into(),

            _ => [].into(),
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
            &Push(r) => write!(f, "PUSH {{{}}}", get_name(r).unwrap_or("unknown")),
            &Pop(r) => write!(f, "POP {{{}}}", get_name(r).unwrap_or("unknown")),
            Ldr(i) => write!(f, "LDR {i}"),
        }
    }
}
