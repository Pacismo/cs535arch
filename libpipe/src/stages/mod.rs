//! Represents the various stages of the processor pipeline.
//!
//! Each stage is defined distinctly from the pipeline to handle
//! the various tasks the pipeline is responsible for.
//!
//! This ensures that the stages may be consistently serialized for
//! display to the user in the frontend.

use crate::{reg_locks::Locks, Registers};
pub use fetch::Fetch;
use libmem::module::MemoryModule;
use serde::Serialize;
use std::fmt::Debug;

mod decode;
mod execute;
mod fetch;
mod memory;
mod writeback;

/// Trait representing a pipeline stage.
///
/// Must be implemented by each stage. Not object-safe.
///
/// Presents interfaces for each stage to perform the necessary processing.
pub trait PipelineStage
where
    Self: Debug + Serialize,
{
    /// What the stage takes in when forwarding a clock.
    type Prev: Debug;
    /// What the stage outputs as a result of taking a clock.
    type Next: Debug;

    /// Called when the clock sends a new clock.
    ///
    /// ## Parameters
    ///
    /// - `clocks`: The number of clocks being sent
    /// - `next_stalled`: Whether the next stage has been stalled
    /// - `registers`: The register-set of the processor
    /// - `memory`: The [memory module](MemoryModule) of the system
    ///
    /// ## Returns
    ///
    /// `true` if the stage is blocked
    /// `false` if the stage is not blocked
    fn clock(
        &mut self,
        clock: Clock,
        registers: &mut Registers,
        reg_locks: &mut Locks,
        memory: &mut dyn MemoryModule,
    ) -> Clock;

    /// Called after all stages have been clocked.
    fn forward(&mut self, input: Status<Self::Prev>) -> Status<Self::Next>;
}

#[derive(Debug, Clone, Copy)]
pub enum Clock {
    Ready(usize),
    Block(usize),
    Squash(usize),
}

impl Clock {
    pub fn clocks(self) -> usize {
        match self {
            Self::Ready(x) | Self::Block(x) | Self::Squash(x) => x,
        }
    }

    pub fn is_block(self) -> bool {
        matches!(self, Self::Block(_))
    }

    pub fn is_squash(self) -> bool {
        matches!(self, Self::Squash(_))
    }

    pub fn is_flow(self) -> bool {
        matches!(self, Self::Ready(_))
    }

    pub fn to_block(self) -> Self {
        Self::Block(self.clocks())
    }

    pub fn to_squash(self) -> Self {
        Self::Squash(self.clocks())
    }

    pub fn to_flow(self) -> Self {
        Self::Ready(self.clocks())
    }

    pub fn then<T: PipelineStage>(
        self,
        next: &mut T,
        registers: &mut Registers,
        reg_locks: &mut Locks,
        memory: &mut dyn MemoryModule,
    ) -> Self {
        next.clock(self, registers, reg_locks, memory)
    }
}

/// The stage of the previous stage in the pipeline.
#[derive(Debug)]
pub enum Status<T: Debug> {
    /// The pipeline has a stall
    Stall(usize),
    /// The pipeline has completed a job and is forwarding a new job
    Flow(T),
    /// The stage is ready, but waiting
    Ready,
    /// The pipeline squashed an instruction
    Squashed,
    /// There are no new jobs
    Dry,
}

impl<T: Debug> Status<T> {
    pub fn is_stall(&self) -> bool {
        matches!(self, Self::Stall(_))
    }

    pub fn is_flow(&self) -> bool {
        matches!(self, Self::Flow(_))
    }

    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready)
    }

    pub fn is_squashed(&self) -> bool {
        matches!(self, Self::Squashed)
    }

    pub fn is_dry(&self) -> bool {
        matches!(self, Self::Dry)
    }

    pub fn then<P: PipelineStage<Prev = T>>(self, stage: &mut P) -> Status<P::Next> {
        stage.forward(self)
    }
}