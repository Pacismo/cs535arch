use crate::{Clock, Locks, Registers, Status};

use super::{memory::MemoryResult, PipelineStage};
use libmem::module::MemoryModule;
use serde::Serialize;

#[derive(Debug, Serialize, Default)]
pub struct Writeback {}

pub type WritebackResult = ();

impl PipelineStage for Writeback {
    type Prev = MemoryResult;

    type Next = WritebackResult;

    fn clock(
        &mut self,
        clock: Clock,
        registers: &mut Registers,
        reg_locks: &mut Locks,
        memory: &mut dyn MemoryModule,
    ) -> Clock {
        todo!()
    }

    fn forward(&mut self, input: Status<Self::Prev>) -> Status<Self::Next> {
        todo!()
    }
}
