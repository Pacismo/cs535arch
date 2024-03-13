use super::{MemoryModule, Result, Status};
use crate::{
    cache::{self, Cache, LineData},
    memory::Memory,
};
use libseis::types::{Byte, Short, Word};
use Status::Busy;
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
    WriteByte(Word, Byte, bool),
    /// The memory unit is writeing two bytes.
    WriteShort(Word, Short, bool),
    /// The memory unit is writeing four bytes.
    WriteWord(Word, Word, bool),

    /// The memory unit is reading a byte, bypassing cache
    ReadByteV(Word),
    /// The memory unit is reading a short, bypassing cache
    ReadShortV(Word),
    /// The memory unit is reading a word, bypassing cache
    ReadWordV(Word),
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
    volatile_penalty: usize,
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
                WriteByte(addr, value, volatile) => {
                    if volatile {
                        self.data_cache.invalidate_line(addr);
                        self.memory.write_byte(addr, value);
                    } else if self.writethrough {
                        self.memory.write_byte(addr, value);
                    } else {
                        if self.data_cache.write_line(addr, &mut self.memory).evicted() {
                            self.evictions += 1;
                        }

                        self.data_cache.write_byte(addr, value);
                    }
                }
                WriteShort(addr, value, volatile) => {
                    if volatile {
                        self.data_cache.invalidate_line(addr);
                        self.memory.write_short(addr, value);
                    } else if self.writethrough {
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
                WriteWord(addr, value, volatile) => {
                    if volatile {
                        self.data_cache.invalidate_line(addr);
                        self.memory.write_word(addr, value);
                    } else if self.writethrough {
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

                _ => return,
            }

            self.current_transaction = Idle;
        }
    }

    fn read_byte(&mut self, addr: Word) -> Result<Byte> {
        if self.current_transaction.busy_data() {
            return Err(Busy(self.clocks));
        }

        match self.data_cache.get_byte(addr) {
            Ok(value) => {
                self.accesses += 1;

                Ok(value)
            }
            Err(cache::Status::Cold) => {
                self.cold_misses += 1;

                Err(self.set_if_idle(ReadByte(addr), self.miss_penalty))
            }
            Err(cache::Status::Conflict) => {
                self.misses += 1;

                Err(self.set_if_idle(ReadByte(addr), self.miss_penalty))
            }
            Err(cache::Status::Disabled) => {
                Err(self.set_if_idle(ReadByte(addr), self.volatile_penalty))
            }

            _ => unreachable!("No hits allowed!"),
        }
    }

    fn read_short(&mut self, addr: Word) -> Result<Short> {
        if self.current_transaction.busy_data() {
            return Err(Busy(self.clocks));
        }

        match self.data_cache.get_short(addr) {
            Ok(value) => {
                self.accesses += 1;

                Ok(value)
            }
            Err(cache::Status::Conflict) => {
                self.misses += 1;
                let mut penalty = self.miss_penalty;

                if !self.data_cache.within_line(addr, 2) {
                    penalty += self.miss_penalty;
                }

                Err(self.set_if_idle(ReadShort(addr), penalty))
            }
            Err(cache::Status::Cold) => {
                self.cold_misses += 1;
                let mut penalty = self.miss_penalty;
                if addr % 4 != 0 {
                    penalty += 1;
                }
                if !self.data_cache.within_line(addr, 2) {
                    penalty += self.miss_penalty;
                }

                Err(self.set_if_idle(ReadShort(addr), penalty))
            }
            Err(cache::Status::Disabled) => {
                let mut penalty = self.volatile_penalty;
                if addr % 2 != 0 {
                    penalty += self.volatile_penalty;
                }

                Err(self.set_if_idle(ReadShort(addr), penalty))
            }

            _ => unreachable!("No hits allowed!"),
        }
    }

    fn read_word(&mut self, addr: Word) -> Result<Word> {
        if self.current_transaction.busy_data() {
            return Err(Busy(self.clocks));
        }

        match self.data_cache.get_word(addr) {
            Ok(value) => {
                self.accesses += 1;

                Ok(value)
            }
            Err(cache::Status::Conflict) => {
                self.misses += 1;

                let mut penalty = self.miss_penalty;
                if !self.data_cache.within_line(addr, 4) {
                    penalty += self.miss_penalty
                }

                Err(self.set_if_idle(ReadWord(addr), penalty))
            }
            Err(cache::Status::Cold) => {
                self.cold_misses += 1;

                let mut penalty = self.miss_penalty;
                if addr % 4 != 0 {
                    penalty += 1;
                }
                if !self.data_cache.within_line(addr, 4) {
                    penalty += self.miss_penalty
                }

                Err(self.set_if_idle(ReadWord(addr), penalty))
            }
            Err(cache::Status::Disabled) => {
                let mut penalty = self.volatile_penalty;
                if addr % 4 != 0 {
                    penalty += self.volatile_penalty * 3;
                }

                Err(self.set_if_idle(ReadWord(addr), penalty))
            }

            _ => unreachable!("No hits allowed!"),
        }
    }

    fn read_byte_volatile(&mut self, addr: Word) -> Result<Byte> {
        if let ReadByteV(addr) = self.current_transaction {
            if self.clocks == 0 {
                self.current_transaction = Idle;
                Ok(self.memory.read_byte(addr))
            } else {
                Err(Busy(self.clocks))
            }
        } else {
            Err(self.set_if_idle(ReadByteV(addr), self.volatile_penalty))
        }
    }

    fn read_short_volatile(&mut self, addr: Word) -> Result<Short> {
        if let ReadShortV(addr) = self.current_transaction {
            if self.clocks == 0 {
                self.current_transaction = Idle;
                Ok(self.memory.read_short(addr))
            } else {
                Err(Busy(self.clocks))
            }
        } else {
            let mut penalty = self.volatile_penalty;
            if addr % 2 != 0 {
                penalty += self.volatile_penalty;
            }

            Err(self.set_if_idle(ReadShortV(addr), penalty))
        }
    }

    fn read_word_volatile(&mut self, addr: Word) -> Result<Word> {
        if let ReadWordV(addr) = self.current_transaction {
            if self.clocks == 0 {
                self.current_transaction = Idle;
                Ok(self.memory.read_word(addr))
            } else {
                Err(Busy(self.clocks))
            }
        } else {
            let mut penalty = self.volatile_penalty;
            if addr % 4 != 0 {
                penalty += self.volatile_penalty * 3;
            }

            Err(self.set_if_idle(ReadWordV(addr), penalty))
        }
    }

    fn read_instruction(&mut self, addr: Word) -> Result<Word> {
        if self.current_transaction.busy_instruction() {
            return Err(Busy(self.clocks));
        }

        match self.instruction_cache.get_word(addr) {
            Ok(value) => {
                self.accesses += 1;

                Ok(value)
            }
            Err(cache::Status::Cold) => {
                self.cold_misses += 1;

                Err(self.set_if_idle(ReadInstruction(addr), self.miss_penalty))
            }
            Err(cache::Status::Conflict) => {
                self.misses += 1;

                Err(self.set_if_idle(ReadInstruction(addr), self.miss_penalty))
            }
            Err(cache::Status::Disabled) => {
                Err(self.set_if_idle(ReadInstruction(addr), self.volatile_penalty))
            }

            _ => unreachable!("No hits allowed!"),
        }
    }

    fn write_byte(&mut self, addr: Word, value: Byte) -> Status {
        if self.current_transaction.busy_data() {
            Busy(self.clocks)
        } else {
            match self.data_cache.write_byte(addr, value) {
                cache::Status::Hit => {
                    self.accesses += 1;

                    Status::Idle
                }
                cache::Status::Conflict => {
                    self.misses += 1;

                    self.set_if_idle(WriteByte(addr, value, false), self.miss_penalty)
                }
                cache::Status::Cold => {
                    self.cold_misses += 1;

                    self.set_if_idle(WriteByte(addr, value, false), self.miss_penalty)
                }
                cache::Status::Disabled => {
                    self.set_if_idle(WriteByte(addr, value, false), self.miss_penalty)
                }
            }
        }
    }

    fn write_short(&mut self, addr: Word, value: Short) -> Status {
        if self.current_transaction.busy_data() {
            Busy(self.clocks)
        } else {
            match self.data_cache.write_short(addr, value) {
                cache::Status::Hit => {
                    self.accesses += 1;

                    Status::Idle
                }
                cache::Status::Conflict => {
                    self.misses += 1;
                    let mut penalty = self.miss_penalty;
                    if addr % 2 != 0 {
                        penalty += 1;
                    }
                    if !self.data_cache.within_line(addr, 2) {
                        penalty += self.miss_penalty;
                    }

                    self.set_if_idle(WriteShort(addr, value, false), penalty)
                }
                cache::Status::Cold => {
                    self.cold_misses += 1;
                    let mut penalty = self.miss_penalty;
                    if !self.data_cache.within_line(addr, 2) {
                        penalty += self.miss_penalty;
                    }

                    self.set_if_idle(WriteShort(addr, value, false), penalty)
                }
                cache::Status::Disabled => {
                    let mut penalty = self.miss_penalty;
                    if !self.data_cache.within_line(addr, 2) {
                        penalty += self.miss_penalty;
                    }

                    self.set_if_idle(WriteShort(addr, value, false), penalty)
                }
            }
        }
    }

    fn write_word(&mut self, addr: Word, value: Word) -> Status {
        if self.current_transaction.busy_data() {
            Busy(self.clocks)
        } else {
            match self.data_cache.write_word(addr, value) {
                cache::Status::Hit => {
                    self.accesses += 1;

                    Status::Idle
                }
                cache::Status::Conflict => {
                    self.misses += 1;
                    let mut penalty = self.miss_penalty;
                    if addr % 4 != 0 {
                        penalty += 1 + self.miss_penalty;
                    }

                    self.set_if_idle(WriteWord(addr, value, false), penalty)
                }
                cache::Status::Cold => {
                    self.cold_misses += 1;
                    let mut penalty = self.miss_penalty;
                    if addr % 4 != 0 {
                        penalty += 1 + self.miss_penalty;
                    }

                    self.set_if_idle(WriteWord(addr, value, false), penalty)
                }
                cache::Status::Disabled => {
                    let mut penalty = self.miss_penalty;
                    if addr % 4 != 0 {
                        penalty += 1 + self.miss_penalty;
                    }

                    self.set_if_idle(WriteWord(addr, value, false), penalty)
                }
            }
        }
    }

    fn write_byte_volatile(&mut self, addr: Word, value: Byte) -> Status {
        if self.current_transaction.is_busy() {
            Busy(self.clocks)
        } else {
            self.set_if_idle(WriteByte(addr, value, true), self.volatile_penalty)
        }
    }

    fn write_short_volatile(&mut self, addr: Word, value: Short) -> Status {
        if self.current_transaction.is_busy() {
            Busy(self.clocks)
        } else {
            let mut penalty = self.volatile_penalty;
            if addr % 2 == 0 {
                penalty += self.volatile_penalty;
            }

            self.set_if_idle(WriteShort(addr, value, true), penalty)
        }
    }

    fn write_word_volatile(&mut self, addr: Word, value: Word) -> Status {
        if self.current_transaction.is_busy() {
            Busy(self.clocks)
        } else {
            let mut penalty = self.volatile_penalty;
            if addr % 4 == 0 {
                penalty += self.volatile_penalty * 3;
            }

            self.set_if_idle(WriteWord(addr, value, true), penalty)
        }
    }

    fn cold_misses(&self) -> usize {
        self.cold_misses
    }

    fn cache_misses(&self) -> usize {
        self.misses + self.cold_misses
    }

    fn cache_hits(&self) -> usize {
        self.accesses - (self.cold_misses + self.misses)
    }

    fn accesses(&self) -> usize {
        self.cold_misses + self.misses + self.accesses
    }

    fn cache_state(&self) -> Vec<(&'static str, Vec<Option<LineData>>)> {
        vec![
            ("data cache", self.data_cache.get_lines()),
            ("instruction cache", self.instruction_cache.get_lines()),
        ]
    }

    fn memory(&self) -> &Memory {
        &self.memory
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
        volatile_penalty: usize,
        writethrough: bool,
    ) -> Self {
        Self::new_with_boxed(
            Box::new(data_cache),
            Box::new(instruction_cache),
            memory,
            miss_penalty,
            volatile_penalty,
            writethrough,
        )
    }

    pub fn new_with_boxed(
        data_cache: Box<dyn Cache>,
        instruction_cache: Box<dyn Cache>,
        memory: Memory,
        miss_penalty: usize,
        volatile_penalty: usize,
        writethrough: bool,
    ) -> Self {
        Self {
            data_cache,
            instruction_cache,
            memory,

            miss_penalty,
            volatile_penalty,
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
    fn set_if_idle(&mut self, transaction: Transaction, clocks: usize) -> Status {
        if !self.current_transaction.is_busy() {
            self.clocks = clocks;
            self.current_transaction = transaction;
        }
        Busy(self.clocks)
    }
}
