use libmem::{
    cache::{Associative, Cache, MultiAssociative, NullCache},
    memory::Memory,
    module::SingleLevel,
};
use libpipe::{Pipeline, Pipelined, Unpipelined};
use serde::Deserialize;
use std::{
    error::Error,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SimulationConfig {
    pub name: String,

    pub writethrough: bool,
    pub miss_penalty: usize,
    pub volatile_penalty: usize,
    pub pipeline: bool,
    #[serde(default)]
    pub cache: CacheConfig,
}

impl SimulationConfig {
    pub fn build_config(&self) -> Box<dyn Pipeline> {
        let (data_cache, instruction_cache) = self.cache.build_config();

        let mem = Box::new(SingleLevel::new(
            data_cache,
            instruction_cache,
            Memory::new(super::PAGES),
            self.miss_penalty,
            self.volatile_penalty,
            self.writethrough,
        ));

        if self.pipeline {
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
    pub fn build_config(&self) -> Box<dyn Cache> {
        if self.ways == 1 {
            Box::new(Associative::new(self.offset_bits, self.set_bits))
        } else {
            Box::new(MultiAssociative::new(
                self.offset_bits,
                self.set_bits,
                self.ways,
            ))
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct CacheConfig {
    pub instruction: Option<CacheModuleConfig>,
    pub data: Option<CacheModuleConfig>,
}

impl CacheConfig {
    pub fn build_config(&self) -> (Box<dyn Cache>, Box<dyn Cache>) {
        (
            if let Some(ref conf) = self.instruction {
                conf.build_config()
            } else {
                Box::new(NullCache::new())
            },
            if let Some(ref conf) = self.data {
                conf.build_config()
            } else {
                Box::new(NullCache::new())
            },
        )
    }
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
    pub configuration: Vec<SimulationConfig>,
    pub benchmark: Vec<Benchmark>,
}

pub fn read_configuration(file: &Path) -> Result<BenchmarkConfig, Box<dyn Error>> {
    let content = std::fs::read_to_string(file)?;

    Ok(toml::from_str(&content)?)
}
