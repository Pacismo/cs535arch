use super::Resolver;
use libseis::instruction_set::FloatingPointOp;

impl Resolver for FloatingPointOp {
    #[inline]
    fn execute(self, regvals: crate::regmap::RegMap) -> crate::stages::execute::ExecuteResult {
        todo!()
    }

    #[inline]
    fn clock_requirement(self) -> usize {
        todo!()
    }
}
