mod cli;
mod config;

use clap::Parser;
use cli::Cli;
use config::{CacheConfiguration, SimulationConfiguration};
use toml::toml;

/// However many pages of memory are supported by the simulator
const PAGES: usize = 16;

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
    } else if let Some(config) = cli.config {
        todo!()
    } else {
        unreachable!()
    }
}
