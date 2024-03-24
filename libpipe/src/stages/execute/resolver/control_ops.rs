use libseis::instruction_set::ControlOp;

use super::Resolver;

impl Resolver for ControlOp {
    #[inline]
    fn execute(self, regvals: crate::regmap::RegMap) -> crate::stages::execute::ExecuteResult {
        todo!()
    }

    #[inline]
    fn clock_requirement(self) -> usize {
        todo!()
    }
}
