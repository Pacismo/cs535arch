use super::Resolver;
use crate::{regmap::RegMap, stages::execute::ExecuteResult};
use libseis::{
    instruction_set::{control::Jump, ControlOp},
    registers::{BP, LP, OF, PC, SP, ZF},
};

use ExecuteResult::*;

impl Resolver for ControlOp {
    #[inline]
    fn execute(self, regvals: RegMap) -> ExecuteResult {
        match self {
            ControlOp::Nop => Nop,
            ControlOp::Halt => Halt,
            ControlOp::Ret => Return {
                link: regvals[LP],
                bp: regvals[BP],
            },
            ControlOp::Jmp(Jump::Register(reg)) => JumpTo {
                address: regvals[reg],
            },
            ControlOp::Jmp(Jump::Relative(rel)) => JumpTo {
                address: regvals[PC].wrapping_add_signed(rel),
            },
            ControlOp::Jsr(Jump::Register(reg)) => Subroutine {
                address: regvals[reg],
                link: regvals[PC].wrapping_add(4),
                lp: regvals[LP],
                sp: regvals[SP],
                bp: regvals[BP],
            },
            ControlOp::Jsr(Jump::Relative(rel)) => Subroutine {
                address: regvals[PC].wrapping_add_signed(rel),
                link: regvals[PC].wrapping_add(4),
                lp: regvals[LP],
                sp: regvals[SP],
                bp: regvals[BP],
            },
            ControlOp::Jeq(Jump::Register(reg)) => {
                if regvals[ZF] == 1 {
                    JumpTo {
                        address: regvals[reg],
                    }
                } else {
                    Ignore { regs: [PC].into() }
                }
            }
            ControlOp::Jeq(Jump::Relative(rel)) => {
                if regvals[ZF] == 1 {
                    JumpTo {
                        address: regvals[PC].wrapping_add_signed(rel),
                    }
                } else {
                    Ignore { regs: [PC].into() }
                }
            }
            ControlOp::Jne(Jump::Register(reg)) => {
                if regvals[ZF] == 0 {
                    JumpTo {
                        address: regvals[reg],
                    }
                } else {
                    Ignore { regs: [PC].into() }
                }
            }
            ControlOp::Jne(Jump::Relative(rel)) => {
                if regvals[ZF] == 0 {
                    JumpTo {
                        address: regvals[PC].wrapping_add_signed(rel),
                    }
                } else {
                    Ignore { regs: [PC].into() }
                }
            }
            ControlOp::Jgt(Jump::Register(reg)) => {
                if regvals[ZF] == 0 && regvals[OF] == 0 {
                    JumpTo {
                        address: regvals[reg],
                    }
                } else {
                    Ignore { regs: [PC].into() }
                }
            }
            ControlOp::Jgt(Jump::Relative(rel)) => {
                if regvals[ZF] == 0 && regvals[OF] == 0 {
                    JumpTo {
                        address: regvals[PC].wrapping_add_signed(rel),
                    }
                } else {
                    Ignore { regs: [PC].into() }
                }
            }
            ControlOp::Jlt(Jump::Register(reg)) => {
                if regvals[ZF] == 0 && regvals[OF] == 1 {
                    JumpTo {
                        address: regvals[reg],
                    }
                } else {
                    Ignore { regs: [PC].into() }
                }
            }
            ControlOp::Jlt(Jump::Relative(rel)) => {
                if regvals[ZF] == 0 && regvals[OF] == 1 {
                    JumpTo {
                        address: regvals[PC].wrapping_add_signed(rel),
                    }
                } else {
                    Ignore { regs: [PC].into() }
                }
            }
            ControlOp::Jge(Jump::Register(reg)) => {
                if regvals[ZF] == 1 || regvals[OF] == 0 {
                    JumpTo {
                        address: regvals[reg],
                    }
                } else {
                    Ignore { regs: [PC].into() }
                }
            }
            ControlOp::Jge(Jump::Relative(rel)) => {
                if regvals[ZF] == 1 || regvals[OF] == 0 {
                    JumpTo {
                        address: regvals[PC].wrapping_add_signed(rel),
                    }
                } else {
                    Ignore { regs: [PC].into() }
                }
            }
            ControlOp::Jle(Jump::Register(reg)) => {
                if regvals[ZF] == 1 || regvals[OF] == 1 {
                    JumpTo {
                        address: regvals[reg],
                    }
                } else {
                    Ignore { regs: [PC].into() }
                }
            }
            ControlOp::Jle(Jump::Relative(rel)) => {
                if regvals[ZF] == 1 || regvals[OF] == 1 {
                    JumpTo {
                        address: regvals[PC].wrapping_add_signed(rel),
                    }
                } else {
                    Ignore { regs: [PC].into() }
                }
            }
        }
    }

    #[inline]
    fn clock_requirement(self) -> usize {
        1
    }
}
