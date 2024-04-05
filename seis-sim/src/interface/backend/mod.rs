mod cmd;

use self::cmd::{Command, Info};
use super::Interface;
use crate::{config::SimulationConfiguration, PAGES};
use clap::Parser;
use libpipe::{ClockResult, Pipeline};
use libseis::pages::PAGE_SIZE;
use serde_json as json;
use std::{
    error::Error,
    io::{stdin, BufRead},
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

            clocks: 0,
            clocks_required: 1,
            finished: false,
        };

        loop {
            let mut command = String::new();
            stdin().lock().read_line(&mut command)?;

            match Command::try_parse_from(command.split_whitespace()) {
                Ok(command) => {
                    if state.execute(command)? {
                        continue;
                    } else {
                        break Ok(());
                    }
                }
                Err(e) => eprintln!("{e}"),
            }
        }
    }
}

struct BackendState {
    pipeline: Box<dyn Pipeline>,
    config: SimulationConfiguration,

    clocks: usize,
    clocks_required: usize,
    finished: bool,
}

impl BackendState {
    fn execute(&mut self, command: Command) -> Result<bool, Box<dyn Error>> {
        use Command::*;

        match command {
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

                        last = now;
                    }

                    // TODO: make nonblocking
                    let mut input = String::new();
                    stdin().lock().read_line(&mut input)?;
                    let command = Command::try_parse_from(input.split_whitespace())?;

                    if matches!(command, Command::Stop {}) {
                        break;
                    } else if matches!(command, Command::Terminate {}) {
                        return Ok(false);
                    }
                }

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

                    let mut input = String::new();
                    stdin().lock().read_line(&mut input)?;
                    let command = Command::try_parse_from(input.split_whitespace())?;

                    if matches!(command, Command::Stop {}) {
                        break;
                    } else if matches!(command, Command::Terminate {}) {
                        return Ok(false);
                    }
                }

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
            ShowRegs {} => {
                self.show_registers()?;
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
        }
    }

    fn show_pipeline(&self) -> Result<(), Box<dyn Error>> {
        println!("{}", json::to_string(&self.pipeline.stages())?);

        Ok(())
    }

    fn show_page(&self, page: usize) -> Result<(), Box<dyn Error>> {
        let mut map = json::Map::new();

        if let Some(page) = self.pipeline.memory_module().memory().get_page(page) {
            map.insert("allocated".to_string(), true.into());
            map.insert("data".to_string(), page.to_vec().into());
        } else {
            map.insert("allocated".to_string(), false.into());
        }

        println!("{}", json::to_string(&map)?);

        Ok(())
    }

    fn show_registers(&self) -> Result<(), Box<dyn Error>> {
        let mut map = json::Map::new();
        let regs = self.pipeline.registers();

        map.insert("v0".to_string(), regs.v[0].into());
        map.insert("v1".to_string(), regs.v[1].into());
        map.insert("v2".to_string(), regs.v[2].into());
        map.insert("v3".to_string(), regs.v[3].into());
        map.insert("v4".to_string(), regs.v[4].into());
        map.insert("v5".to_string(), regs.v[5].into());
        map.insert("v6".to_string(), regs.v[6].into());
        map.insert("v7".to_string(), regs.v[7].into());
        map.insert("v8".to_string(), regs.v[8].into());
        map.insert("v9".to_string(), regs.v[9].into());
        map.insert("va".to_string(), regs.v[10].into());
        map.insert("vb".to_string(), regs.v[11].into());
        map.insert("vc".to_string(), regs.v[12].into());
        map.insert("vd".to_string(), regs.v[13].into());
        map.insert("ve".to_string(), regs.v[14].into());
        map.insert("vf".to_string(), regs.v[15].into());
        map.insert("sp".to_string(), regs.sp.into());
        map.insert("bp".to_string(), regs.bp.into());
        map.insert("lp".to_string(), regs.lp.into());
        map.insert("pc".to_string(), regs.pc.into());
        map.insert("zf".to_string(), regs.zf.into());
        map.insert("of".to_string(), regs.of.into());
        map.insert("eps".to_string(), regs.eps.into());
        map.insert("nan".to_string(), regs.nan.into());
        map.insert("inf".to_string(), regs.inf.into());

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
