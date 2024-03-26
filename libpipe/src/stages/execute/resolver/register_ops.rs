use super::Resolver;
use crate::{regmap::RegMap, stages::execute::ExecuteResult};
use libseis::{
    instruction_set::{
        register::{ImmOp, ReadOp, RegOp, WriteOp},
        RegisterOp,
    },
    pages::ZERO_PAGE,
    registers::{BP, SP},
    types::{Byte, Register, Short, Word},
};

fn compute_read_address(op: ReadOp, regvals: RegMap) -> (Word, Register, bool) {
    match op {
        ReadOp::ZeroPage {
            address,
            destination,
        } => ((ZERO_PAGE | (address as Word)), destination, false),
        ReadOp::Indirect {
            volatile,
            address,
            destination,
        } => (regvals[address], destination, volatile),
        ReadOp::OffsetIndirect {
            volatile,
            address,
            offset,
            destination,
        } => (
            regvals[address].wrapping_add(offset as Word),
            destination,
            volatile,
        ),
        ReadOp::IndexedIndirect {
            volatile,
            address,
            index,
            destination,
        } => (
            regvals[address].wrapping_add(regvals[index]),
            destination,
            volatile,
        ),
        ReadOp::StackOffset {
            offset,
            destination,
        } => (regvals[BP].wrapping_add(offset as Word), destination, false),
    }
}

fn compute_write_address(op: WriteOp, regvals: RegMap) -> (Word, Word, bool) {
    match op {
        WriteOp::ZeroPage { address, source } => {
            ((ZERO_PAGE | (address as Word)), regvals[source], false)
        }
        WriteOp::Indirect {
            volatile,
            address,
            source,
        } => (regvals[address], regvals[source], volatile),
        WriteOp::OffsetIndirect {
            volatile,
            address,
            offset,
            source,
        } => (
            regvals[address].wrapping_add(offset as Word),
            regvals[source],
            volatile,
        ),
        WriteOp::IndexedIndirect {
            volatile,
            address,
            index,
            source,
        } => (
            regvals[address].wrapping_add(regvals[index]),
            regvals[source],
            volatile,
        ),
        WriteOp::StackOffset { offset, source } => (
            regvals[BP].wrapping_add(offset as Word),
            regvals[source],
            false,
        ),
    }
}

impl Resolver for RegisterOp {
    #[inline]
    fn execute(self, regvals: RegMap) -> ExecuteResult {
        match self {
            RegisterOp::Lbr(op) => {
                let (address, destination, volatile) = compute_read_address(op, regvals);
                ExecuteResult::ReadMemByte {
                    address,
                    destination,
                    volatile,
                }
            }
            RegisterOp::Lsr(op) => {
                let (address, destination, volatile) = compute_read_address(op, regvals);
                ExecuteResult::ReadMemShort {
                    address,
                    destination,
                    volatile,
                }
            }
            RegisterOp::Llr(op) => {
                let (address, destination, volatile) = compute_read_address(op, regvals);
                ExecuteResult::ReadMemWord {
                    address,
                    destination,
                    volatile,
                }
            }
            RegisterOp::Sbr(op) => {
                let (address, value, volatile) = compute_write_address(op, regvals);
                ExecuteResult::WriteMemByte {
                    address,
                    value: (value as Byte),
                    volatile,
                }
            }
            RegisterOp::Ssr(op) => {
                let (address, value, volatile) = compute_write_address(op, regvals);
                ExecuteResult::WriteMemShort {
                    address,
                    value: (value as Short),
                    volatile,
                }
            }
            RegisterOp::Slr(op) => {
                let (address, value, volatile) = compute_write_address(op, regvals);
                ExecuteResult::WriteMemWord {
                    address,
                    value,
                    volatile,
                }
            }
            RegisterOp::Tfr(RegOp {
                source,
                destination,
            }) => ExecuteResult::WriteReg {
                destination,
                value: regvals[source],
                zf: regvals[source] == 0,
                of: false,
                eps: false,
                nan: false,
                inf: false,
            },
            RegisterOp::Ldr(imm) => match imm {
                ImmOp::Immediate {
                    zero,
                    shift,
                    immediate,
                    destination,
                } => {
                    let (mut value, mask) = if shift == 0 {
                        (immediate as Word, 0x0000_FFFFu32)
                    } else if shift == 1 {
                        ((immediate as Word) << 16, 0xFFFF_0000u32)
                    } else {
                        (0, 0x0000_0000u32)
                    };

                    if zero {
                        ExecuteResult::WriteReg {
                            destination,
                            value,
                            zf: value == 0,
                            of: false,
                            eps: false,
                            nan: false,
                            inf: false,
                        }
                    } else {
                        value |= regvals[destination] & mask;

                        ExecuteResult::WriteReg {
                            destination,
                            value,
                            zf: value == 0,
                            of: false,
                            eps: false,
                            nan: false,
                            inf: false,
                        }
                    }
                }
                ImmOp::ZeroPageTranslate {
                    address,
                    destination,
                } => ExecuteResult::WriteReg {
                    destination,
                    value: (address as Word) | ZERO_PAGE,
                    zf: false,
                    of: false,
                    eps: false,
                    nan: false,
                    inf: false,
                },
            },

            RegisterOp::Push(reg) => ExecuteResult::WriteRegStack {
                regval: regvals[reg],
                sp: regvals[SP],
            },

            RegisterOp::Pop(reg) => ExecuteResult::ReadRegStack {
                reg,
                sp: regvals[SP],
            },
        }
    }

    #[inline]
    fn clock_requirement(self) -> usize {
        match self {
            RegisterOp::Lbr(r) | RegisterOp::Lsr(r) | RegisterOp::Llr(r) => match r {
                ReadOp::Indirect { .. } | ReadOp::ZeroPage { .. } | ReadOp::StackOffset { .. } => 1,
                _ => 2,
            },
            RegisterOp::Sbr(w) | RegisterOp::Ssr(w) | RegisterOp::Slr(w) => match w {
                WriteOp::ZeroPage { .. }
                | WriteOp::Indirect { .. }
                | WriteOp::StackOffset { .. } => 1,
                _ => 2,
            },
            RegisterOp::Push(_) | RegisterOp::Pop(_) | RegisterOp::Tfr(_) | RegisterOp::Ldr(_) => 1,
        }
    }
}
