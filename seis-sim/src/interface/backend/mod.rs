mod cmd;
mod disassembly;
mod input;

use self::{
    cmd::{Command, Info, Register},
    input::InputHandler,
};
use super::Interface;
use crate::{
    config::SimulationConfiguration, interface::backend::disassembly::DisassemblyRow, PAGES,
};
use clap::Parser;
use libpipe::{ClockResult, Pipeline};
use libseis::{
    instruction_set::{Decode, Instruction},
    pages::PAGE_SIZE,
    types::Word,
};
use serde_json as json;
use std::{
    error::Error,
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Copy)]
pub struct Backend;

impl Interface for Backend {
    type Ok = ();

    type Error = Box<dyn Error>;

    fn run(
        self,
        pipeline: Box<dyn Pipeline>,
        config: SimulationConfiguration,
    ) -> Result<Self::Ok, Self::Error> {
        let mut state = BackendState {
            pipeline,
            config,
            input_handler: InputHandler::new(),

            clocks: 0,
            clocks_required: 1,
            finished: false,
        };

        loop {
            match state.read_input() {
                Ok(true) => continue,
                Ok(false) => break,
                Err(BackendError::ClapError(e)) => eprintln!("{e}"),
                Err(BackendError::OtherError(e)) => return Err(e),
            }
        }

        Ok(())
    }
}

pub enum BackendError {
    ClapError(clap::Error),
    OtherError(Box<dyn Error>),
}
impl From<clap::Error> for BackendError {
    fn from(e: clap::Error) -> Self {
        Self::ClapError(e)
    }
}
impl From<Box<dyn Error>> for BackendError {
    fn from(e: Box<dyn Error>) -> Self {
        Self::OtherError(e)
    }
}
impl From<json::Error> for BackendError {
    fn from(e: json::Error) -> Self {
        Self::OtherError(Box::new(e))
    }
}

struct BackendState {
    pipeline: Box<dyn Pipeline>,
    config: SimulationConfiguration,
    input_handler: InputHandler,

    clocks: usize,
    clocks_required: usize,
    finished: bool,
}

impl BackendState {
    fn read_input(&mut self) -> Result<bool, BackendError> {
        use Command::*;

        match Command::try_parse_from(self.input_handler.get_next().split_whitespace())? {
            DisassemblePage { page } => {
                self.show_disassembled_page(page)?;
                Ok(true)
            }
            Clock { mut count } => {
                while count > 0 && !self.finished {
                    let min = self.clocks_required.min(count);
                    self.clocks += min;
                    count -= min;

                    match self.pipeline.clock(min) {
                        ClockResult::Stall(clocks) => {
                            self.clocks_required = clocks;
                        }
                        ClockResult::Flow => {
                            self.clocks_required = 1;
                        }
                        ClockResult::Dry => {
                            self.finished = true;
                        }
                    }
                }

                Ok(true)
            }
            Run {
                clock_rate: Some(rate_millis),
            } => {
                let mut last = Instant::now();
                while !self.finished {
                    let now = Instant::now();
                    if now.duration_since(last) >= Duration::from_millis(rate_millis) {
                        self.clocks += 1;
                        match self.pipeline.clock(1) {
                            ClockResult::Stall(clocks) => {
                                self.clocks_required = clocks;
                            }
                            ClockResult::Flow => {
                                self.clocks_required = 1;
                            }
                            ClockResult::Dry => {
                                self.finished = true;
                            }
                        }

                        last = now;
                    }

                    if let Some(command) =
                        self.input_handler.get_next_timeout(Duration::from_nanos(0))
                    {
                        let command = Command::try_parse_from(command.split_whitespace());
                        if matches!(command, Ok(Command::Stop {})) {
                            println!("break");
                            return Ok(true);
                        } else if matches!(command, Ok(Command::Terminate {})) {
                            println!("terminated");
                            return Ok(false);
                        }
                    }
                }

                println!("done");

                Ok(true)
            }
            Run { clock_rate: None } => {
                while !self.finished {
                    self.clocks += self.clocks_required;
                    match self.pipeline.clock(self.clocks_required) {
                        ClockResult::Stall(clocks) => {
                            self.clocks_required = clocks;
                        }
                        ClockResult::Flow => {
                            self.clocks_required = 1;
                        }
                        ClockResult::Dry => {
                            self.finished = true;
                        }
                    }

                    if let Some(command) =
                        self.input_handler.get_next_timeout(Duration::from_nanos(0))
                    {
                        let command = Command::try_parse_from(command.split_whitespace());
                        if matches!(command, Ok(Command::Stop {})) {
                            println!("break");
                            return Ok(true);
                        } else if matches!(command, Ok(Command::Terminate {})) {
                            println!("terminated");
                            return Ok(false);
                        }
                    }
                }

                println!("done");

                Ok(true)
            }
            Stop {} => Ok(true),
            Information { what } => {
                self.information(what)?;
                Ok(true)
            }
            ReadPage { page } => {
                self.show_page(page)?;
                Ok(true)
            }
            ShowRegs { regs } => {
                self.show_registers(regs)?;
                Ok(true)
            }
            ShowCache {} => {
                println!(
                    "{}",
                    json::to_string(&self.pipeline.memory_module().cache_state())?
                );
                Ok(true)
            }
            ShowPipeline {} => {
                self.show_pipeline()?;

                Ok(true)
            }
            Statistics {} => {
                self.statistics()?;
                Ok(true)
            }
            Terminate {} => Ok(false),
            Decode { value } => {
                let data: json::Map<String, json::Value> = [(
                    "decoded".to_string(),
                    Instruction::decode(value)
                        .map(|i| i.to_string())
                        .ok()
                        .into(),
                )]
                .into_iter()
                .collect();

                println!("{}", json::to_string(&data)?);

                Ok(true)
            }
        }
    }

    fn show_pipeline(&self) -> Result<(), Box<dyn Error>> {
        println!("{}", json::to_string(&self.pipeline.stages())?);

        Ok(())
    }

    fn show_page(&self, page: usize) -> Result<(), Box<dyn Error>> {
        let mut map = json::Map::new();

        map.insert(
            "data".to_string(),
            self.pipeline
                .memory_module()
                .memory()
                .get_page(page)
                .map(|p| p.to_vec())
                .into(),
        );

        println!("{}", json::to_string(&map)?);

        Ok(())
    }

    fn show_disassembled_page(&self, page: usize) -> Result<(), Box<dyn Error>> {
        let data = self
            .pipeline
            .memory_module()
            .memory()
            .get_page(page)
            .map(|p| {
                p.chunks(4)
                    .enumerate()
                    .map(|(i, b)| DisassemblyRow {
                        address: format!("{:#010X}", i * 4),
                        bytes: [
                            format!("{:02X}", b[0]),
                            format!("{:02X}", b[1]),
                            format!("{:02X}", b[2]),
                            format!("{:02X}", b[3]),
                        ],
                        instruction: Instruction::decode(Word::from_be_bytes([
                            b[0], b[1], b[2], b[3],
                        ]))
                        .unwrap_or_default()
                        .to_string(),
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        println!("{}", json::to_string(&data)?);

        Ok(())
    }

    fn show_registers(&self, regs: Option<Vec<Register>>) -> Result<(), Box<dyn Error>> {
        let mut map = json::Map::new();
        let registers = self.pipeline.registers();

        if let Some(regs) = regs {
            for reg in regs.into_iter() {
                map.insert(reg.to_string(), registers[reg.into()].into());
            }
        } else {
            map.insert("v0".to_string(), registers.v[0].into());
            map.insert("v1".to_string(), registers.v[1].into());
            map.insert("v2".to_string(), registers.v[2].into());
            map.insert("v3".to_string(), registers.v[3].into());
            map.insert("v4".to_string(), registers.v[4].into());
            map.insert("v5".to_string(), registers.v[5].into());
            map.insert("v6".to_string(), registers.v[6].into());
            map.insert("v7".to_string(), registers.v[7].into());
            map.insert("v8".to_string(), registers.v[8].into());
            map.insert("v9".to_string(), registers.v[9].into());
            map.insert("va".to_string(), registers.v[10].into());
            map.insert("vb".to_string(), registers.v[11].into());
            map.insert("vc".to_string(), registers.v[12].into());
            map.insert("vd".to_string(), registers.v[13].into());
            map.insert("ve".to_string(), registers.v[14].into());
            map.insert("vf".to_string(), registers.v[15].into());
            map.insert("sp".to_string(), registers.sp.into());
            map.insert("bp".to_string(), registers.bp.into());
            map.insert("lp".to_string(), registers.lp.into());
            map.insert("pc".to_string(), registers.pc.into());
            map.insert("zf".to_string(), registers.zf.into());
            map.insert("of".to_string(), registers.of.into());
            map.insert("eps".to_string(), registers.eps.into());
            map.insert("nan".to_string(), registers.nan.into());
            map.insert("inf".to_string(), registers.inf.into());
        }

        println!("{}", json::to_string(&map)?);

        Ok(())
    }

    fn statistics(&self) -> Result<(), Box<dyn Error>> {
        let mem = self.pipeline.memory_module();
        let mut map = json::Map::new();

        map.insert("clocks".to_string(), self.clocks.into());
        map.insert("memory_accesses".to_string(), mem.accesses().into());
        map.insert("cache_misses".to_string(), mem.total_misses().into());
        map.insert("cold_misses".to_string(), mem.cold_misses().into());
        map.insert("conflict_misses".to_string(), mem.conflict_misses().into());
        map.insert("cache_hits".to_string(), mem.cache_hits().into());
        map.insert("evictions".to_string(), mem.evictions().into());

        println!("{}", json::to_string(&map)?);

        Ok(())
    }

    fn information(&self, what: Info) -> Result<(), Box<dyn Error>> {
        use json::{to_string, Map, Value};
        use Info::*;
        let mut map = Map::new();

        match what {
            Pages => {
                map.insert("page_count".to_string(), PAGES.into());
                map.insert("page_size".to_string(), PAGE_SIZE.into());
                map.insert(
                    "allocated_pages".to_string(),
                    self.pipeline
                        .memory_module()
                        .memory()
                        .allocated_pages()
                        .count()
                        .into(),
                );
            }
            Cache => {
                let caches = self.pipeline.memory_module().caches();
                map.insert("cache_count".into(), caches.len().into());
                map.insert(
                    "cache_names".into(),
                    caches.keys().map(ToString::to_string).collect(),
                );
            }
            Pipeline => {
                map.insert(
                    "pipeline".to_string(),
                    self.config.pipelining.to_string().into(),
                );
            }
            Configuration => {
                map.insert(
                    "cache_configurations".into(),
                    self.pipeline
                        .memory_module()
                        .caches()
                        .keys()
                        .map(|&k| {
                            (
                                k.to_string(),
                                self.config.cache.get(k).unwrap().to_json().into(),
                            )
                        })
                        .collect::<Map<String, Value>>()
                        .into(),
                );
                map.insert("miss_penalty".to_string(), self.config.miss_penalty.into());
                map.insert(
                    "volatile_penalty".to_string(),
                    self.config.volatile_penalty.into(),
                );
                map.insert("writethrough".to_string(), self.config.writethrough.into());
            }
        }

        println!("{}", to_string(&map)?);
        Ok(())
    }
}
