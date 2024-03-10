mod associative;
mod null;

use crate::memory::Memory;
pub use associative::Associative;
use libseis::types::{Byte, Short, Word};
pub use null::NullCache;
use std::fmt::Debug;

/// Interface for a cache.
///
/// This enables the use of `dyn Cache` in code.
///
/// This interface also requires the struct implement [`Debug`]
/// to allow debug information to be printed to the screen.
pub trait Cache: Debug {
    /// Gets the byte at the address without modifying the contents of the cache.
    ///
    /// See also: [`Cache::get_byte`]
    fn read_byte(&self, address: Word) -> ReadResult<Byte>;
    /// Gets the short at the address without modifying the contents of the cache.
    ///
    /// See also: [`Cache::get_short`]
    fn read_short(&self, address: Word) -> ReadResult<Short>;
    /// Gets the word at the address without modifying the contents of the cache.
    ///
    /// See also: [`Cache::get_word`]
    fn read_word(&self, address: Word) -> ReadResult<Word>;

    /// Gets the byte at the specified address. This may potentially modify the contents of the cache.
    ///
    /// Returns a value only on cache hit.
    fn get_byte(&mut self, address: Word) -> ReadResult<Byte>;
    /// Gets the short at the specified address. This may potentially modify the contents of the cache.
    ///
    /// Returns a value only on cache hit.
    fn get_short(&mut self, address: Word) -> ReadResult<Short>;
    /// Gets the word at the specified address. This may potentially modify the contents of the cache.
    ///
    /// Returns a value only on cache hit.
    fn get_word(&mut self, address: Word) -> ReadResult<Word>;

    /// Sets the byte at the specified address.
    ///
    /// Returns true if the write was a hit.
    fn write_byte(&mut self, address: Word, data: Byte) -> Status;
    /// Sets the short at the specified address.
    ///
    /// Returns true if the write was a hit.
    fn write_short(&mut self, address: Word, data: Short) -> Status;
    /// Sets the word at the specified address.
    ///
    /// Returns true if the write was a hit.
    fn write_word(&mut self, address: Word, data: Word) -> Status;

    /// Returns `true` if the cache contains the provided address.
    fn has_address(&self, address: Word) -> bool;
    /// Returns the length of a line, in bits.
    fn line_len(&self) -> usize;

    /// Returns whether the address' value is contained in the same line up to the length
    fn within_line(&self, address: Word, length: usize) -> bool;

    /// Fetches the data to be stored in the cache from main memory.
    ///
    /// Writes any evicted lines back and returns true if an eviction occurred.
    fn write_line(&mut self, address: Word, memory: &mut Memory) -> bool;
}

/// The status of a read.
#[derive(Debug, Clone, Copy)]
pub enum Status {
    /// The cache got a hit
    Hit,

    /// The cache is disabled (i.e. this is a [`NullCache`])
    Disabled,

    /// Cold miss (not initialized)
    Cold,

    /// Conflict miss (initialized, but wrong tag or address)
    Conflict,
}

impl Status {
    pub fn is_miss(self) -> bool {
        matches!(self, Self::Cold | Self::Conflict)
    }

    pub fn is_hit(self) -> bool {
        matches!(self, Self::Hit)
    }
}

type ReadResult<T> = Result<T, Status>;
