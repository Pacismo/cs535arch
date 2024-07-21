use crate::config::SimulationConfiguration;
use libpipe::Pipeline;
use libseis::types::Word;
use std::{collections::HashMap, str::FromStr, sync::Arc, time::Instant};
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Copy)]
pub enum AddressType {
    Byte,
    Short,
    Word,
    Float,
}
impl FromStr for AddressType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "byte" | "b" => Ok(Self::Byte),
            "short" | "s" => Ok(Self::Short),
            "word" | "w" => Ok(Self::Word),
            "float" | "f" => Ok(Self::Float),
            _ => Err(format!("{s} is not a valid type")),
        }
    }
}
impl ToString for AddressType {
    fn to_string(&self) -> String {
        match *self {
            AddressType::Byte => "byte".to_owned(),
            AddressType::Short => "short".to_owned(),
            AddressType::Word => "word".to_owned(),
            AddressType::Float => "float".to_owned(),
        }
    }
}

#[derive(Debug)]
pub struct Runtime {
    pub uuid: Uuid,
    pub created: Instant,
    pub last_used: Instant,

    pub watchlist: HashMap<Word, AddressType>,

    pub state: Box<dyn Pipeline + Send + Sync>,
    pub config: SimulationConfiguration,
}

impl Runtime {
    pub fn new(uuid: Uuid, config: SimulationConfiguration) -> Arc<Mutex<Self>> {
        let now = Instant::now();

        Arc::new(Mutex::new(Self {
            uuid,
            created: now,
            last_used: now,

            watchlist: HashMap::new(),

            state: config.into_boxed_pipeline(),
            config,
        }))
    }

    pub fn read_watchlist(&self) -> HashMap<Word, String> {
        let mem = self.state.memory_module().memory();

        self.watchlist
            .iter()
            .map(|(&a, &t)| match t {
                AddressType::Byte => (a, mem.read_byte(a).to_string()),
                AddressType::Short => (a, mem.read_short(a).to_string()),
                AddressType::Word => (a, mem.read_word(a).to_string()),
                AddressType::Float => (a, f32::from_bits(mem.read_word(a)).to_string()),
            })
            .collect()
    }

    pub fn read_regs(&self) -> serde_json::Value {
        serde_json::to_value(self.state.registers()).unwrap()
    }
}
