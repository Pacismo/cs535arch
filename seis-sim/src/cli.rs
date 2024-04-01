use clap::{Parser, ValueHint::FilePath};
use std::path::PathBuf;

/// Configure the simulation runtime.
#[derive(Parser)]
#[clap(author, version, about, long_about)]
pub struct Cli {
    #[arg(value_hint = FilePath)]
    pub configuration: PathBuf,

    #[arg(value_hint = FilePath)]
    pub image_file: PathBuf,
}
