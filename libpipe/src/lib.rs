//! Pipelining traits. This provides the necessary means of
//! implementing a processor.
//!
//! The expectation is that the processor will implement the
//! fetch, decode, execute, memory, and writeback stages.

#![warn(missing_docs)]

mod piped;
mod reg_locks;
mod registers;
mod regmap;
mod stages;
mod unpiped;

use libmem::module::MemoryModule;
pub use piped::Pipelined;
pub use reg_locks::Locks;
pub use registers::Registers;
use serde::Serialize;
pub use stages::*;
use std::fmt::Debug;
pub use unpiped::Unpipelined;

/// Represents the major stages in a [`Pipeline`]
#[derive(Debug)]
pub struct PipelineStages<'a> {
    /// The [`fetch`](Fetch) stage
    pub fetch: &'a Fetch,
    /// The [`decode`](Decode) stage
    pub decode: &'a Decode,
    /// The [`execute`](Execute) stage
    pub execute: &'a Execute,
    /// The [`memory`](Memory) stage
    pub memory: &'a Memory,
    /// The [`writeback`](Writeback) stage
    pub writeback: &'a Writeback,
}

impl<'a> Serialize for PipelineStages<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        let mut map = serializer.serialize_map(Some(5))?;

        map.serialize_entry("fetch", self.fetch)?;
        map.serialize_entry("decode", self.decode)?;
        map.serialize_entry("execute", self.execute)?;
        map.serialize_entry("memory", self.memory)?;
        map.serialize_entry("writeback", self.writeback)?;

        map.end()
    }
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
    /// Get a mutable reference to the memory module
    fn memory_module_mut(&mut self) -> &mut dyn MemoryModule;

    /// Gets the registers in the pipeline
    fn registers(&self) -> &Registers;
    /// Get a mutable reference to the registers
    fn registers_mut(&mut self) -> &mut Registers;

    /// Gets references to the pipeline stages
    fn stages(&self) -> PipelineStages;
}
