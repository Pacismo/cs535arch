mod mapped_lru;
mod null;
use std::fmt::Debug;

use libseis::types::{Byte, Short, Word};
pub use mapped_lru::MappedLru;
pub use null::NullCache;

use crate::memory::Memory;

/// Interface for a cache.
///
/// This enables the use of `dyn Cache` in code.
///
/// This interface also requires the struct implement [`Debug`]
/// to allow debug information to be printed to the screen.
pub trait Cache: Debug {
    /// Gets the byte at the specified address.
    ///
    /// Returns a value only on cache hit.
    fn get_byte(&self, address: Word) -> ReadResult<Byte>;
    /// Gets the short at the specified address.
    ///
    /// Returns a value only on cache hit.
    fn get_short(&self, address: Word) -> ReadResult<Short>;
    /// Gets the word at the specified address.
    ///
    /// Returns a value only on cache hit.
    fn get_word(&self, address: Word) -> ReadResult<Word>;

    /// Sets the byte at the specified address.
    ///
    /// Returns true if the write was a hit.
    fn write_byte(&mut self, address: Word, data: Byte) -> bool;
    /// Sets the short at the specified address.
    ///
    /// Returns true if the write was a hit.
    fn write_short(&mut self, address: Word, data: Short) -> bool;
    /// Sets the word at the specified address.
    ///
    /// Returns true if the write was a hit.
    fn write_word(&mut self, address: Word, data: Word) -> bool;

    /// Returns `true` if the cache contains the provided address.
    fn has_address(&self, address: Word) -> bool;
    /// Returns the length of a line, in bits.
    fn line_len(&self) -> usize;

    /// Fetches the data to be stored in the cache from main memory.
    ///
    /// Writes any evicted lines back and returns true if an eviction occurred.
    fn write_line(&mut self, address: Word, memory: &mut Memory) -> bool;
}

/// The status of a read.
pub enum Status {
    /// The cache is disabled (i.e. this is a [`NullCache`])
    Disabled,
    /// Cold miss (not initialized)
    Cold,
    /// Conflict miss (initialized, but wrong tag or address)
    Conflict,
}

type ReadResult<T> = Result<T, Status>;
