use super::{MemoryModule, Result, Status};
use crate::{
    cache::{self, Cache},
    memory::Memory,
};
use libseis::types::{Byte, Short, Word};
use Transaction::*;

/// Represents a memory transaction.
#[derive(Debug, Clone)]
enum Transaction {
    /// No transactions are occurring at this time
    Idle,

    /// The memory unit is reading a byte of memory.
    ReadByte(Word),
    /// The memory unit is reading two bytes of memory.
    ReadShort(Word),
    /// The memory unit is reading four bytes of memory.
    ReadWord(Word),
    /// The memory unit is reading an instruction.
    ReadInstruction(Word),

    /// The memory unit is writeing a byte.
    WriteByte(Word, Byte),
    /// The memory unit is writeing two bytes.
    WriteShort(Word, Short),
    /// The memory unit is writeing four bytes.
    WriteWord(Word, Word),
}

impl Transaction {
    pub fn busy_data(&self) -> bool {
        !matches!(self, Idle | ReadInstruction(_))
    }

    pub fn busy_instruction(&self) -> bool {
        matches!(self, ReadInstruction(_))
    }

    pub fn is_busy(&self) -> bool {
        !matches!(self, Idle)
    }
}

/// Represents a memory module with a single level of cache.
pub struct SingleLevel {
    data_cache: Box<dyn Cache>,
    instruction_cache: Box<dyn Cache>,
    memory: Memory,

    miss_penalty: usize,
    writethrough: bool,

    current_transaction: Transaction,
    clocks: usize,

    cold_misses: usize,
    misses: usize,
    accesses: usize,
    evictions: usize,
}

impl MemoryModule for SingleLevel {
    fn clock(&mut self, amount: usize) {
        self.clocks = self.clocks.saturating_sub(amount);

        if self.clocks == 0 {
            match self.current_transaction {
                WriteByte(addr, value) => {
                    if self.writethrough {
                        // This is required for the null cache. However, this does nothing for other caches.
                        self.data_cache.invalidate_line(addr);
                        self.memory.write_byte(addr, value);
                    } else {
                        if self.data_cache.write_line(addr, &mut self.memory).evicted() {
                            self.evictions += 1;
                        }

                        self.data_cache.write_byte(addr, value);
                    }
                }
                WriteShort(addr, value) => {
                    if self.writethrough {
                        self.data_cache.invalidate_line(addr);
                        self.memory.write_short(addr, value);
                    } else {
                        if self.data_cache.write_line(addr, &mut self.memory).evicted() {
                            self.evictions += 1;
                        }
                        if self.data_cache.within_line(addr, 2) {
                            if self
                                .data_cache
                                .write_line(addr + 1, &mut self.memory)
                                .evicted()
                            {
                                self.evictions += 1;
                            }
                        }

                        self.data_cache.write_short(addr, value);
                    }
                }
                WriteWord(addr, value) => {
                    if self.writethrough {
                        self.data_cache.invalidate_line(addr);
                        self.memory.write_word(addr, value);
                    } else {
                        if self.data_cache.write_line(addr, &mut self.memory).evicted() {
                            self.evictions += 1;
                        }
                        if self.data_cache.within_line(addr, 2) {
                            if self
                                .data_cache
                                .write_line(addr + 3, &mut self.memory)
                                .evicted()
                            {
                                self.evictions += 1;
                            }
                        }

                        self.data_cache.write_word(addr, value);
                    }
                }

                ReadByte(addr) => {
                    if self.data_cache.write_line(addr, &mut self.memory).evicted() {
                        self.evictions += 1;
                    }
                }
                ReadShort(addr) => {
                    if self.data_cache.write_line(addr, &mut self.memory).evicted() {
                        self.evictions += 1;
                    }
                    if self.data_cache.within_line(addr, 2) {
                        if self
                            .data_cache
                            .write_line(addr + 1, &mut self.memory)
                            .evicted()
                        {
                            self.evictions += 1;
                        }
                    }
                }
                ReadWord(addr) => {
                    if self.data_cache.write_line(addr, &mut self.memory).evicted() {
                        self.evictions += 1;
                    }
                    if self.data_cache.within_line(addr, 4) {
                        if self.data_cache.write_line(addr, &mut self.memory).evicted() {
                            self.evictions += 1;
                        }
                    }
                }
                ReadInstruction(addr) => {
                    self.instruction_cache.write_line(addr, &mut self.memory);
                }

                _ => (),
            }

            self.current_transaction = Idle;
        }
    }

    fn read_byte(&mut self, addr: Word) -> Result<Byte> {
        if self.current_transaction.busy_data() {
            return Err(Status::Busy(self.clocks));
        }

        match self.data_cache.read_byte(addr) {
            Ok(value) => {
                self.accesses += 1;

                Ok(value)
            }
            Err(cache::Status::Cold) => {
                self.cold_misses += 1;
                self.clocks = self.miss_penalty;
                Err(self.set_if_idle(ReadByte(addr)))
            }
            Err(cache::Status::Conflict) => {
                self.misses += 1;
                self.clocks = self.miss_penalty;
                Err(self.set_if_idle(ReadByte(addr)))
            }
            Err(cache::Status::Disabled) => {
                self.clocks = self.miss_penalty;
                Err(self.set_if_idle(ReadByte(addr)))
            }

            _ => unreachable!("No hits allowed!"),
        }
    }

    fn read_short(&mut self, addr: Word) -> Result<Short> {
        if self.current_transaction.busy_data() {
            return Err(Status::Busy(self.clocks));
        }

        match self.data_cache.read_short(addr) {
            Ok(value) => {
                self.accesses += 1;

                Ok(value)
            }
            Err(cache::Status::Cold) => {
                self.cold_misses += 1;
                self.clocks = self.miss_penalty;
                Err(self.set_if_idle(ReadShort(addr)))
            }
            Err(cache::Status::Conflict) => {
                self.misses += 1;
                self.clocks = self.miss_penalty;
                Err(self.set_if_idle(ReadShort(addr)))
            }
            Err(cache::Status::Disabled) => {
                self.clocks = self.miss_penalty;
                Err(self.set_if_idle(ReadShort(addr)))
            }

            _ => unreachable!("No hits allowed!"),
        }
    }

    fn read_word(&mut self, addr: Word) -> Result<Word> {
        if self.current_transaction.busy_data() {
            return Err(Status::Busy(self.clocks));
        }

        match self.data_cache.read_word(addr) {
            Ok(value) => {
                self.accesses += 1;

                Ok(value)
            }
            Err(cache::Status::Cold) => {
                self.cold_misses += 1;
                self.clocks = self.miss_penalty;
                Err(self.set_if_idle(ReadWord(addr)))
            }
            Err(cache::Status::Conflict) => {
                self.misses += 1;
                self.clocks = self.miss_penalty;
                Err(self.set_if_idle(ReadWord(addr)))
            }
            Err(cache::Status::Disabled) => {
                self.clocks = self.miss_penalty;
                Err(self.set_if_idle(ReadWord(addr)))
            }

            _ => unreachable!("No hits allowed!"),
        }
    }

    fn read_instruction(&mut self, addr: Word) -> Result<Word> {
        if self.current_transaction.busy_instruction() {
            return Err(Status::Busy(self.clocks));
        }

        match self.instruction_cache.read_word(addr) {
            Ok(value) => {
                self.accesses += 1;

                Ok(value)
            }
            Err(cache::Status::Cold) => {
                self.cold_misses += 1;
                self.clocks = self.miss_penalty;
                Err(self.set_if_idle(ReadInstruction(addr)))
            }
            Err(cache::Status::Conflict) => {
                self.misses += 1;
                self.clocks = self.miss_penalty;
                Err(self.set_if_idle(ReadInstruction(addr)))
            }
            Err(cache::Status::Disabled) => {
                self.clocks = self.miss_penalty;
                Err(self.set_if_idle(ReadInstruction(addr)))
            }

            _ => unreachable!("No hits allowed!"),
        }
    }

    fn write_byte(&mut self, addr: Word, value: Byte) -> Status {
        if self.current_transaction.busy_data() {
            Status::Busy(self.clocks)
        } else {
            match self.data_cache.write_byte(addr, value) {
                cache::Status::Hit => {
                    self.accesses += 1;

                    Status::Idle
                }
                cache::Status::Conflict => {
                    self.misses += 1;
                    self.clocks = self.miss_penalty;

                    self.set_if_idle(WriteByte(addr, value))
                }
                cache::Status::Cold => {
                    self.cold_misses += 1;
                    self.clocks = self.miss_penalty;

                    self.set_if_idle(WriteByte(addr, value))
                }
                cache::Status::Disabled => {
                    self.clocks = self.miss_penalty;

                    self.set_if_idle(WriteByte(addr, value))
                }
            }
        }
    }

    fn write_short(&mut self, addr: Word, value: Short) -> Status {
        if self.current_transaction.busy_data() {
            Status::Busy(self.clocks)
        } else {
            match self.data_cache.write_short(addr, value) {
                cache::Status::Hit => {
                    self.accesses += 1;

                    Status::Idle
                }
                cache::Status::Conflict => {
                    self.misses += 1;
                    self.clocks = self.miss_penalty;
                    if addr % 2 != 0 {
                        self.clocks += 1 + self.miss_penalty;
                    }

                    self.set_if_idle(WriteShort(addr, value))
                }
                cache::Status::Cold => {
                    self.cold_misses += 1;
                    self.clocks = self.miss_penalty;
                    if addr % 2 != 0 {
                        self.clocks += 1 + self.miss_penalty;
                    }

                    self.set_if_idle(WriteShort(addr, value))
                }
                cache::Status::Disabled => {
                    self.clocks = self.miss_penalty;
                    if addr % 2 != 0 {
                        self.clocks += 1 + self.miss_penalty;
                    }

                    self.set_if_idle(WriteShort(addr, value))
                }
            }
        }
    }

    fn write_word(&mut self, addr: Word, value: Word) -> Status {
        if self.current_transaction.busy_data() {
            Status::Busy(self.clocks)
        } else {
            match self.data_cache.write_word(addr, value) {
                cache::Status::Hit => {
                    self.accesses += 1;

                    Status::Idle
                }
                cache::Status::Conflict => {
                    self.misses += 1;
                    self.clocks = self.miss_penalty;
                    if addr % 4 != 0 {
                        self.clocks += 1 + self.miss_penalty;
                    }

                    self.set_if_idle(WriteWord(addr, value))
                }
                cache::Status::Cold => {
                    self.cold_misses += 1;
                    self.clocks = self.miss_penalty;
                    if addr % 4 != 0 {
                        self.clocks += 1 + self.miss_penalty;
                    }

                    self.set_if_idle(WriteWord(addr, value))
                }
                cache::Status::Disabled => {
                    self.clocks = self.miss_penalty;
                    if addr % 4 != 0 {
                        self.clocks += 1 + self.miss_penalty;
                    }

                    self.set_if_idle(WriteWord(addr, value))
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
        self.accesses - (self.cold_misses + self.misses)
    }

    fn accesses(&self) -> usize {
        self.cold_misses + self.misses + self.accesses
    }
}

impl SingleLevel {
    /// Creates a new single-level cache memory system.
    ///
    /// # Arguments
    ///
    /// - `data_cache` -- the cache to use for data
    /// - `instruction_cache` -- the cache to use for instructions
    /// - `memory` -- the memoryspace to use
    /// - `miss_penalty` -- the penalty of a cache miss
    /// - `writethrough` -- whether the cache is *writethrough*, meaning uncached writes go straight to memory
    ///
    /// # Notes
    ///
    /// - A misaligned access has a clock penalty of 1 plus another miss penalty (multiple accesses)
    pub fn new<T: Cache + 'static, U: Cache + 'static>(
        data_cache: U,
        instruction_cache: T,
        memory: Memory,
        miss_penalty: usize,
        writethrough: bool,
    ) -> Self {
        Self::new_with_boxed(
            Box::new(data_cache),
            Box::new(instruction_cache),
            memory,
            miss_penalty,
            writethrough,
        )
    }

    pub fn new_with_boxed(
        data_cache: Box<dyn Cache>,
        instruction_cache: Box<dyn Cache>,
        memory: Memory,
        miss_penalty: usize,
        writethrough: bool,
    ) -> Self {
        Self {
            data_cache,
            instruction_cache,
            memory,

            miss_penalty,
            writethrough,

            current_transaction: Idle,
            clocks: 0,

            cold_misses: 0,
            misses: 0,
            accesses: 0,
            evictions: 0,
        }
    }

    /// Sets the transaction if idle
    fn set_if_idle(&mut self, transaction: Transaction) -> Status {
        if !self.current_transaction.is_busy() {
            self.current_transaction = transaction;
        }
        Status::Busy(self.clocks)
    }
}
