mod asm;
mod config;
mod regfp32;
mod stats;

pub use asm::*;
pub use config::*;
use libpipe::Pipeline;
use libseis::{
    instruction_set::{Decode, Instruction},
    pages::PAGE_SIZE,
    types::Word,
};
use regfp32::RegsFp32;
use serde::{Serialize, Serializer};
use serde_wasm_bindgen::Serializer as JsSer;
use stats::Statistics;
use std::{
    collections::HashMap,
    hash::{BuildHasher, RandomState},
};
use wasm_bindgen::prelude::*;

fn to_object<'a>(value: &impl Serialize) -> Result<JsValue, <&'a JsSer as Serializer>::Error> {
    value.serialize(&JsSer::json_compatible())
}

const PAGES: usize = 16;
const REGIONS_PER_PAGE: usize = 4;
const REGION_SIZE: usize = PAGE_SIZE / REGIONS_PER_PAGE;

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, Serialize)]
pub struct SimulatorMemoryConfiguration {
    pub page_count: usize,
    pub page_size: usize,
    pub regions: usize,
    pub region_size: usize,
}

#[wasm_bindgen]
impl SimulatorMemoryConfiguration {
    pub fn get() -> Self {
        static CONF: SimulatorMemoryConfiguration = SimulatorMemoryConfiguration {
            page_count: PAGES,
            page_size: PAGE_SIZE,
            regions: REGIONS_PER_PAGE * PAGES,
            region_size: REGION_SIZE,
        };
        CONF
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryType {
    Byte,
    Short,
    Word,
    Float,
}

#[wasm_bindgen]
#[derive(Debug)]
pub struct SimulationState {
    hashstate: RandomState,
    state: Box<dyn Pipeline>,
    configuration: SimulationConfiguration,

    clocks: usize,
    clock_req: usize,
}

#[wasm_bindgen]
impl SimulationState {
    #[wasm_bindgen(constructor)]
    pub fn new(config: &SimulationConfiguration, asm: Vec<u8>) -> Result<SimulationState, JsError> {
        let mut state = config.into_boxed_pipeline();

        if asm.len() > PAGES * libseis::pages::PAGE_SIZE {
            return Err(JsError::new("The assembly data is too large"));
        }

        let mem = state.memory_module_mut().memory_mut();

        for (b, a) in asm.into_iter().zip(0..) {
            mem.write_byte(a, b);
        }

        Ok(Self {
            hashstate: RandomState::new(),
            state,
            configuration: config.clone(),
            clocks: 0,
            clock_req: 1,
        })
    }

    pub fn get_configuration(&self) -> SimulationConfiguration {
        self.configuration.clone()
    }

    #[inline]
    pub fn is_done(&self) -> bool {
        self.clock_req == 0
    }

    pub fn clock(&mut self) -> bool {
        if !self.is_done() {
            self.clocks += 1;
            self.clock_req = match self.state.clock(1) {
                libpipe::ClockResult::Stall(clk) => clk,
                libpipe::ClockResult::Flow => 1,
                libpipe::ClockResult::Dry => 0,
            };

            self.is_done()
        } else {
            true
        }
    }

    pub fn step(&mut self) -> bool {
        if !self.is_done() {
            self.clocks += self.clock_req;
            self.clock_req = match self.state.clock(self.clock_req) {
                libpipe::ClockResult::Stall(clk) => clk,
                libpipe::ClockResult::Flow => 1,
                libpipe::ClockResult::Dry => 0,
            };

            self.is_done()
        } else {
            true
        }
    }

    pub fn run(&mut self) {
        while !self.step() {}
    }

    /// Reads a value from an address
    pub fn read_address(&self, address: Word, memory_type: MemoryType) -> Result<JsValue, JsError> {
        let mem = self.state.memory_module().memory();

        if PAGES * PAGE_SIZE > address as usize {
            Ok(match memory_type {
                MemoryType::Byte => JsValue::from(mem.read_byte(address)),
                MemoryType::Short => JsValue::from(mem.read_short(address)),
                MemoryType::Word => JsValue::from(mem.read_word(address)),
                MemoryType::Float => JsValue::from(f32::from_bits(mem.read_word(address))),
            })
        } else {
            Err(JsError::new("Address is out of bounds"))
        }
    }

    pub fn read_pipeline_state(&self) -> JsValue {
        to_object(&self.state.stages()).unwrap()
    }

    pub fn read_cache_state(&self) -> JsValue {
        to_object(
            &self
                .state
                .memory_module()
                .cache_state()
                .into_iter()
                .map(|c| (c.name, c.lines))
                .collect::<HashMap<_, _>>(),
        )
        .unwrap()
    }

    pub fn read_registers(&self) -> JsValue {
        to_object(&self.state.registers()).unwrap()
    }

    pub fn read_registers_fp32(&self) -> JsValue {
        to_object(&(RegsFp32::from(self.state.registers().clone()))).unwrap()
    }

    pub fn get_region_hash(&self, region_id: usize) -> Result<String, JsError> {
        let page_id = region_id / REGIONS_PER_PAGE;
        let region_id = region_id % REGIONS_PER_PAGE;
        let region_start = REGION_SIZE * region_id;
        let region_end = REGION_SIZE * (region_id + 1);

        self.state
            .memory_module()
            .memory()
            .get_page(page_id)
            .map(|p| {
                self.hashstate
                    .hash_one(&p[region_start..region_end])
                    .to_string()
            })
            .ok_or_else(|| JsError::new("Failed to get page"))
    }

    pub fn read_region(&self, region_id: usize) -> Result<Vec<u8>, JsError> {
        let page_id = region_id / REGIONS_PER_PAGE;
        let region_id = region_id % REGIONS_PER_PAGE;
        let region_start = REGION_SIZE * region_id;
        let region_end = REGION_SIZE * (region_id + 1);

        self.state
            .memory_module()
            .memory()
            .get_page(page_id)
            .map(|p| Vec::from(&p[region_start..region_end]))
            .ok_or_else(|| JsError::new("Failed to get page"))
    }

    pub fn disassemble_region(&self, region_id: usize) -> Result<Vec<String>, JsError> {
        let page_id = region_id / REGIONS_PER_PAGE;
        let region_id = region_id % REGIONS_PER_PAGE;
        let region_start = REGION_SIZE * region_id;
        let region_end = REGION_SIZE * (region_id + 1);

        self.state
            .memory_module()
            .memory()
            .get_page(page_id)
            .map(|p| {
                p[region_start..region_end]
                    .chunks(4)
                    .map(|c| {
                        Instruction::decode(Word::from_be_bytes([c[0], c[1], c[2], c[3]]))
                            .map(|i| i.to_string())
                            .unwrap_or_else(|_| "<unknown>".to_string())
                    })
                    .collect()
            })
            .ok_or_else(|| JsError::new("Failed to get page"))
    }

    pub fn get_stats(&self) -> JsValue {
        let mem_module = self.state.memory_module();

        to_object(&Statistics {
            clocks: self.clocks,
            memory_accesses: mem_module.accesses(),
            cache_hits: mem_module.cache_hits(),
            cache_conflict_misses: mem_module.conflict_misses(),
            cache_cold_misses: mem_module.cold_misses(),
        })
        .unwrap()
    }
}
