use crate::PAGES;
use libmem::{
    cache::{Associative, Cache, MultiAssociative, NullCache},
    memory::Memory,
    module::SingleLevel,
};
use libpipe::{Pipeline, Pipelined, Unpipelined};
use std::{collections::HashMap, error::Error, fmt::Display, path::PathBuf, str::FromStr};

#[derive(Debug, Clone, Copy)]
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
                if ways == 1 {
                    Box::new(Associative::new(offset_bits, set_bits))
                } else {
                    Box::new(MultiAssociative::new(offset_bits, set_bits, ways))
                }
            }
        }
    }

    pub fn from_toml(table: &toml::Table) -> Result<Self, Box<dyn Error>> {
        match table
            .get("mode")
            .ok_or_else(|| "Key required: \"mode\"")?
            .as_str()
            .ok_or_else(|| "Expected string")?
        {
            "disabled" => Ok(Self::Disabled),
            "associative" => {
                let set_bits = table
                    .get("set_bits")
                    .ok_or_else(|| "Key required: \"set_bits\"")?
                    .as_integer()
                    .ok_or_else(|| "Expected integer")? as usize;
                let offset_bits = table
                    .get("offset_bits")
                    .ok_or_else(|| "Key required: \"offset_bits\"")?
                    .as_integer()
                    .ok_or_else(|| "Expected integer")? as usize;
                let ways = table
                    .get("ways")
                    .ok_or_else(|| "Key required: \"ways\"")?
                    .as_integer()
                    .ok_or_else(|| "Expected integer")? as usize;

                Ok(Self::Associative {
                    set_bits,
                    offset_bits,
                    ways,
                })
            }

            mode => Err(format!("Unrecognized mode: {mode}").into()),
        }
    }

    pub fn to_toml(self) -> toml::Table {
        let mut table = toml::Table::new();

        match self {
            CacheConfiguration::Disabled => {
                table.insert("mode".to_string(), "disabled".into());
            }
            CacheConfiguration::Associative {
                set_bits,
                offset_bits,
                ways,
            } => {
                table.insert("mode".to_string(), "associative".into());
                table.insert("set_bits".to_string(), (set_bits as i64).into());
                table.insert("offset_bits".to_string(), (offset_bits as i64).into());
                table.insert("ways".to_string(), (ways as i64).into());
            }
        }

        table
    }

    pub fn to_json(self) -> serde_json::Map<String, serde_json::Value> {
        let mut map = serde_json::Map::new();

        match self {
            CacheConfiguration::Disabled => {
                map.insert("mode".to_string(), "Disabled".into());
            }
            CacheConfiguration::Associative {
                set_bits,
                offset_bits,
                ways,
            } => {
                map.insert("mode".to_string(), "Associative".into());
                map.insert("set_bits".to_string(), set_bits.into());
                map.insert("offset_bits".to_string(), offset_bits.into());
                map.insert("ways".to_string(), ways.into());
            }
        }

        map
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum PipelineMode {
    #[default]
    Disabled,
    Enabled,
}

impl Display for PipelineMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PipelineMode::Disabled => write!(f, "disabled"),
            PipelineMode::Enabled => write!(f, "enabled"),
        }
    }
}

impl FromStr for PipelineMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "disabled" => Ok(Self::Disabled),
            "enabled" => Ok(Self::Enabled),
            _ => Err(format!("{s} is not a recognized pipelining mode")),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SimulationConfiguration {
    pub cache: HashMap<String, CacheConfiguration>,

    pub miss_penalty: usize,
    pub volatile_penalty: usize,
    pub writethrough: bool,

    pub pipelining: PipelineMode,
}

impl SimulationConfiguration {
    pub fn into_boxed_pipeline(self) -> Box<dyn Pipeline> {
        let data_config = self
            .cache
            .get("data")
            .expect("Expected a configuration for the data cache");
        let instruction_config = self
            .cache
            .get("instruction")
            .expect("Expected a configuration for the instruction cache");

        let mem = SingleLevel::new(
            data_config.into_boxed_cache(),
            instruction_config.into_boxed_cache(),
            Memory::new(PAGES),
            self.miss_penalty,
            self.volatile_penalty,
            self.writethrough,
        );

        match self.pipelining {
            PipelineMode::Disabled => Box::new(Unpipelined::new(Box::new(mem))),
            PipelineMode::Enabled => Box::new(Pipelined::new(Box::new(mem))),
        }
    }

    pub fn from_toml(table: &toml::Table) -> Result<Self, Box<dyn Error>> {
        let mut result = SimulationConfiguration::default();

        result.miss_penalty = table
            .get("miss_penalty")
            .ok_or_else(|| "Key required: \"miss_penalty\"")?
            .as_integer()
            .ok_or_else(|| "Expected integer")? as usize;
        result.volatile_penalty = table
            .get("volatile_penalty")
            .ok_or_else(|| "Key required: \"volatile_penalty\"")?
            .as_integer()
            .ok_or_else(|| "Expected integer")? as usize;
        result.writethrough = table
            .get("writethrough")
            .ok_or_else(|| "Key required: \"writethrough\"")?
            .as_bool()
            .ok_or_else(|| "Expected boolean")?;
        result.pipelining = if let Some(value) = table.get("pipelining") {
            if let Some(str) = value.as_str() {
                str.parse()?
            } else if let Some(mode) = value.as_bool() {
                if mode {
                    PipelineMode::Enabled
                } else {
                    PipelineMode::Disabled
                }
            } else {
                return Err("Key \"pipelining\" must be either a string or a boolean".into());
            }
        } else {
            PipelineMode::default()
        };

        result.cache = table
            .get("cache")
            .ok_or_else(|| "Key required: \"cache\"")?
            .as_table()
            .ok_or_else(|| "Expected table")?
            .into_iter()
            .map(|(key, value)| {
                Ok((
                    key.to_owned(),
                    CacheConfiguration::from_toml(
                        value.as_table().ok_or_else(|| "Expected table")?,
                    )?,
                ))
            })
            .collect::<Result<HashMap<String, CacheConfiguration>, Box<dyn Error>>>()?;

        Ok(result)
    }

    pub fn to_toml(&self) -> toml::Table {
        let mut table = toml::Table::new();

        table.insert(
            "miss_penalty".to_string(),
            (self.miss_penalty as i64).into(),
        );
        table.insert(
            "volatile_penalty".to_string(),
            (self.volatile_penalty as i64).into(),
        );
        table.insert("writethrough".to_string(), self.writethrough.into());
        table.insert(
            "pipelining".to_string(),
            match self.pipelining {
                PipelineMode::Disabled => false.into(),
                PipelineMode::Enabled => true.into(),
                // _ => self.pipelining.to_string().into(),
            },
        );

        let mut caches = toml::Table::new();

        for (name, config) in &self.cache {
            caches.insert(name.to_owned(), config.to_toml().into());
        }

        table.insert("cache".to_string(), caches.into());

        table
    }

    pub fn from_toml_file(file: PathBuf) -> Result<Self, Box<dyn Error>> {
        let fdata = std::fs::read_to_string(file)?;
        let table = toml::from_str(&fdata)?;
        Self::from_toml(&table)
    }
}
