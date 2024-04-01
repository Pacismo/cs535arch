use clap::{Parser, ValueHint::FilePath};
use std::path::PathBuf;

/// Configure the simulation runtime.
#[derive(Parser)]
#[clap(author, version, about, long_about)]
pub struct Cli {
    /// A TOML file containing a configuration to use for the simulation.
    #[arg(value_hint = FilePath)]
    pub configuration: PathBuf,

    /// The binary image file to be used for the simulation
    #[arg(value_hint = FilePath)]
    pub image_file: PathBuf,
}
