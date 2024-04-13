//! Register operations

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

/// Represents an immediate load operation
#[derive(Debug, Clone, Copy)]
pub enum ImmOp {
    /// An immediate value, with a shift
    Immediate {
        /// Whether to zero out the entire register
        zero: bool,
        /// How many bytes to shift the number
        shift: Byte,
        /// What to set the register to
        immediate: Short,
        /// Where to store the immediate value
        destination: Register,
    },
    /// Loads the address value in the lower-order bits
    /// and the short page's ID in the higher-order bits
    ZeroPageTranslate {
        /// The short-page address
        address: Short,
        /// The register to store the address to
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
            let zero = (word & Self::ZERO_FLAG) != 0;
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
                let destination = (destination as Word) & Self::DEST_REG_MASK;
                let immediate = (immediate as Word) << Self::IMM_SHIFT & Self::IMM_MASK;
                let shift = (shift as Word) << Self::IMM_BSHIFT_SHIFT & Self::IMM_BSHIFT_MASK;

                zero | destination | immediate | shift
            }
            ZeroPageTranslate {
                address,
                destination,
            } => {
                let address = (address as Word) << Self::IMM_SHIFT;
                let destination = destination as Word & Self::DEST_REG_MASK;

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

/// Represents a register transfer operation
#[derive(Debug, Clone, Copy)]
pub struct RegOp {
    /// Where to read the value from
    pub source: Register,
    /// Where to store the value to
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
    ZeroPage {
        /// The address to write to
        address: Short,
        /// The source register
        source: Register,
    },
    /// Data from an address
    Indirect {
        /// Whether to skip the cache
        volatile: bool,
        /// The register containing the effective address
        address: Register,
        /// The source register
        source: Register,
    },
    /// Data from an address at an offset (times the width of the read)
    OffsetIndirect {
        /// Whether to skip the cache
        volatile: bool,
        /// The register from which the address comes from
        address: Register,
        /// The immediate offset
        offset: Short,
        /// The source register
        source: Register,
    },
    /// Data from an address at an index (times the width of the read)
    IndexedIndirect {
        /// Whether to skip the cache
        volatile: bool,
        /// The register from which the address comes from
        address: Register,
        /// The offset from that address
        index: Register,
        /// The source register
        source: Register,
    },
    /// Data from the stack, offset from the stack base pointer
    StackOffset {
        /// The offset from the base pointer
        offset: Short,
        /// The source register
        source: Register,
    },
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

    const INDIRECT_MODE: Word = 0b000;
    const OFFSET_MODE: Word = 0b001;
    const INDEXED_MODE: Word = 0b010;
    const STACK_OFFSET_MODE: Word = 0b011;
    const VOLATILE_INDIRECT_MODE: Word = 0b100;
    const VOLATILE_OFFSET_MODE: Word = 0b101;
    const VOLATILE_INDEXED_MODE: Word = 0b110;
    const ZERO_PAGE_MODE: Word = 0b111;
}

impl Decode for WriteOp {
    fn decode(word: Word) -> DecodeResult<Self> {
        use WriteOp::*;
        let addr_mode = (word & Self::ADDR_MODE_MASK) >> Self::ADDR_MODE_SHIFT;
        let source = (word & Self::SRC_REG_MASK) as Register;

        match addr_mode {
            Self::INDIRECT_MODE => Ok(Indirect {
                volatile: false,
                address: ((word & Self::ADDRESS_REG_MASK) >> Self::ADDRESS_REG_SHIFT) as Register,
                source,
            }),
            Self::OFFSET_MODE => Ok(OffsetIndirect {
                volatile: false,
                address: ((word & Self::ADDRESS_REG_MASK) >> Self::ADDRESS_REG_SHIFT) as Register,
                offset: ((word & Self::OFFSET_MASK) >> Self::OFFSET_SHIFT) as Short,
                source,
            }),
            Self::INDEXED_MODE => Ok(IndexedIndirect {
                volatile: false,
                address: ((word & Self::ADDRESS_REG_MASK) >> Self::ADDRESS_REG_SHIFT) as Register,
                index: ((word & Self::INDEX_REG_MASK) >> Self::INDEX_REG_SHIFT) as Register,
                source,
            }),
            Self::STACK_OFFSET_MODE => Ok(StackOffset {
                offset: ((word & Self::OFFSET_MASK) >> Self::OFFSET_SHIFT) as Short,
                source,
            }),
            Self::VOLATILE_INDIRECT_MODE => Ok(Indirect {
                volatile: true,
                address: ((word & Self::ADDRESS_REG_MASK) >> Self::ADDRESS_REG_SHIFT) as Register,
                source,
            }),
            Self::VOLATILE_OFFSET_MODE => Ok(OffsetIndirect {
                volatile: true,
                address: ((word & Self::ADDRESS_REG_MASK) >> Self::ADDRESS_REG_SHIFT) as Register,
                offset: ((word & Self::OFFSET_MASK) >> Self::OFFSET_SHIFT) as Short,
                source,
            }),
            Self::VOLATILE_INDEXED_MODE => Ok(IndexedIndirect {
                volatile: true,
                address: ((word & Self::ADDRESS_REG_MASK) >> Self::ADDRESS_REG_SHIFT) as Register,
                index: ((word & Self::INDEX_REG_MASK) >> Self::INDEX_REG_SHIFT) as Register,
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
            StackOffset { offset, source } => {
                (Self::STACK_OFFSET_MODE << Self::ADDR_MODE_SHIFT)
                    | ((offset as Word) << Self::OFFSET_SHIFT)
                    | (source as Word)
            }
            ZeroPage { address, source } => {
                (Self::ZERO_PAGE_MODE << Self::ADDR_MODE_SHIFT)
                    | ((address as Word) << Self::ZPG_ADDR_SHIFT)
                    | (source as Word)
            }
            Indirect {
                volatile: false,
                address,
                source,
            } => {
                (Self::INDIRECT_MODE << Self::ADDR_MODE_SHIFT)
                    | ((address as Word) << Self::ADDRESS_REG_SHIFT)
                    | (source as Word)
            }
            OffsetIndirect {
                volatile: false,
                address,
                offset,
                source,
            } => {
                (Self::OFFSET_MODE << Self::ADDR_MODE_SHIFT)
                    | ((address as Word) << Self::ADDRESS_REG_SHIFT)
                    | ((offset as Word) << Self::OFFSET_SHIFT)
                    | (source as Word)
            }
            IndexedIndirect {
                volatile: false,
                address,
                index,
                source,
            } => {
                (Self::INDEXED_MODE << Self::ADDR_MODE_SHIFT)
                    | ((address as Word) << Self::ADDRESS_REG_SHIFT)
                    | ((index as Word) << Self::INDEX_REG_SHIFT)
                    | (source as Word)
            }
            Indirect {
                volatile: true,
                address,
                source,
            } => {
                (Self::VOLATILE_INDIRECT_MODE << Self::ADDR_MODE_SHIFT)
                    | ((address as Word) << Self::ADDRESS_REG_SHIFT)
                    | (source as Word)
            }
            OffsetIndirect {
                volatile: true,
                address,
                offset,
                source,
            } => {
                (Self::VOLATILE_OFFSET_MODE << Self::ADDR_MODE_SHIFT)
                    | ((address as Word) << Self::ADDRESS_REG_SHIFT)
                    | ((offset as Word) << Self::OFFSET_SHIFT)
                    | (source as Word)
            }
            IndexedIndirect {
                volatile: true,
                address,
                index,
                source,
            } => {
                (Self::VOLATILE_INDEXED_MODE << Self::ADDR_MODE_SHIFT)
                    | ((address as Word) << Self::ADDRESS_REG_SHIFT)
                    | ((index as Word) << Self::INDEX_REG_SHIFT)
                    | (source as Word)
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
        /// The effective address from within the short page
        address: Short,
        /// Where to store the read value
        destination: Register,
    },
    /// Data from an address
    Indirect {
        /// Whether to skip the cache
        volatile: bool,
        /// The register containing the effective address
        address: Register,
        /// Where to store the read value
        destination: Register,
    },
    /// Data from an address at an offset (times the width of the read)
    OffsetIndirect {
        /// Whether to skip the cache
        volatile: bool,
        /// The register containing an address
        address: Register,
        /// An immediate offset
        offset: Short,
        /// Where to store the read value
        destination: Register,
    },
    /// Data from an address at an index (times the width of the read)
    IndexedIndirect {
        /// Whether to skip the cache
        volatile: bool,
        /// The register containing an address
        address: Register,
        /// The register containing the offset
        index: Register,
        /// Where to store the read value
        destination: Register,
    },
    /// Data from the stack, offset from the stack base
    StackOffset {
        /// The immediate offset
        offset: Short,
        /// Where to store the read value
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
    const INDEX_REG_SHIFT: Word = 8;
    const OFFSET_MASK: Word = 0b0000_0000_0000_1111_1111_1111_0000_0000;
    const OFFSET_SHIFT: Word = 8;

    const INDIRECT_MODE: Word = 0b000;
    const OFFSET_MODE: Word = 0b001;
    const INDEXED_MODE: Word = 0b010;
    const STACK_OFFSET_MODE: Word = 0b011;
    const VOLATILE_INDIRECT_MODE: Word = 0b100;
    const VOLATILE_OFFSET_MODE: Word = 0b101;
    const VOLATILE_INDEXED_MODE: Word = 0b110;
    const ZERO_PAGE_MODE: Word = 0b111;
}

impl Decode for ReadOp {
    fn decode(word: Word) -> DecodeResult<Self> {
        use ReadOp::*;
        let addr_mode = (word & Self::ADDR_MODE_MASK) >> Self::ADDR_MODE_SHIFT;
        let destination = (word & Self::DST_REG_MASK) as Register;

        match addr_mode {
            Self::INDIRECT_MODE => Ok(Indirect {
                volatile: false,
                address: ((word & Self::ADDRESS_REG_MASK) >> Self::ADDRESS_REG_SHIFT) as Register,
                destination,
            }),
            Self::OFFSET_MODE => Ok(OffsetIndirect {
                volatile: false,
                address: ((word & Self::ADDRESS_REG_MASK) >> Self::ADDRESS_REG_SHIFT) as Register,
                offset: ((word & Self::OFFSET_MASK) >> Self::OFFSET_SHIFT) as Short,
                destination,
            }),
            Self::INDEXED_MODE => Ok(IndexedIndirect {
                volatile: false,
                address: ((word & Self::ADDRESS_REG_MASK) >> Self::ADDRESS_REG_SHIFT) as Register,
                index: ((word & Self::INDEX_REG_MASK) >> Self::INDEX_REG_SHIFT) as Register,
                destination,
            }),
            Self::STACK_OFFSET_MODE => Ok(StackOffset {
                offset: ((word & Self::OFFSET_MASK) >> Self::OFFSET_SHIFT) as Short,
                destination,
            }),
            Self::VOLATILE_INDIRECT_MODE => Ok(Indirect {
                volatile: true,
                address: ((word & Self::ADDRESS_REG_MASK) >> Self::ADDRESS_REG_SHIFT) as Register,
                destination,
            }),
            Self::VOLATILE_OFFSET_MODE => Ok(OffsetIndirect {
                volatile: true,
                address: ((word & Self::ADDRESS_REG_MASK) >> Self::ADDRESS_REG_SHIFT) as Register,
                offset: ((word & Self::OFFSET_MASK) >> Self::OFFSET_SHIFT) as Short,
                destination,
            }),
            Self::VOLATILE_INDEXED_MODE => Ok(IndexedIndirect {
                volatile: true,
                address: ((word & Self::ADDRESS_REG_MASK) >> Self::ADDRESS_REG_SHIFT) as Register,
                index: ((word & Self::INDEX_REG_MASK) >> Self::INDEX_REG_SHIFT) as Register,
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
            StackOffset {
                offset,
                destination,
            } => {
                (Self::STACK_OFFSET_MODE << Self::ADDR_MODE_SHIFT)
                    | ((offset as Word) << Self::OFFSET_SHIFT)
                    | (destination as Word)
            }
            ZeroPage {
                address,
                destination,
            } => {
                (Self::ZERO_PAGE_MODE << Self::ADDR_MODE_SHIFT)
                    | ((address as Word) << Self::ZPG_ADDR_SHIFT)
                    | (destination as Word)
            }
            Indirect {
                volatile: false,
                address,
                destination,
            } => {
                (Self::INDIRECT_MODE << Self::ADDR_MODE_SHIFT)
                    | ((address as Word) << Self::ADDRESS_REG_SHIFT)
                    | (destination as Word)
            }
            OffsetIndirect {
                volatile: false,
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
                volatile: false,
                address,
                index,
                destination,
            } => {
                (Self::INDEXED_MODE << Self::ADDR_MODE_SHIFT)
                    | ((address as Word) << Self::ADDRESS_REG_SHIFT)
                    | ((index as Word) << Self::INDEX_REG_SHIFT)
                    | (destination as Word)
            }
            Indirect {
                volatile: true,
                address,
                destination,
            } => {
                (Self::VOLATILE_INDIRECT_MODE << Self::ADDR_MODE_SHIFT)
                    | ((address as Word) << Self::ADDRESS_REG_SHIFT)
                    | (destination as Word)
            }
            OffsetIndirect {
                volatile: true,
                address,
                offset,
                destination,
            } => {
                (Self::VOLATILE_OFFSET_MODE << Self::ADDR_MODE_SHIFT)
                    | ((address as Word) << Self::ADDRESS_REG_SHIFT)
                    | ((offset as Word) << Self::OFFSET_SHIFT)
                    | (destination as Word)
            }
            IndexedIndirect {
                volatile: true,
                address,
                index,
                destination,
            } => {
                (Self::VOLATILE_INDEXED_MODE << Self::ADDR_MODE_SHIFT)
                    | ((address as Word) << Self::ADDRESS_REG_SHIFT)
                    | ((index as Word) << Self::INDEX_REG_SHIFT)
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

/// Instructions for any [register operations](crate::instruction_set::Instruction::Register)
#[derive(Debug, Clone, Copy)]
pub enum RegisterOp {
    /// Load byte to register
    ///
    /// ```seis
    /// LBR Va, Vx     ; Indirect
    /// LBR Va + n, Vx ; Offset indirect
    /// LBR Va[Vi], Vx ; Indexed indirect
    /// LBR @zpg, Vx   ; Zero page
    /// LBR %bpo, Vx   ; Base pointer offset
    /// ```
    Lbr(ReadOp),
    /// Load short to register
    ///
    /// ```seis
    /// LSR Va, Vx     ; Indirect
    /// LSR Va + n, Vx ; Offset indirect
    /// LSR Va[Vi], Vx ; Indexed indirect
    /// LSR @zpg, Vx   ; Zero page
    /// LSR %bpo, Vx   ; Base pointer offset
    /// ```
    Lsr(ReadOp),
    /// Load long (word) to register
    ///
    /// ```seis
    /// LLR Va, Vx     ; Indirect
    /// LLR Va + n, Vx ; Offset indirect
    /// LLR Va[Vi], Vx ; Indexed indirect
    /// LLR @zpg, Vx   ; Zero page
    /// LLR %bpo, Vx   ; Base pointer offset
    /// ```
    Llr(ReadOp),
    /// Store byte from register
    ///
    /// ```seis
    /// SBR Vx, Va     ; Indirect
    /// SBR Vx, Va + n ; Offset indirect
    /// SBR Vx, Va[Vi] ; Indexed indirect
    /// SBR Vx, @zpg   ; Zero page
    /// SBR Vx, %bpo   ; Base pointer offset
    /// ```
    Sbr(WriteOp),
    /// Store short from register
    ///
    /// ```seis
    /// SSR Vx, Va     ; Indirect
    /// SSR Vx, Va + n ; Offset indirect
    /// SSR Vx, Va[Vi] ; Indexed indirect
    /// SSR Vx, @zpg   ; Zero page
    /// SSR Vx, %bpo   ; Base pointer offset
    /// ```
    Ssr(WriteOp),
    /// Store long (word) from register
    ///
    /// ```seis
    /// SLR Vx, Va     ; Indirect
    /// SLR Vx, Va + n ; Offset indirect
    /// SLR Vx, Va[Vi] ; Indexed indirect
    /// SLR Vx, @zpg   ; Zero page
    /// SLR Vx, %bpo   ; Base pointer offset
    /// ```
    Slr(WriteOp),
    /// Transfer
    ///
    /// ```seis
    /// TFR Vs, Vd
    /// ```
    Tfr(RegOp),
    /// Push to stack
    ///
    /// ```seis
    /// PUSH { V1, V2, ..., Vn } ; Expands to a sequence of PUSH operations
    /// ```
    Push(Register),
    /// Pop
    ///
    /// ```seis
    /// POP { V1, V2, ..., Vn } ; Expands to a sequence of POP instructions (in reverse order)
    /// ```
    Pop(Register),
    /// Load immediate value to register
    ///
    /// ```seis
    /// LDR imm, Vx   ; Load and zero
    /// LDR imm, Vx.0 ; Load lower without zeroing
    /// LDR imm, Vx.1 ; Load upper without zeroing
    /// LDR &zpa, Vx  ; Load zero page address
    /// ```
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
