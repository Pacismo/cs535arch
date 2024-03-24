use libseis::instruction_set::IntegerOp;
use super::Resolver;

impl Resolver for IntegerOp {
    #[inline]
    fn execute(self, regvals: crate::regmap::RegMap) -> crate::stages::execute::ExecuteResult {
        todo!()
    }

    #[inline]
    fn clock_requirement(self) -> usize {
        todo!()
    }
}
