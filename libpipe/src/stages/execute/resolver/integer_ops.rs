use super::Resolver;
use crate::{regmap::RegMap, stages::execute::ExecuteResult};
use libseis::{
    instruction_set::integer::{
        BinaryOp as B, CompOp as C, SignExtendOp as S, TestOp as T, UnaryOp as U,
    },
    types::{SWord, Word},
};
use libseis::{instruction_set::IntegerOp, types::Register};
use ExecuteResult::*;

#[inline]
fn add(l: Word, r: Word, destination: Register) -> ExecuteResult {
    let (value, of) = l.overflowing_add(r);

    WriteReg {
        destination,
        value,
        zf: value == 0,
        of,
        eps: false,
        nan: false,
        inf: false,
    }
}

#[inline]
fn sub(l: Word, r: Word, destination: Register) -> ExecuteResult {
    let (value, of) = l.overflowing_sub(r);

    WriteReg {
        destination,
        value,
        zf: value == 0,
        of,
        eps: false,
        nan: false,
        inf: false,
    }
}

#[inline]
fn mul(l: Word, r: Word, destination: Register) -> ExecuteResult {
    let (value, of) = l.overflowing_mul(r);

    WriteReg {
        destination,
        value,
        zf: value == 0,
        of,
        eps: false,
        nan: false,
        inf: false,
    }
}

#[inline]
fn div_unsigned(l: Word, r: Word, destination: Register) -> ExecuteResult {
    let (value, of) = l.overflowing_div(r);

    WriteReg {
        destination,
        value,
        zf: value == 0,
        of,
        eps: false,
        nan: false,
        inf: false,
    }
}

#[inline]
fn div_signed(l: Word, r: Word, destination: Register) -> ExecuteResult {
    let (value, of) = (l as SWord).overflowing_div(r as SWord);
    WriteReg {
        destination,
        value: value as Word,
        zf: value == 0,
        of,
        eps: false,
        nan: false,
        inf: false,
    }
}

#[inline]
fn modulo(l: Word, r: Word, destination: Register) -> ExecuteResult {
    let (value, of) = l.overflowing_rem(r);

    WriteReg {
        destination,
        value,
        zf: value == 0,
        of,
        eps: false,
        nan: false,
        inf: false,
    }
}

#[inline]
fn cmp(l: Word, r: Word, signed: bool) -> ExecuteResult {
    if signed {
        let l = l as SWord;
        let r = r as SWord;

        let value = l.wrapping_sub(r);
        WriteStatus {
            zf: value == 0,
            of: r > l,
            eps: false,
            nan: false,
            inf: false,
        }
    } else {
        let (value, overflow) = l.overflowing_sub(r);

        WriteStatus {
            zf: value == 0,
            of: overflow,
            eps: false,
            nan: false,
            inf: false,
        }
    }
}

#[inline]
fn tst(l: Word, r: Word) -> ExecuteResult {
    WriteStatus {
        zf: l & r == l,
        of: false,
        eps: false,
        nan: false,
        inf: false,
    }
}

#[inline]
fn and(l: Word, r: Word, destination: Register) -> ExecuteResult {
    let value = l & r;

    WriteReg {
        destination,
        value,
        zf: value == 0,
        of: false,
        eps: false,
        nan: false,
        inf: false,
    }
}

#[inline]
fn ior(l: Word, r: Word, destination: Register) -> ExecuteResult {
    let value = l | r;

    WriteReg {
        destination,
        value,
        zf: value == 0,
        of: false,
        eps: false,
        nan: false,
        inf: false,
    }
}

#[inline]
fn xor(l: Word, r: Word, destination: Register) -> ExecuteResult {
    let value = l ^ r;

    WriteReg {
        destination,
        value,
        zf: value == 0,
        of: false,
        eps: false,
        nan: false,
        inf: false,
    }
}

#[inline]
fn bsl(l: Word, r: Word, destination: Register) -> ExecuteResult {
    let (value, of) = l.overflowing_shl(r);

    WriteReg {
        destination,
        value,
        zf: value == 0,
        of,
        eps: false,
        nan: false,
        inf: false,
    }
}

#[inline]
fn bsr(l: Word, r: Word, destination: Register) -> ExecuteResult {
    let (value, of) = l.overflowing_shr(r);

    WriteReg {
        destination,
        value,
        zf: value == 0,
        of,
        eps: false,
        nan: false,
        inf: false,
    }
}

#[inline]
fn asr(l: Word, r: Word, destination: Register) -> ExecuteResult {
    let (value, of) = (l as SWord).overflowing_shr(r);

    WriteReg {
        destination,
        value: value as Word,
        zf: value == 0,
        of,
        eps: false,
        nan: false,
        inf: false,
    }
}

#[inline]
fn rol(l: Word, r: Word, destination: Register) -> ExecuteResult {
    WriteReg {
        destination,
        value: l.rotate_left(r),
        zf: false,
        of: false,
        eps: false,
        nan: false,
        inf: false,
    }
}

#[inline]
fn ror(l: Word, r: Word, destination: Register) -> ExecuteResult {
    WriteReg {
        destination,
        value: l.rotate_right(r),
        zf: false,
        of: false,
        eps: false,
        nan: false,
        inf: false,
    }
}

impl Resolver for IntegerOp {
    #[inline]
    fn execute(self, regvals: RegMap) -> ExecuteResult {
        match self {
            IntegerOp::Add(B::Immediate(operand, immediate, destination)) => {
                add(regvals[operand], immediate, destination)
            }
            IntegerOp::Add(B::Registers(operand, optional, destination)) => {
                add(regvals[operand], regvals[optional], destination)
            }

            IntegerOp::Sub(B::Immediate(operand, immediate, destination)) => {
                sub(regvals[operand], immediate, destination)
            }
            IntegerOp::Sub(B::Registers(operand, optional, destination)) => {
                sub(regvals[operand], regvals[optional], destination)
            }

            IntegerOp::Mul(B::Immediate(operand, immediate, destination)) => {
                mul(regvals[operand], immediate, destination)
            }
            IntegerOp::Mul(B::Registers(operand, optional, destination)) => {
                mul(regvals[operand], regvals[optional], destination)
            }

            IntegerOp::Dvu(B::Immediate(operand, immediate, destination)) => {
                div_unsigned(regvals[operand], immediate, destination)
            }
            IntegerOp::Dvu(B::Registers(operand, optional, destination)) => {
                div_unsigned(regvals[operand], regvals[optional], destination)
            }

            IntegerOp::Dvs(B::Immediate(operand, immediate, destination)) => {
                div_signed(regvals[operand], immediate, destination)
            }
            IntegerOp::Dvs(B::Registers(operand, optional, destination)) => {
                div_signed(regvals[operand], regvals[optional], destination)
            }

            IntegerOp::Mod(B::Immediate(operand, immediate, destination)) => {
                modulo(regvals[operand], immediate, destination)
            }
            IntegerOp::Mod(B::Registers(operand, optional, destination)) => {
                modulo(regvals[operand], regvals[optional], destination)
            }

            IntegerOp::Cmp(C::Immediate(operand, immediate, signed)) => {
                cmp(regvals[operand], immediate, signed)
            }
            IntegerOp::Cmp(C::Registers(operand, optional, signed)) => {
                cmp(regvals[operand], regvals[optional], signed)
            }

            IntegerOp::Tst(T::Immediate(operand, immediate)) => tst(regvals[operand], immediate),
            IntegerOp::Tst(T::Registers(operand, optional)) => {
                tst(regvals[operand], regvals[optional])
            }

            IntegerOp::And(B::Immediate(operand, immediate, destination)) => {
                and(regvals[operand], immediate, destination)
            }
            IntegerOp::And(B::Registers(operand, optional, destination)) => {
                and(regvals[operand], regvals[optional], destination)
            }

            IntegerOp::Ior(B::Immediate(operand, immediate, destination)) => {
                ior(regvals[operand], immediate, destination)
            }
            IntegerOp::Ior(B::Registers(operand, optional, destination)) => {
                ior(regvals[operand], regvals[optional], destination)
            }

            IntegerOp::Xor(B::Immediate(operand, immediate, destination)) => {
                xor(regvals[operand], immediate, destination)
            }
            IntegerOp::Xor(B::Registers(operand, optional, destination)) => {
                xor(regvals[operand], regvals[optional], destination)
            }

            IntegerOp::Not(U(operand, destination)) => {
                let value = !regvals[operand];

                WriteReg {
                    destination,
                    value,
                    zf: value == 0,
                    of: false,
                    eps: false,
                    nan: false,
                    inf: false,
                }
            }

            IntegerOp::Sxt(S(width, destination)) => {
                let mut value = regvals[destination];
                if width == 0 {
                    if value & 0b1000_0000 != 0 {
                        value |= 0xFFFF_FF00;

                        WriteReg {
                            destination,
                            value,
                            zf: false,
                            of: true,
                            eps: false,
                            nan: false,
                            inf: false,
                        }
                    } else {
                        WriteReg {
                            destination,
                            value,
                            zf: value == 0,
                            of: false,
                            eps: false,
                            nan: false,
                            inf: false,
                        }
                    }
                } else if width == 1 {
                    if value & 0b1000_0000_0000_0000 != 0 {
                        value |= 0xFFFF_0000;

                        WriteReg {
                            destination,
                            value,
                            zf: false,
                            of: true,
                            eps: false,
                            nan: false,
                            inf: false,
                        }
                    } else {
                        WriteReg {
                            destination,
                            value,
                            zf: value == 0,
                            of: false,
                            eps: false,
                            nan: false,
                            inf: false,
                        }
                    }
                } else {
                    Nop
                }
            }

            IntegerOp::Bsl(B::Immediate(operand, immediate, destination)) => {
                bsl(regvals[operand], immediate, destination)
            }
            IntegerOp::Bsl(B::Registers(operand, optional, destination)) => {
                bsl(regvals[operand], regvals[optional], destination)
            }

            IntegerOp::Bsr(B::Immediate(operand, immediate, destination)) => {
                bsr(regvals[operand], immediate, destination)
            }
            IntegerOp::Bsr(B::Registers(operand, optional, destination)) => {
                bsr(regvals[operand], regvals[optional], destination)
            }

            IntegerOp::Asr(B::Immediate(operand, immediate, destination)) => {
                asr(regvals[operand], immediate, destination)
            }
            IntegerOp::Asr(B::Registers(operand, optional, destination)) => {
                asr(regvals[operand], regvals[optional], destination)
            }

            IntegerOp::Rol(B::Immediate(operand, immediate, destination)) => {
                rol(regvals[operand], immediate, destination)
            }
            IntegerOp::Rol(B::Registers(operand, optional, destination)) => {
                rol(regvals[operand], regvals[optional], destination)
            }

            IntegerOp::Ror(B::Immediate(operand, immediate, destination)) => {
                ror(regvals[operand], immediate, destination)
            }
            IntegerOp::Ror(B::Registers(operand, optional, destination)) => {
                ror(regvals[operand], regvals[optional], destination)
            }
        }
    }

    #[inline]
    fn clock_requirement(self) -> usize {
        1
    }
}
