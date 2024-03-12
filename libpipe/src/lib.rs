//! Pipelining traits. This provides the necessary means of
//! implementing a processor.
//!
//! The expectation is that the processor will implement the
//! fetch, decode, execute, memory, and writeback stages.

use libmem::module::MemoryModule;
use std::fmt::Debug;

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
    fn clock(&mut self) -> ClockResult;

    fn memory_module(&self) -> &dyn MemoryModule;
}
