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
                sp: regvals[SP],
                bp: regvals[BP],
            },
            ControlOp::Jmp(Jump::Register(reg)) => JumpTo {
                location: regvals[reg],
            },
            ControlOp::Jmp(Jump::Relative(rel)) => JumpTo {
                location: regvals[PC].wrapping_add_signed(rel),
            },
            ControlOp::Jsr(Jump::Register(reg)) => Subroutine {
                location: regvals[reg],
                link: regvals[PC],
                sp: regvals[SP],
                bp: regvals[BP],
            },
            ControlOp::Jsr(Jump::Relative(rel)) => Subroutine {
                location: regvals[PC].wrapping_add_signed(rel),
                link: regvals[PC],
                sp: regvals[SP],
                bp: regvals[BP],
            },
            ControlOp::Jeq(Jump::Register(reg)) => {
                if regvals[ZF] == 1 {
                    JumpTo {
                        location: regvals[reg],
                    }
                } else {
                    Nop
                }
            }
            ControlOp::Jeq(Jump::Relative(rel)) => {
                if regvals[ZF] == 1 {
                    JumpTo {
                        location: regvals[PC].wrapping_add_signed(rel),
                    }
                } else {
                    Nop
                }
            }
            ControlOp::Jne(Jump::Register(reg)) => {
                if regvals[ZF] == 0 {
                    JumpTo {
                        location: regvals[reg],
                    }
                } else {
                    Nop
                }
            }
            ControlOp::Jne(Jump::Relative(rel)) => {
                if regvals[ZF] == 0 {
                    JumpTo {
                        location: regvals[PC].wrapping_add_signed(rel),
                    }
                } else {
                    Nop
                }
            }
            ControlOp::Jgt(Jump::Register(reg)) => {
                if regvals[ZF] == 0 && regvals[OF] == 1 {
                    JumpTo {
                        location: regvals[reg],
                    }
                } else {
                    Nop
                }
            }
            ControlOp::Jgt(Jump::Relative(rel)) => {
                if regvals[ZF] == 0 && regvals[OF] == 1 {
                    JumpTo {
                        location: regvals[PC].wrapping_add_signed(rel),
                    }
                } else {
                    Nop
                }
            }
            ControlOp::Jlt(Jump::Register(reg)) => {
                if regvals[ZF] == 0 && regvals[OF] == 0 {
                    JumpTo {
                        location: regvals[reg],
                    }
                } else {
                    Nop
                }
            }
            ControlOp::Jlt(Jump::Relative(rel)) => {
                if regvals[ZF] == 0 && regvals[OF] == 0 {
                    JumpTo {
                        location: regvals[PC].wrapping_add_signed(rel),
                    }
                } else {
                    Nop
                }
            }
            ControlOp::Jge(Jump::Register(reg)) => {
                if regvals[ZF] == 1 || regvals[OF] == 1 {
                    JumpTo {
                        location: regvals[reg],
                    }
                } else {
                    Nop
                }
            }
            ControlOp::Jge(Jump::Relative(rel)) => {
                if regvals[ZF] == 1 || regvals[OF] == 1 {
                    JumpTo {
                        location: regvals[PC].wrapping_add_signed(rel),
                    }
                } else {
                    Nop
                }
            }
            ControlOp::Jle(Jump::Register(reg)) => {
                if regvals[ZF] == 1 || regvals[OF] == 0 {
                    JumpTo {
                        location: regvals[reg],
                    }
                } else {
                    Nop
                }
            }
            ControlOp::Jle(Jump::Relative(rel)) => {
                if regvals[ZF] == 1 || regvals[OF] == 0 {
                    JumpTo {
                        location: regvals[PC].wrapping_add_signed(rel),
                    }
                } else {
                    Nop
                }
            }
        }
    }

    #[inline]
    fn clock_requirement(self) -> usize {
        match self {
            ControlOp::Jmp(Jump::Relative(_))
            | ControlOp::Jsr(Jump::Relative(_))
            | ControlOp::Jeq(Jump::Relative(_))
            | ControlOp::Jne(Jump::Relative(_))
            | ControlOp::Jgt(Jump::Relative(_))
            | ControlOp::Jlt(Jump::Relative(_))
            | ControlOp::Jge(Jump::Relative(_))
            | ControlOp::Jle(Jump::Relative(_)) => 2,

            _ => 1,
        }
    }
}
