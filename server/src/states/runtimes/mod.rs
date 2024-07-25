use crate::config::SimulationConfiguration;
use libpipe::Pipeline;
use libseis::types::Word;
use serde::{de::Visitor, Deserialize, Serialize};
use std::{
    collections::HashMap,
    hash::RandomState,
    str::FromStr,
    sync::{Arc, Mutex},
    time::Instant,
};
use tokio::sync::RwLock;
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

impl<'de> Deserialize<'de> for AddressType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct _Visitor;
        impl<'de> Visitor<'de> for _Visitor {
            type Value = AddressType;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "byte, short, word, float, or remove")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                AddressType::from_str(&v).map_err(|e| E::custom(e))
            }
        }
        deserializer.deserialize_str(_Visitor)
    }
}

impl Serialize for AddressType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Debug)]
pub struct Runtime {
    pub uuid: Uuid,
    pub created: Instant,
    pub hash_state: RandomState,

    pub watchlist: HashMap<Word, AddressType>,

    pub state: Box<dyn Pipeline + Send + Sync>,
    pub clocks: usize,

    pub config: SimulationConfiguration,
}

impl Runtime {
    pub fn new(
        uuid: Uuid,
        config: SimulationConfiguration,
        bin: Vec<u8>,
    ) -> (Arc<RwLock<Self>>, Mutex<Instant>) {
        let now = Instant::now();

        let mut state = config.into_boxed_pipeline();

        let mem = state.memory_module_mut().memory_mut();

        for (address, byte) in bin.into_iter().enumerate() {
            mem.write_byte(address as u32, byte);
        }

        (
            Arc::new(RwLock::new(Self {
                uuid,
                created: now,
                hash_state: RandomState::new(),

                watchlist: HashMap::new(),

                state,
                clocks: 0,

                config,
            })),
            Mutex::new(now),
        )
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
        use libseis::registers::NAME;
        use serde_json::Map;

        let mut value = Map::default();

        for (id, &val) in self.state.registers().iter().enumerate() {
            let mut values = Map::default();
            values.insert("integer".to_owned(), val.into());
            values.insert("float".to_owned(), f32::from_bits(val).into());

            value.insert(NAME[id].to_string(), values.into());
        }

        value.into()
    }
}
