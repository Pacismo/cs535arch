//! Caches for use in the memory module.
//!
//! The [`Cache`] trait is an abstract representation of the cache.
//!
//! There are a handful of datastructures that implement the trait:
//!
//! - [`NullCache`], which always directs memory accesses to the
//! main memory
//! - [`Associative`], which represents a one-way set-associative
//! cache.

mod associative;
mod null;

use crate::memory::Memory;
pub use associative::*;
use libseis::types::{Byte, Short, Word};
pub use null::NullCache;
use std::fmt::Debug;

pub struct LineData<'a> {
    pub address_base: Word,
    pub dirty: bool,
    pub data: &'a [u8],
}

impl<'a> From<(Word, bool, &'a [u8])> for LineData<'a> {
    fn from((address_base, dirty, data): (Word, bool, &'a [u8])) -> Self {
        Self {
            address_base,
            dirty,
            data,
        }
    }
}

/// Interface for a cache.
///
/// This enables the use of `dyn Cache` in code.
///
/// This interface also requires the struct implement [`Debug`]
/// to allow debug information to be printed to the screen.
pub trait Cache: Debug {
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

    /// Invalidates a line (say, due to a volatile write)
    ///
    /// Returns true if a line *has* been invalidated.
    fn invalidate_line(&mut self, address: Word) -> bool;

    /// Fetches the data to be stored in the cache from main memory.
    ///
    /// Writes any evicted lines back and returns true if an eviction occurred.
    fn write_line(&mut self, address: Word, memory: &mut Memory) -> LineReadStatus;

    /// Gets all the lines available in the cache.
    ///
    /// The data stored is useful to provide information about what the cache is doing.
    fn get_lines(&self) -> Vec<Option<LineData>>;

    /// Read a byte at an address, if available
    fn byte_at(&self, address: Word) -> Option<Byte>;
    /// Read a short at an address, if available
    fn short_at(&self, address: Word) -> Option<Short>;
    /// Read a word at an address if available
    fn word_at(&self, address: Word) -> Option<Word>;
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

#[derive(Debug, Clone, Copy)]
pub enum LineReadStatus {
    Evicted,
    Swapped,
    Disabled,
}

impl LineReadStatus {
    #[inline(always)]
    pub fn evicted(self) -> bool {
        matches!(self, Self::Evicted)
    }

    #[inline(always)]
    pub fn swapped(self) -> bool {
        matches!(self, Self::Swapped)
    }

    #[inline(always)]
    pub fn disabled(self) -> bool {
        matches!(self, Self::Disabled)
    }
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
