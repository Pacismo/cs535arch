use super::{memory::MemoryResult, PipelineStage};
use crate::{Clock, Locks, Registers, Status};
use libmem::module::MemoryModule;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Default)]
pub struct Writeback {
    job: Option<MemoryResult>,
}

pub type WritebackResult = ();

impl PipelineStage for Writeback {
    type Prev = MemoryResult;

    type Next = WritebackResult;

    fn clock(
        &mut self,
        _: Clock,
        _: &mut Registers,
        _: &mut Locks,
        _: &mut dyn MemoryModule,
    ) -> Clock {
        todo!()
    }

    fn forward(&mut self, _: Status<Self::Prev>) -> Status<Self::Next> {
        todo!()
    }
}
