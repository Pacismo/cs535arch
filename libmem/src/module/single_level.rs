use super::{MemoryModule, Result, Status};
use crate::{
    cache::{self, Cache},
    memory::Memory,
};
use libseis::types::{Byte, Short, Word};
use Transaction::*;

#[derive(Debug, Clone)]
pub enum Transaction {
    Idle,

    ReadByte(Word),
    ReadShort(Word),
    ReadWord(Word),

    WriteByte(Word, Byte),
    WriteShort(Word, Short),
    WriteWord(Word, Word),
}

impl Transaction {
    pub fn is_busy(&self) -> bool {
        matches!(self, Self::Idle)
    }
}

/// Represents a memory module with a single level of cache.
pub struct SingleLevel {
    cache: Box<dyn Cache>,
    memory: Memory,

    miss_penalty: usize,
    writethrough: bool,

    current_transaction: Transaction,
    clocks: usize,

    cold_misses: usize,
    misses: usize,
    hits: usize,
    evictions: usize,
}

impl MemoryModule for SingleLevel {
    fn clock(&mut self, amount: usize) {
        self.clocks = self.clocks.saturating_sub(amount);

        if self.clocks == 0 {
            match self.current_transaction {
                WriteByte(addr, value) => {
                    if self.writethrough {
                        self.memory.write_byte(addr, value);
                    } else {
                        if self.cache.write_line(addr, &mut self.memory) {
                            self.evictions += 1;
                        }

                        self.cache.write_byte(addr, value);
                    }
                }
                WriteShort(addr, value) => {
                    if self.writethrough {
                        self.memory.write_short(addr, value);
                    } else {
                        if self.cache.write_line(addr, &mut self.memory) {
                            self.evictions += 1;
                        }
                        if self.cache.within_line(addr, 2) {
                            if self.cache.write_line(addr + 1, &mut self.memory) {
                                self.evictions += 1;
                            }
                        }

                        self.cache.write_short(addr, value);
                    }
                }
                WriteWord(addr, value) => {
                    if self.writethrough {
                        self.memory.write_word(addr, value);
                    } else {
                        if self.cache.write_line(addr, &mut self.memory) {
                            self.evictions += 1;
                        }
                        if self.cache.within_line(addr, 2) {
                            if self.cache.write_line(addr + 3, &mut self.memory) {
                                self.evictions += 1;
                            }
                        }

                        self.cache.write_word(addr, value);
                    }
                }

                _ => (),
            }
        }
    }

    fn read_byte(&mut self, addr: Word) -> Result<Byte> {
        if let ReadByte(address) = self.current_transaction {
            if address == addr {
                if self.clocks > 0 {
                    Err(Status::Reading(self.clocks))
                } else {
                    if self.cache.write_line(addr, &mut self.memory) {
                        self.evictions += 1;
                    }
                    self.current_transaction = Idle;
                    Ok(self.cache.read_byte(addr).unwrap())
                }
            } else {
                Err(Status::Busy(self.clocks))
            }
        } else {
            match self.cache.read_byte(addr) {
                Ok(value) => {
                    self.hits += 1;

                    Ok(value)
                }
                Err(cache::Status::Cold) => {
                    self.cold_misses += 1;
                    self.clocks = self.miss_penalty;
                    self.current_transaction = ReadByte(addr);
                    Err(Status::Reading(self.clocks))
                }
                Err(cache::Status::Conflict) => {
                    self.misses += 1;
                    self.clocks = self.miss_penalty;
                    self.current_transaction = ReadByte(addr);
                    Err(Status::Reading(self.clocks))
                }
                Err(cache::Status::Disabled) => {
                    self.clocks = self.miss_penalty;
                    self.current_transaction = ReadByte(addr);
                    Err(Status::Reading(self.clocks))
                }
                _ => unreachable!("No hits allowed!"),
            }
        }
    }

    fn read_short(&mut self, addr: Word) -> Result<Short> {
        if let ReadShort(address) = self.current_transaction {
            if address == addr {
                if self.clocks > 0 {
                    Err(Status::Reading(self.clocks))
                } else {
                    if self.cache.write_line(addr, &mut self.memory) {
                        self.evictions += 1;
                    }
                    self.current_transaction = Idle;
                    Ok(self.cache.read_short(addr).unwrap())
                }
            } else {
                Err(Status::Busy(self.clocks))
            }
        } else {
            match self.cache.read_short(addr) {
                Ok(value) => {
                    self.hits += 1;

                    Ok(value)
                }
                Err(cache::Status::Cold) => {
                    self.cold_misses += 1;

                    self.clocks = self.miss_penalty;
                    if addr % 2 != 0 {
                        self.clocks += 1 + self.miss_penalty;
                    }

                    self.current_transaction = ReadShort(addr);
                    Err(Status::Reading(self.clocks))
                }
                Err(cache::Status::Conflict) => {
                    self.misses += 1;

                    self.clocks = self.miss_penalty;
                    if addr % 2 != 0 {
                        self.clocks += 1 + self.miss_penalty;
                    }

                    self.current_transaction = ReadShort(addr);
                    Err(Status::Reading(self.clocks))
                }
                Err(cache::Status::Disabled) => {
                    self.clocks = self.miss_penalty;
                    if addr % 2 != 0 {
                        self.clocks += 1 + self.miss_penalty;
                    }

                    self.current_transaction = ReadShort(addr);
                    Err(Status::Reading(self.clocks))
                }
                _ => unreachable!("No hits allowed!"),
            }
        }
    }

    fn read_word(&mut self, addr: Word) -> Result<Word> {
        if let ReadWord(address) = self.current_transaction {
            if address == addr {
                if self.clocks > 0 {
                    Err(Status::Reading(self.clocks))
                } else {
                    if self.cache.write_line(addr, &mut self.memory) {
                        self.evictions += 1;
                    }
                    self.current_transaction = Idle;
                    Ok(self.cache.read_word(addr).unwrap())
                }
            } else {
                Err(Status::Busy(self.clocks))
            }
        } else {
            match self.cache.read_word(addr) {
                Ok(value) => {
                    self.hits += 1;

                    Ok(value)
                }
                Err(cache::Status::Cold) => {
                    self.cold_misses += 1;

                    self.clocks = self.miss_penalty;
                    if addr % 4 != 0 {
                        self.clocks += 1 + self.miss_penalty;
                    }

                    self.current_transaction = ReadWord(addr);
                    Err(Status::Reading(self.clocks))
                }
                Err(cache::Status::Conflict) => {
                    self.misses += 1;

                    self.clocks = self.miss_penalty;
                    if addr % 4 != 0 {
                        self.clocks += 1 + self.miss_penalty;
                    }

                    self.current_transaction = ReadWord(addr);
                    Err(Status::Reading(self.clocks))
                }
                Err(cache::Status::Disabled) => {
                    self.clocks = self.miss_penalty;
                    if addr % 4 != 0 {
                        self.clocks += 1 + self.miss_penalty;
                    }

                    self.current_transaction = ReadWord(addr);
                    Err(Status::Reading(self.clocks))
                }
                _ => unreachable!("No hits allowed!"),
            }
        }
    }

    fn write_byte(&mut self, addr: Word, value: Byte) -> Status {
        if self.current_transaction.is_busy() {
            Status::Busy(self.clocks)
        } else {
            match self.cache.write_byte(addr, value) {
                cache::Status::Hit => {
                    self.hits += 1;

                    Status::Idle
                }
                cache::Status::Conflict => {
                    self.misses += 1;
                    self.clocks = self.miss_penalty;

                    self.current_transaction = WriteByte(addr, value);
                    Status::Busy(self.clocks)
                }
                cache::Status::Cold => {
                    self.cold_misses += 1;
                    self.clocks = self.miss_penalty;

                    self.current_transaction = WriteByte(addr, value);
                    Status::Busy(self.clocks)
                }
                cache::Status::Disabled => {
                    self.clocks = self.miss_penalty;

                    self.current_transaction = WriteByte(addr, value);
                    Status::Busy(self.clocks)
                }
            }
        }
    }

    fn write_short(&mut self, addr: Word, value: Short) -> Status {
        if self.current_transaction.is_busy() {
            Status::Busy(self.clocks)
        } else {
            match self.cache.write_short(addr, value) {
                cache::Status::Hit => {
                    self.hits += 1;

                    Status::Idle
                }
                cache::Status::Conflict => {
                    self.misses += 1;
                    self.clocks = self.miss_penalty;
                    if addr % 2 != 0 {
                        self.clocks += 1 + self.miss_penalty;
                    }

                    self.current_transaction = WriteShort(addr, value);
                    Status::Busy(self.clocks)
                }
                cache::Status::Cold => {
                    self.cold_misses += 1;
                    self.clocks = self.miss_penalty;
                    if addr % 2 != 0 {
                        self.clocks += 1 + self.miss_penalty;
                    }

                    self.current_transaction = WriteShort(addr, value);
                    Status::Busy(self.clocks)
                }
                cache::Status::Disabled => {
                    self.clocks = self.miss_penalty;
                    if addr % 2 != 0 {
                        self.clocks += 1 + self.miss_penalty;
                    }

                    self.current_transaction = WriteShort(addr, value);
                    Status::Busy(self.clocks)
                }
            }
        }
    }

    fn write_word(&mut self, addr: Word, value: Word) -> Status {
        if self.current_transaction.is_busy() {
            Status::Busy(self.clocks)
        } else {
            match self.cache.write_word(addr, value) {
                cache::Status::Hit => {
                    self.hits += 1;

                    Status::Idle
                }
                cache::Status::Conflict => {
                    self.misses += 1;
                    self.clocks = self.miss_penalty;
                    if addr % 4 != 0 {
                        self.clocks += 1 + self.miss_penalty;
                    }

                    self.current_transaction = WriteWord(addr, value);
                    Status::Busy(self.clocks)
                }
                cache::Status::Cold => {
                    self.cold_misses += 1;
                    self.clocks = self.miss_penalty;
                    if addr % 4 != 0 {
                        self.clocks += 1 + self.miss_penalty;
                    }

                    self.current_transaction = WriteWord(addr, value);
                    Status::Busy(self.clocks)
                }
                cache::Status::Disabled => {
                    self.clocks = self.miss_penalty;
                    if addr % 4 != 0 {
                        self.clocks += 1 + self.miss_penalty;
                    }

                    self.current_transaction = WriteWord(addr, value);
                    Status::Busy(self.clocks)
                }
            }
        }
    }

    fn cold_misses(&self) -> usize {
        self.cold_misses
    }

    fn cache_misses(&self) -> usize {
        self.misses
    }

    fn cache_hits(&self) -> usize {
        self.hits
    }

    fn accesses(&self) -> usize {
        self.cold_misses + self.misses + self.hits
    }
}

impl SingleLevel {
    /// Creates a new single-level cache memory system.
    ///
    /// # Arguments
    ///
    /// - `cache` -- the cache to use
    /// - `memory` -- the memoryspace to use
    /// - `miss_penalty` -- the penalty of a cache miss
    /// - `writethrough` -- whether the cache is *writethrough*, meaning uncached writes go straight to memory
    ///
    /// # Notes
    ///
    /// - A misaligned access has a clock penalty of 1 plus another miss penalty (multiple accesses)
    pub fn new<T: Cache + 'static>(
        cache: T,
        memory: Memory,
        miss_penalty: usize,
        writethrough: bool,
    ) -> Self {
        Self::new_with_boxed(Box::new(cache), memory, miss_penalty, writethrough)
    }

    pub fn new_with_boxed(
        cache: Box<dyn Cache>,
        memory: Memory,
        miss_penalty: usize,
        writethrough: bool,
    ) -> Self {
        Self {
            cache,
            memory,

            miss_penalty,
            writethrough,

            current_transaction: Idle,
            clocks: 0,

            cold_misses: 0,
            misses: 0,
            hits: 0,
            evictions: 0,
        }
    }
}
