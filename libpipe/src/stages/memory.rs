use crate::{Clock, Locks, PipelineStage, Registers, Status};

use super::execute::ExecuteResult;
use libmem::module::MemoryModule;
use libseis::{
    registers::RegisterFlags,
    types::{Register, Word},
};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Default)]
enum State {
    #[default]
    Idle,
    Waiting {
        operation: ExecuteResult,
        clocks: usize,
    },
    Ready {
        result: MemoryResult,
    },
    Squashed {
        wregs: RegisterFlags,
    },
}

#[derive(Debug, Clone, Serialize)]
pub enum MemoryResult {
    Nop,
    Squashed {
        wregs: RegisterFlags,
    },
    WriteReg1 {
        register: Register,
        value: Word,

        zf: bool,
        of: bool,
        eps: bool,
        nan: bool,
        inf: bool,
    },
    WriteReg2 {
        register: Register,
        value: Word,
        sp: Word,

        zf: bool,
        of: bool,
        eps: bool,
        nan: bool,
        inf: bool,
    },
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Memory {
    state: State,
    forward: Option<MemoryResult>,
}

impl PipelineStage for Memory {
    type Prev = ExecuteResult;

    type Next = MemoryResult;

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
