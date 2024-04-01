mod cli;
mod config;

use clap::Parser;
use cli::{Cli, SimulatorConfig};
use config::{CacheConfiguration, SimulationConfiguration};
use std::{error::Error, path::PathBuf};
use toml::toml;

/// However many pages of memory are supported by the simulator
const PAGES: usize = 16;

fn run_config(conf: PathBuf, bin: PathBuf) -> Result<(), Box<dyn Error>> {
    Ok(())
}

fn main() {
    let cli = Cli::parse();

    if let Some(info) = cli.info {
        if info.print_example_config {
            println!(
                "{}",
                toml! {
                    miss_penalty = 10
                    volatile_penalty = 2
                    writethrough = true

                    [cache.data]
                    mode = "disabled"

                    [cache.instruction]
                    mode = "associative"
                    set_bits = 2
                    offset_bits = 2
                    ways = 2
                }
            );
        }
    } else if let Some(SimulatorConfig {
        configuration,
        image_file,
    }) = cli.config
    {
        run_config(configuration, image_file).expect("Simulator ran into an error");
    }
}
