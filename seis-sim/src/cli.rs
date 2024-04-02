use clap::{Args, Parser, ValueHint::FilePath};
use std::path::PathBuf;

#[derive(Debug, Args)]
#[group(conflicts_with = "SimulatorInfo")]
pub struct SimulatorConfig {
    /// A TOML file containing a configuration to use for the simulation.
    #[arg(value_hint = FilePath)]
    pub configuration: PathBuf,

    /// The binary image file to be used for the simulation
    #[arg(value_hint = FilePath)]
    pub image_file: PathBuf,
}

#[derive(Debug, Args)]
#[group(conflicts_with = "SimulatorConfig", multiple = false)]
pub struct SimulatorInfo {
    /// Prints or outputs an example configuration file
    #[arg(short = 'e', long, value_hint = FilePath)]
    pub print_example_config: Option<Option<PathBuf>>,
}

/// Configure the simulation runtime.
///
/// The simulation runtime
#[derive(Debug, Parser)]
#[clap(author, version, about, long_about)]
pub struct Cli {
    #[command(flatten)]
    pub info: Option<SimulatorInfo>,

    #[command(flatten)]
    pub config: Option<SimulatorConfig>,
}
