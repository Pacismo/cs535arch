mod mapped_lru;
mod null;
use libseis::types::{Byte, Short, Word};
pub use mapped_lru::MappedLru;
pub use null::NullCache;

use crate::memory::Memory;

pub trait Cache {
    /// Gets the byte at the specified address.
    fn get_byte(&self, address: Word) -> Option<Byte>;
    /// Gets the short at the specified address.
    fn get_short(&self, address: Word) -> Option<Short>;
    /// Gets the word at the specified address.
    fn get_word(&self, address: Word) -> Option<Word>;

    /// Sets the byte at the specified address.
    fn write_byte(&mut self, address: Word, data: Byte) -> bool;
    /// Sets the short at the specified address.
    fn write_short(&mut self, address: Word, data: Short) -> bool;
    /// Sets the word at the specified address.
    fn write_word(&mut self, address: Word, data: Word) -> bool;

    /// Returns `true` if the cache contains the provided address.
    fn has_address(&self, address: Word) -> bool;
    /// Returns the length of a line, in bits.
    fn line_len(&self) -> usize;

    /// Fetches the data to be stored in the cache from main memory.
    ///
    /// Returns the evicted line.
    fn write_line(&mut self, address: Word, memory: &Memory) -> Option<(Word, Box<[u8]>)>;
}
