use clap::{Parser, ValueHint::FilePath};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[clap(version, about, long_about = None)]
pub struct Command {
    /// The files to assemble.
    #[clap(num_args=1.., value_hint=FilePath)]
    pub files: Vec<PathBuf>,
    /// Where to store the output.
    ///
    /// a.out by default
    #[clap(short='o', value_hint=FilePath)]
    pub output: Option<PathBuf>,
}
