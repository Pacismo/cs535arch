use crate::regmap::RegMap;
use super::Resolver;
use libseis::instruction_set::FloatingPointOp;

impl Resolver for FloatingPointOp {
    #[inline]
    fn execute(self, regvals: RegMap) -> crate::stages::execute::ExecuteResult {
        todo!()
    }

    #[inline]
    fn clock_requirement(self) -> usize {
        todo!()
    }
}
