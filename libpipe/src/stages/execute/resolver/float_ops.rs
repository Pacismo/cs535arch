use super::Resolver;
use crate::{regmap::RegMap, stages::execute::ExecuteResult};
use libseis::instruction_set::FloatingPointOp;

impl Resolver for FloatingPointOp {
    #[inline]
    fn execute(self, _: RegMap) -> ExecuteResult {
        todo!("Floating-point operations need to be investigated prior to implementation")
    }

    #[inline]
    fn clock_requirement(self) -> usize {
        todo!("Floating-point operations need to be investigated prior to implementation")
    }
}
