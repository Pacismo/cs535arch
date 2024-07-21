//! Datastructures representing a configuration for the benchmarker to run.
use libmem::{
    cache::{Associative, Cache, MultiAssociative, NullCache},
    memory::Memory,
    module::SingleLevel,
};
use libpipe::{Pipeline, Pipelined, Unpipelined};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use crate::error::Error;

/// A singular configuration for the benchmark.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct SimulationConfig {
    /// The name of the configuration
    pub name: String,

    // Whether to enable writethrough
    pub writethrough: bool,
    /// The penalty of a miss
    pub miss_penalty: usize,
    /// The penalty of a volatile operation
    pub volatile_penalty: usize,
    /// Whether to enable the pipeline
    pub pipeline: bool,
    /// The configuration of a cache
    ///
    /// This field is optional in the file
    #[serde(default)]
    pub cache: CacheConfig,
}

impl SimulationConfig {
    /// Construct a pipeline out of this configuration
    pub fn build_config(&self) -> Box<dyn Pipeline> {
        let (instruction_cache, data_cache) = self.cache.build_config();

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

/// The configuration of a singular cache module
#[derive(Debug, Clone, Default, Deserialize)]
pub struct CacheModuleConfig {
    /// Bits for offset
    pub offset_bits: usize,
    /// Bits for set
    pub set_bits: usize,
    /// The number of ways per set
    pub ways: usize,
}

impl CacheModuleConfig {
    /// Construct a cache out of this configuration
    pub fn build_config(&self) -> Box<dyn Cache + Send + Sync> {
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

/// The configuration of the entire cache
#[derive(Debug, Clone, Default, Deserialize)]
pub struct CacheConfig {
    /// Configuration for the instruction cache
    pub instruction: Option<CacheModuleConfig>,
    /// Configuration for the data cache
    pub data: Option<CacheModuleConfig>,
}

impl CacheConfig {
    /// Create the instruction and data caches
    pub fn build_config(&self) -> (Box<dyn Cache + Send + Sync>, Box<dyn Cache + Send + Sync>) {
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

/// Represents a benchmark
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Benchmark {
    /// The name of the benchmark
    pub name: String,
    /// Where to find the benchmark relative to the configuration file
    pub path: PathBuf,
    /// What source files to include in the build process, relative to the benchmark's root directory
    pub sources: Vec<String>,
    /// What to call the resulting binary file
    pub binary: String,
}

/// The entire configuration for the benchmarking tool
#[derive(Debug, Clone, Default, Deserialize)]
pub struct BenchmarkConfig {
    /// The set of configurations to run for each benchmark
    pub configuration: Vec<SimulationConfig>,
    /// The set of benchmarks
    pub benchmark: Vec<Benchmark>,
}

/// Read a configuration from a file.
///
/// Reads the entire contents of the `file` to memory and deserializes it.
pub fn read_configuration(file: &Path) -> Result<BenchmarkConfig, Error> {
    let content = std::fs::read_to_string(file)?;

    Ok(toml::from_str(&content)?)
}
