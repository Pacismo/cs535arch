use libmem::{
    cache::{Associative, Cache, MultiAssociative, NullCache},
    memory::Memory,
    module::{MemoryModule, SingleLevel},
};
use serde::{Deserialize, Serialize};

use crate::PAGES;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum CacheConfiguration {
    Disabled,
    Associative {
        set_bits: usize,
        offset_bits: usize,
        ways: usize,
    },
}

impl CacheConfiguration {
    pub fn into_boxed_cache(self) -> Box<dyn Cache> {
        match self {
            CacheConfiguration::Disabled => Box::new(NullCache::new()),
            CacheConfiguration::Associative {
                set_bits,
                offset_bits,
                ways,
            } => {
                assert!(
                    offset_bits >= 2,
                    "Must have at least two bits for byte offset"
                );
                assert!(
                    set_bits + offset_bits <= 32,
                    "set_bits + offset_bits must sum to at most 32"
                );

                if ways == 1 {
                    Box::new(Associative::new(offset_bits, set_bits))
                } else {
                    Box::new(MultiAssociative::new(offset_bits, set_bits, ways))
                }
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum MemoryConfiguration {
    SingleLevel {
        instruction_cache: CacheConfiguration,
        data_cache: CacheConfiguration,

        miss_penalty: usize,
        volatile_penalty: usize,
        writethrough: bool,
    },
}

impl MemoryConfiguration {
    pub fn into_boxed_module(self) -> Box<dyn MemoryModule> {
        match self {
            MemoryConfiguration::SingleLevel {
                instruction_cache,
                data_cache,
                miss_penalty,
                volatile_penalty,
                writethrough,
            } => Box::new(SingleLevel::new(
                data_cache.into_boxed_cache(),
                instruction_cache.into_boxed_cache(),
                Memory::new(PAGES),
                miss_penalty,
                volatile_penalty,
                writethrough,
            )),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SimulationConfiguration {
    pub memory_mode: MemoryConfiguration,
}
