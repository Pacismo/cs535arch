mod cli;
mod config;
mod interface;

use clap::Parser;
use cli::{Cli, Configuration, SimulatorConfig};
use config::{CacheConfiguration, PipelineMode, SimulationConfiguration};
use interface::Interface;
use libpipe::Pipeline;
use libseis::{pages::PAGE_SIZE, types::Word};
use std::{error::Error, fs::read, path::PathBuf};

fn into_toml(config: Configuration) -> Result<toml::Table, Box<dyn Error>> {
    if let Some(f) = config.file {
        Ok(toml::from_str(&std::fs::read_to_string(f)?)?)
    } else if let Some(s) = config.inline {
        Ok(toml::from_str(&s)?)
    } else {
        Err("Etiher a string or a file configuration is required".into())
    }
}

/// However many pages of memory are supported by the simulator
const PAGES: usize = 16;

fn prepare_config(
    conf: toml::Table,
    bin: PathBuf,
) -> Result<(Box<dyn Pipeline>, SimulationConfiguration), Box<dyn Error>> {
    let conf = SimulationConfiguration::from_toml(&conf)?;
    let mut pipeline = conf.clone().into_boxed_pipeline();

    let memory = pipeline.memory_module_mut().memory_mut();

    let data = read(&bin)?;

    if data.len() >= PAGES << 16 {
        return Err(format!("File too long: {}", bin.display()).into());
    }

    for (page, data) in data.chunks(PAGE_SIZE).enumerate() {
        memory.set_page((page << 16) as Word, data)
    }

    Ok((pipeline, conf))
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli {
        Cli::Run(SimulatorConfig {
            image_file,
            configuration,
            backend_mode,
        }) => {
            let (pipeline, config) = prepare_config(into_toml(configuration)?, image_file)?;

            if backend_mode {
                interface::Backend.run(pipeline, config)?;
            } else {
                interface::Tui.run(pipeline, config)?;
            }
        }
        Cli::Simulate {
            image_file,
            configuration,
            clock_only,
        } => {
            let (mut pipeline, _) = prepare_config(into_toml(configuration)?, image_file)?;

            let mut clocks = 0;
            let mut clocks_required = 1;

            let start = std::time::Instant::now();
            if clock_only {
                loop {
                    clocks += 1;
                    match pipeline.clock(1) {
                        libpipe::ClockResult::Stall(_) => (),
                        libpipe::ClockResult::Flow => (),
                        libpipe::ClockResult::Dry => break,
                    }
                }
            } else {
                loop {
                    clocks += clocks_required;
                    match pipeline.clock(clocks_required) {
                        libpipe::ClockResult::Stall(n) => clocks_required = n,
                        libpipe::ClockResult::Flow => clocks_required = 1,
                        libpipe::ClockResult::Dry => break,
                    }
                }
            }
            let end = std::time::Instant::now();

            println!("Total clocks: {clocks}");
            println!("Total time: {} seconds", (end - start).as_secs_f64());
        }
        Cli::PrintExampleConfiguration { output_file } => {
            let example = SimulationConfiguration {
                cache: [
                    ("instruction".into(), CacheConfiguration::Disabled),
                    (
                        "data".into(),
                        CacheConfiguration::Associative {
                            set_bits: 2,
                            offset_bits: 2,
                            ways: 2,
                        },
                    ),
                ]
                .into(),
                miss_penalty: 100,
                volatile_penalty: 20,
                writethrough: false,
                pipelining: PipelineMode::Enabled,
            };

            if let Some(out) = output_file {
                std::fs::write(&out, example.to_toml().to_string())?;
            } else {
                println!("{}", example.to_toml());
            }
        }
    }

    Ok(())
}
