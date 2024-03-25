mod control_ops;
mod float_ops;
mod integer_ops;
mod register_ops;

use crate::regmap::RegMap;
use libseis::instruction_set::Instruction;

/// A trait which will resolve the instruction and then execute it
///
/// This trait aims to simplify the process of executing instructions, as
/// the resolving process is now down to a set of limited match expressions
pub trait Resolver {
    fn execute(self, regvals: RegMap) -> super::ExecuteResult;

    fn clock_requirement(self) -> usize;
}

impl Resolver for Instruction {
    #[inline]
    fn execute(self, regvals: RegMap) -> super::ExecuteResult {
        match self {
            Instruction::Control(c) => c.execute(regvals),
            Instruction::Integer(i) => i.execute(regvals),
            Instruction::FloatingPoint(f) => f.execute(regvals),
            Instruction::Register(r) => r.execute(regvals),
        }
    }

    #[inline]
    fn clock_requirement(self) -> usize {
        match self {
            Instruction::Control(c) => c.clock_requirement(),
            Instruction::Integer(i) => i.clock_requirement(),
            Instruction::FloatingPoint(f) => f.clock_requirement(),
            Instruction::Register(r) => r.clock_requirement(),
        }
    }
}
