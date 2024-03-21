use super::{CacheData, MemoryModule, Result, Status};
use crate::{
    cache::{self, Cache},
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
    /// The cache is being flushed to memory
    FlushCache,
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
#[derive(Debug)]
pub struct SingleLevel<'a> {
    data_cache: Box<dyn Cache<'a>>,
    instruction_cache: Box<dyn Cache<'a>>,
    memory: Memory,

    read_miss_penalty: usize,
    write_miss_penalty: usize,
    volatile_penalty: usize,
    writethrough: bool,

    current_transaction: Transaction,
    clocks: usize,

    cold_misses: usize,
    conflict_misses: usize,
    hits: usize,
    accesses: usize,
    last_access_miss: bool,
    evictions: usize,
}

impl<'a> MemoryModule for SingleLevel<'a> {
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
                        if self.data_cache.check_address(addr + 1).is_miss() {
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
                        if self.data_cache.check_address(addr + 3).is_miss() {
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
                    if self.data_cache.check_address(addr + 1).is_miss() {
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
                    if self.data_cache.check_address(addr + 3).is_miss() {
                        if self
                            .data_cache
                            .write_line(addr + 3, &mut self.memory)
                            .evicted()
                        {
                            self.evictions += 1;
                        }
                    }
                }
                ReadInstruction(addr) => {
                    self.instruction_cache.write_line(addr, &mut self.memory);
                }

                FlushCache => {
                    self.data_cache.flush(&mut self.memory);
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
                self.hit();

                Ok(value)
            }
            Err(cache::Status::Cold) => {
                self.cold();

                Err(self.set_if_idle(ReadByte(addr), self.read_miss_penalty))
            }
            Err(cache::Status::Conflict) => {
                self.conflict();

                Err(self.set_if_idle(ReadByte(addr), self.read_miss_penalty))
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
                self.hit();
                if addr % 2 != 0 {
                    self.hit();
                }

                Ok(value)
            }
            Err(cache::Status::Conflict) => {
                self.conflict();
                let mut penalty = self.read_miss_penalty;
                if addr % 4 != 0 {
                    penalty += 1;
                }
                match self.data_cache.check_address(addr + 1) {
                    cache::Status::Conflict => {
                        self.conflict();
                        penalty += self.read_miss_penalty;
                    }
                    cache::Status::Cold => {
                        self.cold();
                        penalty += self.read_miss_penalty;
                    }
                    _ => (),
                }

                Err(self.set_if_idle(ReadShort(addr), penalty))
            }
            Err(cache::Status::Cold) => {
                self.cold();
                let mut penalty = self.read_miss_penalty;
                if addr % 4 != 0 {
                    penalty += 1;
                }
                match self.data_cache.check_address(addr + 1) {
                    cache::Status::Conflict => {
                        self.conflict();
                        penalty += self.read_miss_penalty;
                    }
                    cache::Status::Cold => {
                        self.cold();
                        penalty += self.read_miss_penalty;
                    }
                    _ => (),
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
                self.hit();
                if addr % 4 != 0 {
                    self.hit();
                }

                Ok(value)
            }
            Err(cache::Status::Conflict) => {
                self.conflict();

                let mut penalty = self.read_miss_penalty;
                if addr % 4 != 0 {
                    penalty += 1;
                }
                match self.data_cache.check_address(addr + 3) {
                    cache::Status::Conflict => {
                        self.conflict();
                        penalty += self.read_miss_penalty;
                    }
                    cache::Status::Cold => {
                        self.cold();
                        penalty += self.read_miss_penalty;
                    }
                    _ => (),
                }

                Err(self.set_if_idle(ReadWord(addr), penalty))
            }
            Err(cache::Status::Cold) => {
                self.cold();

                let mut penalty = self.read_miss_penalty;
                if addr % 4 != 0 {
                    penalty += 1;
                }
                match self.data_cache.check_address(addr + 3) {
                    cache::Status::Conflict => {
                        self.conflict();
                        penalty += self.read_miss_penalty;
                    }
                    cache::Status::Cold => {
                        self.cold();
                        penalty += self.read_miss_penalty;
                    }
                    _ => (),
                }

                Err(self.set_if_idle(ReadWord(addr), penalty))
            }
            Err(cache::Status::Disabled) => {
                let mut penalty = self.volatile_penalty;
                if addr % 4 != 0 {
                    self.hit();
                    penalty += self.volatile_penalty;
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
                self.hit();

                Ok(value)
            }
            Err(cache::Status::Cold) => {
                self.cold();

                Err(self.set_if_idle(ReadInstruction(addr), self.read_miss_penalty))
            }
            Err(cache::Status::Conflict) => {
                self.conflict();

                Err(self.set_if_idle(ReadInstruction(addr), self.read_miss_penalty))
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
                    self.hit();

                    Status::Idle
                }
                cache::Status::Conflict => {
                    self.conflict();

                    self.set_if_idle(WriteByte(addr, value, false), self.write_miss_penalty)
                }
                cache::Status::Cold => {
                    self.cold();

                    self.set_if_idle(WriteByte(addr, value, false), self.write_miss_penalty)
                }
                cache::Status::Disabled => {
                    self.set_if_idle(WriteByte(addr, value, false), self.volatile_penalty)
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
                    self.hit();
                    if !self.data_cache.within_line(addr, 2) {
                        self.hit();
                    }

                    Status::Idle
                }
                cache::Status::Conflict => {
                    self.conflict();
                    let mut penalty = self.write_miss_penalty;
                    if addr % 2 != 0 {
                        penalty += 1;
                    }
                    match self.data_cache.check_address(addr + 1) {
                        cache::Status::Conflict => {
                            self.conflict();
                            if !self.writethrough {
                                penalty += self.write_miss_penalty;
                            } else {
                                penalty += self.volatile_penalty
                            }
                        }
                        cache::Status::Cold => {
                            self.cold();
                            if !self.writethrough {
                                penalty += self.write_miss_penalty;
                            } else {
                                penalty += self.volatile_penalty
                            }
                        }
                        _ => (),
                    }

                    self.set_if_idle(WriteShort(addr, value, false), penalty)
                }
                cache::Status::Cold => {
                    self.cold();
                    let mut penalty = self.write_miss_penalty;
                    if addr % 2 != 0 {
                        penalty += 1;
                    }
                    match self.data_cache.check_address(addr + 1) {
                        cache::Status::Conflict => {
                            self.conflict();
                            if !self.writethrough {
                                penalty += self.write_miss_penalty;
                            } else {
                                penalty += self.volatile_penalty
                            }
                        }
                        cache::Status::Cold => {
                            self.cold();
                            if !self.writethrough {
                                penalty += self.write_miss_penalty;
                            } else {
                                penalty += self.volatile_penalty
                            }
                        }
                        _ => (),
                    }

                    self.set_if_idle(WriteShort(addr, value, false), penalty)
                }
                cache::Status::Disabled => {
                    let mut penalty = self.volatile_penalty;
                    if addr % 2 != 0 {
                        penalty += self.volatile_penalty;
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
                    self.hit();
                    if !self.data_cache.within_line(addr, 4) {
                        self.hit();
                    }

                    Status::Idle
                }
                cache::Status::Conflict => {
                    self.conflict();
                    let mut penalty = self.write_miss_penalty;
                    if addr % 4 != 0 {
                        penalty += 1;
                    }
                    match self.data_cache.check_address(addr + 3) {
                        cache::Status::Conflict => {
                            self.conflict();
                            if !self.writethrough {
                                penalty += self.write_miss_penalty;
                            } else {
                                penalty += self.volatile_penalty
                            }
                        }
                        cache::Status::Cold => {
                            self.cold();
                            if !self.writethrough {
                                penalty += self.write_miss_penalty;
                            } else {
                                penalty += self.volatile_penalty
                            }
                        }
                        _ => (),
                    }

                    self.set_if_idle(WriteWord(addr, value, false), penalty)
                }
                cache::Status::Cold => {
                    self.cold();
                    let mut penalty = self.write_miss_penalty;
                    if addr % 4 != 0 {
                        penalty += 1;
                    }
                    match self.data_cache.check_address(addr + 3) {
                        cache::Status::Conflict => {
                            self.conflict();
                            if !self.writethrough {
                                penalty += self.write_miss_penalty;
                            } else {
                                penalty += self.volatile_penalty
                            }
                        }
                        cache::Status::Cold => {
                            self.cold();
                            if !self.writethrough {
                                penalty += self.write_miss_penalty;
                            } else {
                                penalty += self.volatile_penalty
                            }
                        }
                        _ => (),
                    }

                    self.set_if_idle(WriteWord(addr, value, false), penalty)
                }
                cache::Status::Disabled => {
                    let mut penalty = self.volatile_penalty;
                    if addr % 4 != 0 {
                        penalty += 1 + self.volatile_penalty;
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

    fn conflict_misses(&self) -> usize {
        self.conflict_misses
    }

    fn cache_hits(&self) -> usize {
        self.hits
    }

    fn accesses(&self) -> usize {
        self.accesses
    }

    fn memory(&self) -> &Memory {
        &self.memory
    }

    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.memory
    }

    fn cache_state(&self) -> Vec<CacheData> {
        vec![
            ("data cache", self.data_cache.get_lines()).into(),
            ("instruction cache", self.instruction_cache.get_lines()).into(),
        ]
    }

    fn flush_cache(&mut self) -> Status {
        if self.current_transaction.is_busy() {
            Busy(self.clocks)
        } else {
            let count = self.data_cache.dirty_lines();

            if count == 0 {
                Status::Idle
            } else {
                self.clocks = self.read_miss_penalty;
                self.current_transaction = FlushCache;
                Busy(self.read_miss_penalty)
            }
        }
    }
}

impl<'a> SingleLevel<'a> {
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
    pub fn new<T: Cache<'a> + 'static, U: Cache<'a> + 'static>(
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
        data_cache: Box<dyn Cache<'a>>,
        instruction_cache: Box<dyn Cache<'a>>,
        memory: Memory,
        miss_penalty: usize,
        volatile_penalty: usize,
        writethrough: bool,
    ) -> Self {
        Self {
            data_cache,
            instruction_cache,
            memory,

            read_miss_penalty: miss_penalty,
            write_miss_penalty: if writethrough {
                volatile_penalty
            } else {
                miss_penalty
            },
            volatile_penalty,
            writethrough,

            current_transaction: Idle,
            clocks: 0,

            cold_misses: 0,
            conflict_misses: 0,
            hits: 0,
            accesses: 0,
            last_access_miss: false,
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

    /// Increments the hit counter if the last access was not a miss, otherwise unsets
    /// the miss flag
    fn hit(&mut self) {
        if self.last_access_miss {
            self.last_access_miss = false;
        } else {
            self.accesses += 1;
            self.hits += 1;
        }
    }

    /// Increments the cold miss counter and sets the miss flag if the memory subsystem
    /// is not busy reading data
    fn cold(&mut self) {
        if !self.current_transaction.is_busy() {
            self.accesses += 1;
            self.cold_misses += 1;
            self.last_access_miss = true;
        }
    }

    /// Increments the conflict miss counter and sets the miss flag if the memory
    /// subsystem is not busy reading data
    fn conflict(&mut self) {
        if !self.current_transaction.is_busy() {
            self.accesses += 1;
            self.conflict_misses += 1;
            self.last_access_miss = true;
        }
    }
}
