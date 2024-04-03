use super::{memory::MemoryResult, PipelineStage};
use crate::{Clock, Locks, Registers, Status};
use libmem::module::MemoryModule;
use libseis::registers::{BP, EPS, INF, LP, NAN, OF, PC, SP, ZF};
use serde::Serialize;
use std::mem::take;

#[derive(Debug, Clone, Serialize, Default)]
pub struct Writeback {
    job: Option<MemoryResult>,
}

pub type WritebackResult = ();

impl PipelineStage for Writeback {
    type Prev = MemoryResult;
    type Next = WritebackResult;
    type State = Option<MemoryResult>;

    fn clock(
        &mut self,
        clock: Clock,
        registers: &mut Registers,
        locks: &mut Locks,
        _: &mut dyn MemoryModule,
    ) -> Clock {
        if let Some(job) = take(&mut self.job) {
            match job {
                MemoryResult::Nop | MemoryResult::Halt => {}
                MemoryResult::Squashed { wregs } | MemoryResult::Ignore { wregs } => {
                    for reg in wregs {
                        locks[reg] -= 1;
                    }
                }
                MemoryResult::WriteStatus {
                    zf,
                    of,
                    eps,
                    nan,
                    inf,
                } => {
                    registers[ZF] = zf.then_some(1).unwrap_or(0);
                    registers[OF] = of.then_some(1).unwrap_or(0);
                    registers[EPS] = eps.then_some(1).unwrap_or(0);
                    registers[NAN] = nan.then_some(1).unwrap_or(0);
                    registers[INF] = inf.then_some(1).unwrap_or(0);

                    locks[ZF] -= 1;
                    locks[OF] -= 1;
                    locks[EPS] -= 1;
                    locks[NAN] -= 1;
                    locks[INF] -= 1;
                }
                MemoryResult::WriteRegNoStatus { destination, value } => {
                    registers[destination] = value;
                }
                MemoryResult::WriteReg1 {
                    destination: register,
                    value,

                    zf,
                    of,
                    eps,
                    nan,
                    inf,
                } => {
                    registers[register] = value;
                    registers[ZF] = zf.then_some(1).unwrap_or(0);
                    registers[OF] = of.then_some(1).unwrap_or(0);
                    registers[EPS] = eps.then_some(1).unwrap_or(0);
                    registers[NAN] = nan.then_some(1).unwrap_or(0);
                    registers[INF] = inf.then_some(1).unwrap_or(0);

                    locks[register] -= 1;
                    locks[ZF] -= 1;
                    locks[OF] -= 1;
                    locks[EPS] -= 1;
                    locks[NAN] -= 1;
                    locks[INF] -= 1;
                }
                MemoryResult::WriteReg2 {
                    register,
                    value,
                    sp,
                    zf,
                    of,
                    eps,
                    nan,
                    inf,
                } => {
                    registers[register] = value;
                    registers[SP] = sp;
                    registers[ZF] = zf.then_some(1).unwrap_or(0);
                    registers[OF] = of.then_some(1).unwrap_or(0);
                    registers[EPS] = eps.then_some(1).unwrap_or(0);
                    registers[NAN] = nan.then_some(1).unwrap_or(0);
                    registers[INF] = inf.then_some(1).unwrap_or(0);

                    locks[register] -= 1;
                    locks[SP] -= 1;
                    locks[ZF] -= 1;
                    locks[OF] -= 1;
                    locks[EPS] -= 1;
                    locks[NAN] -= 1;
                    locks[INF] -= 1;
                }
                MemoryResult::JumpSubroutine {
                    address,
                    link,
                    sp,
                    bp,
                } => {
                    registers[PC] = address;
                    registers[LP] = link;
                    registers[SP] = sp;
                    registers[BP] = bp;

                    locks[PC] -= 1;
                    locks[LP] -= 1;
                    locks[SP] -= 1;
                    locks[BP] -= 1;
                }
                MemoryResult::Jump { address } => {
                    registers[PC] = address;

                    locks[PC] -= 1;
                }
                MemoryResult::Return {
                    address,
                    bp,
                    sp,
                    lp,
                } => {
                    registers[PC] = address;
                    registers[BP] = bp;
                    registers[SP] = sp;
                    registers[LP] = lp;

                    locks[PC] -= 1;
                    locks[BP] -= 1;
                    locks[SP] -= 1;
                    locks[LP] -= 1;
                }
            }
        }

        clock.to_ready()
    }

    fn forward(&mut self, input: Status<Self::Prev>) -> Status<Self::Next> {
        match input {
            Status::Stall(clocks) => Status::Stall(clocks),
            Status::Flow(input) => {
                self.job = Some(input);
                Status::Flow(())
            }
            Status::Ready(_) => Status::Stall(1),
            Status::Squashed => Status::Squashed,
            Status::Dry => Status::Dry,
        }
    }

    fn get_state(&self) -> &Self::State {
        &self.job
    }
}
