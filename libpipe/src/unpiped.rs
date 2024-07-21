//! This module contains the implementation of a non-pipelined processor.
//!
//! The [`Pipeline`] trait simply enables consistent interfacing.

use crate::{
    stages::{self, Clock, PipelineStage, Status},
    ClockResult, Locks, PipelineStages,
};
use crate::{Pipeline, Registers};
use libmem::module::MemoryModule;

/// The current stage of the pipeline
#[derive(Debug, Default, Clone)]
enum Stage {
    #[default]
    Fetch,
    Decode,
    Execute,
    Memory,
    Writeback,
}
use Stage::*;

/// Represents an unpipelined processor
#[derive(Debug)]
pub struct Unpipelined {
    memory_module: Box<dyn MemoryModule + Send + Sync>,
    registers: Registers,
    locks: Locks,

    stage: Stage,

    fetch: stages::Fetch,
    decode: stages::Decode,
    execute: stages::Execute,
    memory: stages::Memory,
    writeback: stages::Writeback,
}

impl Pipeline for Unpipelined {
    fn clock(&mut self, clocks: usize) -> ClockResult {
        self.memory_module.clock(clocks);

        match self.stage {
            Stage::Fetch => {
                self.fetch.clock(
                    Clock::Ready(clocks),
                    &mut self.registers,
                    &mut self.locks,
                    self.memory_module.as_mut(),
                );

                match self.fetch.forward(Status::Ready(0)) {
                    Status::Stall(k) => ClockResult::Stall(k),
                    Status::Flow(r) => {
                        self.stage = Decode;
                        self.decode.forward(Status::Flow(r));
                        ClockResult::Flow
                    }
                    Status::Ready(_) => ClockResult::Stall(1),
                    Status::Squashed => ClockResult::Stall(1),
                    Status::Dry => ClockResult::Dry,
                }
            }
            Stage::Decode => {
                self.decode.clock(
                    Clock::Ready(clocks),
                    &mut self.registers,
                    &mut self.locks,
                    self.memory_module.as_mut(),
                );

                match self.decode.forward(Status::Ready(0)) {
                    Status::Stall(k) => ClockResult::Stall(k),
                    Status::Flow(r) => {
                        self.stage = Execute;
                        self.execute.forward(Status::Flow(r));
                        ClockResult::Flow
                    }
                    Status::Ready(_) => ClockResult::Stall(1),
                    Status::Squashed => ClockResult::Stall(1),
                    Status::Dry => ClockResult::Dry,
                }
            }
            Stage::Execute => {
                self.execute.clock(
                    Clock::Ready(clocks),
                    &mut self.registers,
                    &mut self.locks,
                    self.memory_module.as_mut(),
                );

                match self.execute.forward(Status::Ready(0)) {
                    Status::Stall(k) => ClockResult::Stall(k),
                    Status::Flow(r) => {
                        self.stage = Memory;
                        self.memory.forward(Status::Flow(r));
                        ClockResult::Flow
                    }
                    Status::Ready(_) => ClockResult::Stall(1),
                    Status::Squashed => ClockResult::Stall(1),
                    Status::Dry => ClockResult::Dry,
                }
            }
            Stage::Memory => {
                self.memory.clock(
                    Clock::Ready(clocks),
                    &mut self.registers,
                    &mut self.locks,
                    self.memory_module.as_mut(),
                );

                match self.memory.forward(Status::Ready(0)) {
                    Status::Stall(k) => ClockResult::Stall(k),
                    Status::Flow(r) => {
                        self.stage = Writeback;
                        self.writeback.forward(Status::Flow(r));
                        ClockResult::Flow
                    }
                    Status::Ready(_) => ClockResult::Stall(1),
                    Status::Squashed => ClockResult::Stall(1),
                    Status::Dry => ClockResult::Dry,
                }
            }
            Stage::Writeback => {
                self.writeback.clock(
                    Clock::Ready(clocks),
                    &mut self.registers,
                    &mut self.locks,
                    self.memory_module.as_mut(),
                );

                self.stage = Fetch;

                match self.writeback.forward(Status::Ready(0)) {
                    Status::Stall(k) => ClockResult::Stall(k),
                    Status::Flow(()) => ClockResult::Flow,
                    Status::Ready(_) => ClockResult::Stall(1),
                    Status::Squashed => ClockResult::Stall(1),
                    Status::Dry => ClockResult::Dry,
                }
            }
        }
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

impl Unpipelined {
    /// Create a new unpipelined processor
    pub fn new(memory_module: Box<dyn MemoryModule + Send + Sync>) -> Self {
        Self {
            memory_module,
            registers: Default::default(),
            locks: Default::default(),
            stage: Default::default(),
            fetch: Default::default(),
            decode: Default::default(),
            execute: Default::default(),
            memory: Default::default(),
            writeback: Default::default(),
        }
    }
}
