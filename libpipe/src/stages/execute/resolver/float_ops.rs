use super::Resolver;
use crate::{
    regmap::RegMap,
    stages::execute::ExecuteResult::{self, WriteReg, WriteStatus},
};
use libseis::{
    instruction_set::{
        floating_point::{BinaryOp, CheckOp, CompOp, ConversionOp, UnaryOp},
        FloatingPointOp,
    },
    types::{SWord, Word},
};

impl Resolver for FloatingPointOp {
    #[inline]
    fn execute(self, regvals: RegMap) -> ExecuteResult {
        match self {
            FloatingPointOp::Fadd(BinaryOp {
                left,
                right,
                destination,
            }) => {
                let left = f32::from_bits(regvals[left]);
                let right = f32::from_bits(regvals[right]);
                let value = left + right;

                WriteReg {
                    destination,
                    value: f32::to_bits(value),
                    zf: value.abs() == 0.0,
                    of: value.is_sign_negative(),
                    eps: value.abs() <= f32::EPSILON,
                    nan: value.is_nan(),
                    inf: value.is_infinite(),
                }
            }
            FloatingPointOp::Fsub(BinaryOp {
                left,
                right,
                destination,
            }) => {
                let left = f32::from_bits(regvals[left]);
                let right = f32::from_bits(regvals[right]);
                let value = left - right;

                WriteReg {
                    destination,
                    value: f32::to_bits(value),
                    zf: value.abs() == 0.0,
                    of: value.is_sign_negative(),
                    eps: value.abs() <= f32::EPSILON,
                    nan: value.is_nan(),
                    inf: value.is_infinite(),
                }
            }
            FloatingPointOp::Fmul(BinaryOp {
                left,
                right,
                destination,
            }) => {
                let left = f32::from_bits(regvals[left]);
                let right = f32::from_bits(regvals[right]);
                let value = left * right;

                WriteReg {
                    destination,
                    value: f32::to_bits(value),
                    zf: value.abs() == 0.0,
                    of: value.is_sign_negative(),
                    eps: value.abs() <= f32::EPSILON,
                    nan: value.is_nan(),
                    inf: value.is_infinite(),
                }
            }
            FloatingPointOp::Fdiv(BinaryOp {
                left,
                right,
                destination,
            }) => {
                let left = f32::from_bits(regvals[left]);
                let right = f32::from_bits(regvals[right]);
                let value = left / right;

                WriteReg {
                    destination,
                    value: f32::to_bits(value),
                    zf: value.abs() == 0.0,
                    of: value.is_sign_negative(),
                    eps: value.abs() <= f32::EPSILON,
                    nan: value.is_nan(),
                    inf: value.is_infinite(),
                }
            }
            FloatingPointOp::Fmod(BinaryOp {
                left,
                right,
                destination,
            }) => {
                let left = f32::from_bits(regvals[left]);
                let right = f32::from_bits(regvals[right]);
                let value = left % right;

                WriteReg {
                    destination,
                    value: f32::to_bits(value),
                    zf: value.abs() == 0.0,
                    of: value.is_sign_negative(),
                    eps: value.abs() <= f32::EPSILON,
                    nan: value.is_nan(),
                    inf: value.is_infinite(),
                }
            }
            FloatingPointOp::Fcmp(CompOp { left, right }) => {
                let left = f32::from_bits(regvals[left]);
                let right = f32::from_bits(regvals[right]);
                let value = left - right;

                WriteStatus {
                    zf: value.abs() == 0.0,
                    of: left < right,
                    eps: value.abs() <= f32::EPSILON,
                    nan: value.is_nan(),
                    inf: value.is_infinite(),
                }
            }
            FloatingPointOp::Fneg(UnaryOp {
                source,
                destination,
            }) => {
                let value = -f32::from_bits(regvals[source]);

                WriteReg {
                    destination,
                    value: f32::to_bits(value),
                    zf: value.abs() == 0.0,
                    of: value.is_sign_negative(),
                    eps: value.abs() <= f32::EPSILON,
                    nan: value.is_nan(),
                    inf: value.is_infinite(),
                }
            }
            FloatingPointOp::Frec(UnaryOp {
                source,
                destination,
            }) => {
                let value = f32::from_bits(regvals[source]).recip();

                WriteReg {
                    destination,
                    value: f32::to_bits(value),
                    zf: value.abs() == 0.0,
                    of: value.is_sign_negative(),
                    eps: value.abs() <= f32::EPSILON,
                    nan: value.is_nan(),
                    inf: value.is_infinite(),
                }
            }
            FloatingPointOp::Itof(ConversionOp {
                source,
                destination,
            }) => {
                let value = (regvals[source] as SWord) as f32;

                WriteReg {
                    destination,
                    value: f32::to_bits(value),
                    zf: value.abs() == 0.0,
                    of: value.is_sign_negative(),
                    eps: value.abs() <= f32::EPSILON,
                    nan: value.is_nan(),
                    inf: value.is_infinite(),
                }
            }
            FloatingPointOp::Ftoi(ConversionOp {
                source,
                destination,
            }) => {
                let value = (f32::from_bits(regvals[source]) as SWord) as Word;

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
            FloatingPointOp::Fchk(CheckOp(register)) => {
                let value = f32::from_bits(regvals[register]);

                WriteStatus {
                    zf: value == 0.0,
                    of: value.is_sign_negative(),
                    eps: value.abs() <= f32::EPSILON,
                    nan: value.is_nan(),
                    inf: value.is_infinite(),
                }
            }
        }
    }

    #[inline]
    fn clock_requirement(self) -> usize {
        20
    }
}
