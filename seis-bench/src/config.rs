use std::{
    error::Error,
    path::{Path, PathBuf},
};
use libmem::{
    cache::{Associative, Cache, MultiAssociative, NullCache},
    memory::Memory,
    module::SingleLevel,
};
use libpipe::{Pipeline, Pipelined, Unpipelined};
use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SimulationConfig {
    pub writethrough: bool,
    pub miss_penalty: usize,
    pub volatile_penalty: usize,
    pub cache: CacheConfig,
}

impl SimulationConfig {
    pub fn build_config(&self, pipeline_enable: bool, cache_enable: bool) -> Box<dyn Pipeline> {
        let data_cache = self.cache.data.build_config(cache_enable);
        let instruction_cache = self.cache.instruction.build_config(cache_enable);

        let mem = Box::new(SingleLevel::new(
            data_cache,
            instruction_cache,
            Memory::new(super::PAGES),
            self.miss_penalty,
            self.volatile_penalty,
            self.writethrough,
        ));

        if pipeline_enable {
            Box::new(Pipelined::new(mem))
        } else {
            Box::new(Unpipelined::new(mem))
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct CacheModuleConfig {
    pub offset_bits: usize,
    pub set_bits: usize,
    pub ways: usize,
}

impl CacheModuleConfig {
    pub fn build_config(&self, cache_enable: bool) -> Box<dyn Cache> {
        if cache_enable {
            if self.ways == 1 {
                Box::new(Associative::new(self.offset_bits, self.set_bits))
            } else {
                Box::new(MultiAssociative::new(
                    self.offset_bits,
                    self.set_bits,
                    self.ways,
                ))
            }
        } else {
            Box::new(NullCache::new())
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct CacheConfig {
    pub instruction: CacheModuleConfig,
    pub data: CacheModuleConfig,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Benchmark {
    pub name: String,
    pub path: PathBuf,
    pub sources: Vec<String>,
    pub binary: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct BenchmarkConfig {
    pub configuration: SimulationConfig,
    pub benchmark: Vec<Benchmark>,
}

pub fn read_configuration(file: &Path) -> Result<BenchmarkConfig, Box<dyn Error>> {
    let content = std::fs::read_to_string(file)?;

    Ok(toml::from_str(&content)?)
}
