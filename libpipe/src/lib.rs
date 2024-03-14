//! Pipelining traits. This provides the necessary means of
//! implementing a processor.
//!
//! The expectation is that the processor will implement the
//! fetch, decode, execute, memory, and writeback stages.

mod registers;

use libmem::module::MemoryModule;
pub use registers::Registers;
use serde_json::ser::CompactFormatter;
use std::fmt::Debug;
use std::io::Write;

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

type JsonSerializer<'a> = &'a mut serde_json::Serializer<&'a mut dyn Write, CompactFormatter>;
type JsonSerializerResult<'a> = Result<
    <JsonSerializer<'a> as serde::Serializer>::Ok,
    <JsonSerializer<'a> as serde::Serializer>::Error,
>;

/// Represents a processor pipeline. Clocking the processor will yield
/// a [`ClockResult`].
pub trait Pipeline: Debug {
    /// Applies a clock on the pipeline
    fn clock(&mut self) -> ClockResult;

    /// Gets the memory module in the pipeline
    fn memory_module(&self) -> &dyn MemoryModule;

    /// Gets the registers in the pipeline
    fn registers(&self) -> &Registers;

    /// Serializes the stages of the pipeline into JSON data
    fn serialize_stages_json(&self, jser: JsonSerializer) -> JsonSerializerResult;
}
