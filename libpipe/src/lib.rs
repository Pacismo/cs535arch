//! Pipelining traits. This provides the necessary means of
//! implementing a processor.
//!
//! The expectation is that the processor will implement the
//! fetch, decode, execute, memory, and writeback stages.

mod registers;
mod stages;
mod unpiped;
mod reg_locks;

use libmem::module::MemoryModule;
pub use libser::{CompactJson, PrettyJson, Serializable};
pub use registers::Registers;
pub use stages::*;
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
///
/// Implementation requires the [`Serializable`] trait to be implemented for
/// both *pretty* and *compact* JSON. This may be trivially acheived by deriving
/// the [`serde::Serialize`] trait and calling the [`serialize`](serde::Serialize::serialize)
/// function
pub trait Pipeline<'a>:
    Debug + Serializable<CompactJson<'a>> + Serializable<PrettyJson<'a>>
{
    /// Applies a clock on the pipeline
    fn clock(&mut self) -> ClockResult;

    /// Gets the memory module in the pipeline
    fn memory_module(&self) -> &dyn MemoryModule;

    /// Gets the registers in the pipeline
    fn registers(&self) -> &Registers;
}

impl<'a> serde::Serialize for dyn Pipeline<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
        Self: Serializable<S>,
    {
        self.serialize_to(serializer)
    }
}
