use crate::{to_object, PAGES};
use libmem::{
    cache::{Associative, Cache, MultiAssociative, NullCache},
    memory::Memory,
    module::SingleLevel,
};
use libpipe::{Pipeline, Pipelined, Unpipelined};
use serde_json::{Map, Value};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CacheMode {
    #[default]
    Disabled,
    Associative,
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, Default)]
pub struct CacheConfiguration {
    pub mode: CacheMode,
    pub set_bits: usize,
    pub offset_bits: usize,
    pub ways: usize,
}

#[wasm_bindgen]
impl CacheConfiguration {
    #[wasm_bindgen(constructor)]
    pub fn new(
        mode: CacheMode,
        set_bits: usize,
        offset_bits: usize,
        ways: usize,
    ) -> Result<CacheConfiguration, JsError> {
        if mode == CacheMode::Disabled {
            Ok(Self {
                mode,
                set_bits: 0,
                offset_bits: 0,
                ways: 0,
            })
        } else {
            if offset_bits < 2 || offset_bits > 32 {
                return Err(JsError::new("offset_bits must be between 2 and 32"));
            }

            if set_bits > 30 {
                return Err(JsError::new("set_bits must be between 0 and 30"));
            }

            if ways == 0 || ways > 4 {
                return Err(JsError::new("ways must be between 1 and 4"));
            }

            if offset_bits + set_bits > 32 {
                return Err(JsError::new(
                    "offset_bits + set_bits must be less than or equal to 32",
                ));
            }

            Ok(Self {
                mode,
                set_bits,
                offset_bits,
                ways,
            })
        }
    }
}

impl CacheConfiguration {
    pub fn to_json(&self) -> Map<String, Value> {
        let mut map = Map::new();

        match self.mode {
            CacheMode::Disabled => {
                map.insert("mode".to_string(), "disabled".into());
            }
            CacheMode::Associative => {
                map.insert("mode".to_string(), "associative".into());
                map.insert("set_bits".to_string(), self.set_bits.into());
                map.insert("offset_bits".to_string(), self.offset_bits.into());
                map.insert("ways".to_string(), self.ways.into());
            }
        }

        map
    }
}

impl CacheConfiguration {
    pub fn into_boxed_cache(self) -> Box<dyn Cache + Send + Sync> {
        match self.mode {
            CacheMode::Disabled => Box::new(NullCache::new()),
            CacheMode::Associative => {
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
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, Default)]
pub struct SimulationConfiguration {
    pub instruction_cache: CacheConfiguration,
    pub data_cache: CacheConfiguration,

    pub miss_penalty: usize,
    pub volatile_penalty: usize,
    pub writethrough: bool,
    pub pipelining: bool,
}

#[wasm_bindgen]
impl SimulationConfiguration {
    #[wasm_bindgen(constructor)]
    pub fn new(
        miss_penalty: usize,
        volatile_penalty: usize,
        writethrough: bool,
        pipelining: bool,
        instruction_cache: CacheConfiguration,
        data_cache: CacheConfiguration,
    ) -> Result<SimulationConfiguration, JsError> {
        if miss_penalty == 0 {
            return Err(JsError::new("miss_penalty must be greater than 0"));
        }

        if volatile_penalty == 0 {
            return Err(JsError::new("volatile_penalty must be greater than 0"));
        }

        Ok(Self {
            instruction_cache,
            data_cache,

            miss_penalty,
            volatile_penalty,
            writethrough,
            pipelining,
        })
    }
}

impl SimulationConfiguration {
    pub fn into_boxed_pipeline(&self) -> Box<dyn Pipeline + Send + Sync> {
        let mem = SingleLevel::new(
            self.data_cache.into_boxed_cache(),
            self.instruction_cache.into_boxed_cache(),
            Memory::new(PAGES),
            self.miss_penalty,
            self.volatile_penalty,
            self.writethrough,
        );

        if self.pipelining {
            Box::new(Pipelined::new(Box::new(mem)))
        } else {
            Box::new(Unpipelined::new(Box::new(mem)))
        }
    }

    pub fn to_json(&self) -> Value {
        let mut object = Map::default();

        object.insert(
            "miss_penalty".to_string(),
            (self.miss_penalty as u64).into(),
        );
        object.insert(
            "volatile_penalty".to_string(),
            (self.volatile_penalty as u64).into(),
        );
        object.insert("writethrough".to_string(), self.writethrough.into());
        object.insert("pipelining".to_string(), self.pipelining.into());

        let mut caches = Map::new();

        caches.insert(
            "instruction".to_owned(),
            self.instruction_cache.to_json().into(),
        );
        caches.insert("data".to_owned(), self.data_cache.to_json().into());

        object.insert("cache".to_string(), caches.into());

        object.into()
    }
}

#[wasm_bindgen]
impl SimulationConfiguration {
    pub fn as_json(&self) -> JsValue {
        to_object(&self.to_json()).unwrap()
    }
}
