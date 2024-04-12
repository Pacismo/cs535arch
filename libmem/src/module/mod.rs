//! The representation of a memory heirarchy.
//!
//! The [`MemoryModule`] trait to allow different memory configurations,
//! including numerous levels of cache.
//!
//! [`SingleLevel`] represents a memory hierarchy containing a single
//! level of cache.
//!
//! See [`cache`](crate::cache) for information about creating the cache.

mod single_level;

use crate::{
    cache::{Cache, LineData},
    memory::Memory,
};
use libseis::types::{Byte, Short, Word};
use serde::Serialize;
pub use single_level::SingleLevel;
use std::{collections::HashMap, fmt::Debug};

#[derive(Debug, Serialize)]
pub struct CacheData<'a> {
    pub name: String,
    pub lines: Vec<Option<LineData<'a>>>,
}

impl<'a, T: ToString + 'a> From<(T, Vec<Option<LineData<'a>>>)> for CacheData<'a> {
    fn from((name, lines): (T, Vec<Option<LineData<'a>>>)) -> Self {
        Self {
            name: name.to_string(),
            lines,
        }
    }
}

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

/// Represents a memory module containing a cache and a DRAM memory.
pub trait MemoryModule: Debug {
    /// Clocks the module, decrementing any counters.
    fn clock(&mut self, amount: usize);

    /// The number of clocks needed to unblock a blockage
    fn wait_time(&self) -> usize;

    /// Reads a byte from memory. Returns value on cache hit
    fn read_byte(&mut self, addr: Word) -> Result<Byte>;
    /// Reads a short from memory. Returns value on cache hit
    fn read_short(&mut self, addr: Word) -> Result<Short>;
    /// Reads a word from memory. Returns value on cache hit
    fn read_word(&mut self, addr: Word) -> Result<Word>;

    /// Reads a byte from memory, bypassing the cache
    fn read_byte_volatile(&mut self, addr: Word) -> Result<Byte>;
    /// Reads a short from memory, bypassing the cache
    fn read_short_volatile(&mut self, addr: Word) -> Result<Short>;
    /// Reads a word from memory, bypassing the cache
    fn read_word_volatile(&mut self, addr: Word) -> Result<Word>;

    /// Reads an instruction from memory. Returns value on cache hit
    fn read_instruction(&mut self, addr: Word) -> Result<Word>;

    /// Writes a byte to memory
    fn write_byte(&mut self, addr: Word, value: Byte) -> Status;
    /// Writes a short to memory
    fn write_short(&mut self, addr: Word, value: Short) -> Status;
    /// Writes a word to memory
    fn write_word(&mut self, addr: Word, value: Word) -> Status;

    /// Writes a byte to memory, bypassing the cache
    fn write_byte_volatile(&mut self, addr: Word, value: Byte) -> Status;
    /// Writes a short to memory, bypassing the cache
    fn write_short_volatile(&mut self, addr: Word, value: Short) -> Status;
    /// Writes a word to memory, bypassing the cache
    fn write_word_volatile(&mut self, addr: Word, value: Word) -> Status;

    /// Returns the number of cold misses that have occurred for the duration of the runtime.
    fn cold_misses(&self) -> usize;
    /// Returns the number of conflict misses that have occurred for the duration of the runtime.
    fn conflict_misses(&self) -> usize;
    /// Returns the total number of cache misses.
    fn total_misses(&self) -> usize {
        self.cold_misses() + self.conflict_misses()
    }
    /// Returns the number of cache hits that have occurred for the duration of the runtime.
    fn cache_hits(&self) -> usize;
    /// Returns the total number of memory accesses that have occurred in the duration of the runtime.
    fn accesses(&self) -> usize;
    /// Returns the total number of times a cache line was evicted
    fn evictions(&self) -> usize;

    /// Get the memory structure
    fn memory(&self) -> &Memory;

    /// Get a mutable reference to the memory structure
    fn memory_mut(&mut self) -> &mut Memory;

    /// Gets a hashmap containing the caches in the memory module
    fn caches<'a>(&'a self) -> HashMap<&'static str, &'a dyn Cache>;

    /// Gets a hashmap containing mutable references to the caches in the memory module
    fn caches_mut<'a>(&'a mut self) -> HashMap<&'static str, &'a mut dyn Cache>;

    /// Writes the cache back to memory. Returns [`Status::Idle`] on success.
    fn immediate_writeback(&mut self) -> Status;

    /// Get the state of the cache structures
    ///
    /// Provides the names of the caches as well
    fn cache_state<'a>(&'a self) -> Vec<CacheData> {
        self.caches()
            .into_iter()
            .map(|(name, cache)| CacheData {
                name: name.into(),
                lines: cache.get_lines(),
            })
            .collect()
    }

    /// Gets a reference to the data cache (L1)
    fn data_cache<'a>(&'a self) -> &'a dyn Cache;

    /// Flush the contents of cache into memory
    fn flush_cache(&mut self) -> Status;
}
