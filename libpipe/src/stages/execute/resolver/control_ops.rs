use super::Resolver;
use crate::{regmap::RegMap, stages::execute::ExecuteResult};
use libmem::module::MemoryModule;
use libseis::{instruction_set::ControlOp, registers::BP};

impl Resolver for ControlOp {
    #[inline]
    fn execute(self, regvals: RegMap) -> ExecuteResult {
        todo!()
    }

    #[inline]
    fn clock_requirement(self, mem: &dyn MemoryModule, regvals: &RegMap) -> usize {
        match self {
            ControlOp::Nop => 1,
            ControlOp::Halt => 1,
            ControlOp::Ret => {
                1 + if mem.data_cache().check_address(regvals[BP]).is_hit() {
                    0
                } else {
                    1
                }
            }
            ControlOp::Jmp(_) => todo!(),
            ControlOp::Jsr(_) => todo!(),
            ControlOp::Jeq(_) => todo!(),
            ControlOp::Jne(_) => todo!(),
            ControlOp::Jgt(_) => todo!(),
            ControlOp::Jlt(_) => todo!(),
            ControlOp::Jge(_) => todo!(),
            ControlOp::Jle(_) => todo!(),
        }
    }
}
