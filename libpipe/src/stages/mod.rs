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
    Self: Debug + Serialize + Default,
{
    /// What the stage takes in when forwarding a clock.
    type Prev: Debug + Serialize;
    /// What the stage outputs as a result of taking a clock.
    type Next: Debug + Serialize;

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
    /// The next stage is ready to receive
    Ready(usize),
    /// The next stage is not ready to receive
    Block(usize),
    /// The next stage ordered a squash 🍐
    Squash(usize),
}

impl Clock {
    /// The number of clocks sent
    pub fn clocks(self) -> usize {
        match self {
            Self::Ready(x) | Self::Block(x) | Self::Squash(x) => x,
        }
    }

    /// Whether the signal is a block
    pub fn is_block(self) -> bool {
        matches!(self, Self::Block(_))
    }

    /// Whether the signal is a squash
    pub fn is_squash(self) -> bool {
        matches!(self, Self::Squash(_))
    }

    /// Whether the next stage is ready
    ///
    /// True for [`Ready`](Clock::Ready) or [`Squash`](Clock::Squash) signals
    pub fn is_ready(self) -> bool {
        matches!(self, Self::Ready(_) | Self::Squash(_))
    }

    /// Transforms the clock to a block signal
    pub fn to_block(self) -> Self {
        Self::Block(self.clocks())
    }

    /// Transforms the clock to a squash signal
    pub fn to_squash(self) -> Self {
        Self::Squash(self.clocks())
    }

    /// Transforms the clock to a ready signal
    pub fn to_ready(self) -> Self {
        Self::Ready(self.clocks())
    }

    /// Creates a clock signal
    pub fn begin<T: PipelineStage>(
        self,
        clocks: usize,
        first: &mut T,
        registers: &mut Registers,
        reg_locks: &mut Locks,
        memory: &mut dyn MemoryModule,
    ) -> Self {
        first.clock(Self::Ready(clocks), registers, reg_locks, memory)
    }

    /// Forwards the previous stage's clock to the next stage
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

/// The stage of the previous stage in the pipeline
///
/// If there is a stall, [`Stall`](Status::Stall) *must* contain the shortest stall time
#[derive(Debug, Default)]
pub enum Status<T: Debug = ()> {
    /// Some previous stage has a stall
    ///
    /// The contained value is the minimum number of clocks required
    /// to clear the shortest stall
    Stall(usize),
    /// The pipeline has completed a job and is forwarding a new job
    Flow(T),
    /// The stage is ready, but waiting
    #[default]
    Ready,
    /// The pipeline squashed an instruction
    Squashed,
    /// There are no new jobs
    Dry,
}

impl<T: Debug> Status<T> {
    pub fn is_stall(&self) -> bool {
        matches!(self, Self::Stall(..))
    }

    pub fn is_flow(&self) -> bool {
        matches!(self, Self::Flow(..))
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

    /// Starts the process of forwarding results from the previous clock
    pub fn begin<P: PipelineStage<Prev = ()>>(stage: &mut P) -> Status<P::Next> {
        stage.forward(Status::default())
    }

    /// Forwards any information about the state, if available
    pub fn then<P: PipelineStage<Prev = T>>(self, stage: &mut P) -> Status<P::Next> {
        stage.forward(self)
    }

    /// Ends the stage-forwarding, possibly returning the number of clocks needed to clear any blockages
    pub fn end<P: PipelineStage<Next = (), Prev = T>>(self, stage: &mut P) -> Option<usize> {
        match stage.forward(self) {
            Status::Stall(n) => Some(n),
            Status::Dry => None,
            _ => Some(1),
        }
    }
}
