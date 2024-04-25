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
