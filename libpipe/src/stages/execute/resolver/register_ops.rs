use super::Resolver;
use crate::{regmap::RegMap, stages::execute::ExecuteResult};
use libseis::instruction_set::{
    register::{ReadOp, WriteOp},
    RegisterOp,
};

impl Resolver for RegisterOp {
    #[inline]
    fn execute(self, regvals: RegMap) -> ExecuteResult {
        todo!()
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
            RegisterOp::Push(regs) | RegisterOp::Pop(regs) => regs.count(),
            RegisterOp::Tfr(_) | RegisterOp::Ldr(_) => 1,
        }
    }
}
