//! Pipelining traits. This provides the necessary means of
//! implementing a processor.
//!
//! The expectation is that the processor will implement the
//! fetch, decode, execute, memory, and writeback stages.

mod piped;
mod reg_locks;
mod registers;
mod regmap;
mod stages;
mod unpiped;

use libmem::module::MemoryModule;
pub use reg_locks::Locks;
pub use registers::Registers;
pub use stages::*;
use std::fmt::Debug;
pub use unpiped::Unpipelined;
pub use piped::Pipelined;

pub struct PipelineStages<'a> {
    pub fetch: &'a Fetch,
    pub decode: &'a Decode,
    pub execute: &'a Execute,
    pub memory: &'a Memory,
    pub writeback: &'a Writeback,
}

/// The result of clocking the pipeline.
pub enum ClockResult {
    /// A stall occurred in the pipeline
    ///
    /// The contained value is the number of clocks required for the
    /// shortest stall to pass
    Stall(usize),

    /// The pipeline has no stalls
    Flow,

    /// The pipeline is bone-dry, implying a [`Halt`](libseis::instruction_set::control::ControlOp::Halt)
    /// was executed
    Dry,
}

/// Represents a processor pipeline. Clocking the processor will yield
/// a [`ClockResult`].
pub trait Pipeline: Debug {
    /// Applies a clock on the pipeline
    fn clock(&mut self, amount: usize) -> ClockResult;

    /// Gets the memory module in the pipeline
    fn memory_module(&self) -> &dyn MemoryModule;

    /// Gets the registers in the pipeline
    fn registers(&self) -> &Registers;

    /// Gets references to the pipeline stages
    fn stages(&self) -> PipelineStages;
}
