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
use serde::Serialize;
use std::fmt::Debug;

/// Represents the data in a cache
#[derive(Debug, Serialize)]
pub struct LineData<'a> {
    /// Where, in memory, the cache's data is located
    pub base_address: Word,
    /// Whether the line has been written to before
    pub dirty: bool,
    /// The data contained in the line
    pub data: &'a [u8],
}

impl<'a> From<(Word, bool, &'a [u8])> for LineData<'a> {
    fn from((base_address, dirty, data): (Word, bool, &'a [u8])) -> Self {
        Self {
            base_address,
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
    fn check_address(&self, address: Word) -> Status;
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
    fn get_line(&mut self, address: Word, memory: &mut Memory) -> LineReadStatus;

    /// Flushes all dirty lines to memory.
    ///
    /// Returns the number of lines written back to memory.
    fn flush(&mut self, memory: &mut Memory) -> usize;

    /// Returns the number of dirty lines.
    fn dirty_lines(&self) -> usize;

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

/// The status of a cache line read
#[derive(Debug, Clone, Copy)]
pub enum LineReadStatus {
    /// An eviction ocurred
    Evicted,
    /// The cache is disabled
    Disabled,
    /// The operation was skipped
    Skipped,
    /// No eviction ocurred
    Inserted,
}

impl LineReadStatus {
    /// Returns true if a cache line got evicted
    #[inline(always)]
    pub fn evicted(self) -> bool {
        matches!(self, Self::Evicted)
    }

    /// Returns true if the cache returned the [`Disabled`](LineReadStatus::Disabled)
    /// status
    #[inline(always)]
    pub fn disabled(self) -> bool {
        matches!(self, Self::Disabled)
    }

    /// Returns true if the read got skipped
    #[inline(always)]
    pub fn skipped(self) -> bool {
        matches!(self, Self::Skipped)
    }

    /// Returns true if the data got inserted, but no eviction had occurred
    #[inline(always)]
    pub fn inserted(self) -> bool {
        matches!(self, Self::Inserted)
    }
}

impl Status {
    /// Returns true if the result is a [`Cold`](Status::Cold) or [`Conflict`](Status::Conflict)
    pub fn is_miss(self) -> bool {
        matches!(self, Self::Cold | Self::Conflict)
    }

    /// Returns true if the result is a [`Hit`](Status::Hit)
    pub fn is_hit(self) -> bool {
        matches!(self, Self::Hit)
    }
}

/// The result of a cache line read
type ReadResult<T> = Result<T, Status>;

impl Serialize for dyn Cache {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_seq(self.get_lines())
    }
}
