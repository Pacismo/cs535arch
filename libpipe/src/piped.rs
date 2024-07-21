//! This module contains the implementation of a pipelined processor.
//!
//! The [`Pipeline`] trait simply enables consistent interfacing.

use crate::{
    stages::{self, Clock, Status},
    ClockResult, Locks, PipelineStages,
};
use crate::{Pipeline, Registers};
use libmem::module::MemoryModule;

/// Represents a pipelined processor
#[derive(Debug)]
pub struct Pipelined {
    memory_module: Box<dyn MemoryModule + Send + Sync>,
    registers: Registers,
    locks: Locks,

    fetch: stages::Fetch,
    decode: stages::Decode,
    execute: stages::Execute,
    memory: stages::Memory,
    writeback: stages::Writeback,
}

impl Pipeline for Pipelined {
    fn clock(&mut self, clocks: usize) -> ClockResult {
        self.memory_module.clock(clocks);

        Clock::begin(
            clocks,
            &mut self.writeback,
            &mut self.registers,
            &mut self.locks,
            self.memory_module.as_mut(),
        )
        .then(
            &mut self.memory,
            &mut self.registers,
            &mut self.locks,
            self.memory_module.as_mut(),
        )
        .then(
            &mut self.execute,
            &mut self.registers,
            &mut self.locks,
            self.memory_module.as_mut(),
        )
        .then(
            &mut self.decode,
            &mut self.registers,
            &mut self.locks,
            self.memory_module.as_mut(),
        )
        .finally(
            &mut self.fetch,
            &mut self.registers,
            &mut self.locks,
            self.memory_module.as_mut(),
        );

        Status::begin(&mut self.fetch)
            .then(&mut self.decode)
            .then(&mut self.execute)
            .then(&mut self.memory)
            .finally(&mut self.writeback)
    }

    fn memory_module(&self) -> &dyn MemoryModule {
        self.memory_module.as_ref()
    }

    fn memory_module_mut(&mut self) -> &mut dyn MemoryModule {
        self.memory_module.as_mut()
    }

    fn registers(&self) -> &crate::Registers {
        &self.registers
    }

    fn registers_mut(&mut self) -> &mut Registers {
        &mut self.registers
    }

    fn stages(&self) -> PipelineStages {
        PipelineStages {
            fetch: &self.fetch,
            decode: &self.decode,
            execute: &self.execute,
            memory: &self.memory,
            writeback: &self.writeback,
        }
    }
}

impl Pipelined {
    /// Creates a new pipelined processor
    pub fn new(memory_module: Box<dyn MemoryModule + Send + Sync>) -> Self {
        Self {
            memory_module,
            registers: Default::default(),
            locks: Default::default(),
            fetch: Default::default(),
            decode: Default::default(),
            execute: Default::default(),
            memory: Default::default(),
            writeback: Default::default(),
        }
    }
}
