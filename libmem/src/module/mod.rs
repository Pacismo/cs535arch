mod single_level;

use libseis::types::{Byte, Short, Word};
pub use single_level::SingleLevel;

#[derive(Debug, Clone, Copy)]
pub enum Status {
    /// The module is idle, awaiting a new command.
    Idle,

    /// The module is busy getting a value for a different address.
    ///
    /// The contained value represents the number of remaining clocks for completion.
    Busy(usize),
}

pub type Result<T> = std::result::Result<T, Status>;

/// Represents a memory module containing a chache and a DRAM memory.
pub trait MemoryModule {
    /// Clocks the module, decrementing any counters.
    fn clock(&mut self, amount: usize);

    /// Reads a byte from memory. Returns value on cache hit
    fn read_byte(&mut self, addr: Word) -> Result<Byte>;
    /// Reads a short from memory. Returns value on cache hit
    fn read_short(&mut self, addr: Word) -> Result<Short>;
    /// Reads a word from memory. Returns value on cache hit
    fn read_word(&mut self, addr: Word) -> Result<Word>;
    /// Reads an instruction from memory. Returns value on cache hit
    fn read_instruction(&mut self, addr: Word) -> Result<Word>;

    /// Writes a byte to memory
    fn write_byte(&mut self, addr: Word, value: Byte) -> Status;
    /// Writes a short to memory
    fn write_short(&mut self, addr: Word, value: Short) -> Status;
    /// Writes a word to memory
    fn write_word(&mut self, addr: Word, value: Word) -> Status;

    /// Returns the total number of *cold* cache misses that have occurred for the duration of the runtime.
    fn cold_misses(&self) -> usize;
    /// Returns the number of cache misses that have occurred for the duration of the runtime.
    fn cache_misses(&self) -> usize;
    /// Returns the number of cache hits that have occurred for the duration of the runtime.
    fn cache_hits(&self) -> usize;
    /// Returns the total number of memory accesses that have occurred in the duration of the runtime.
    fn accesses(&self) -> usize;
}
