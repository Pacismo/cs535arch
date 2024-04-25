use clap::{Args, Parser, ValueHint::FilePath};
use std::path::PathBuf;

#[derive(Debug, Args)]
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
    /// An inline TOML configuration
    pub inline_config: Option<String>,

    /// A TOML file containing a configuration to use for the simulation.
    #[arg(conflicts_with = "inline_config", required_unless_present = "inline_config", value_hint = FilePath)]
    pub file_config: Option<PathBuf>,

    /// Enables backend mode
    #[arg(short, long)]
    pub backend_mode: bool,
}

/// Configure the simulation runtime.
///
/// The simulation runtime
#[derive(Debug, Parser)]
#[clap(author, version, about, long_about)]
pub enum Cli {
    /// Run the simulation with a provided configuration
    Run(#[command(flatten)] SimulatorConfig),

    /// Prints an example configuration file
    #[command(aliases = ["example-config", "e"])]
    PrintExampleConfiguration {
        /// Where to store the example configuration
        #[arg(value_hint = FilePath)]
        output_file: Option<PathBuf>,
    },
}
