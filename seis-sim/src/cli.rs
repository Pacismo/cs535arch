use clap::{Args, Parser, ValueHint::FilePath};
use std::path::PathBuf;

#[derive(Debug, Args)]
#[group(conflicts_with = "SimulatorInfo")]
pub struct SimulatorConfig {
    /// The binary image file to be used for the simulation
    #[arg(value_hint = FilePath)]
    pub image_file: PathBuf,

    /// A string containing a TOML configuration
    #[arg(
        short = 'i',
        long = "inline",
        required_unless_present = "file_config",
        conflicts_with = "file_config"
    )]
    pub inline_config: Option<String>,

    /// A TOML file containing a configuration to use for the simulation.
    #[arg(conflicts_with = "inline_config", required_unless_present = "inline_config", value_hint = FilePath)]
    pub file_config: Option<PathBuf>,

    /// Enables backend mode
    #[arg(short, long)]
    pub backend_mode: bool,
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
