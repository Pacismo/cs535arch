mod cli;
mod config;

use config::{CacheConfiguration, SimulationConfiguration};
use toml::toml;

/// 64 pages of memory are supported by the simulator
const PAGES: usize = 64;

fn main() {
    let config = SimulationConfiguration {
        cache: [
            ("data".to_string(), CacheConfiguration::Disabled),
            (
                "instruction".to_string(),
                CacheConfiguration::Associative {
                    set_bits: 2,
                    offset_bits: 2,
                    ways: 2,
                },
            ),
        ]
        .into(),

        miss_penalty: 10,
        volatile_penalty: 2,
        writethrough: true,
    };

    let toml = config.to_toml();

    println!("{toml}");

    let table = toml! {
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
    };

    let config = SimulationConfiguration::from_toml(&table).unwrap();

    println!("{config:#?}")
}
