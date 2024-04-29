use clap::{Parser, ValueHint::FilePath};
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct Cli {
    /// Where to read information about the benchmarks
    #[arg(value_hint = FilePath)]
    pub bench_conf: PathBuf,

    /// Where to put the results of each benchmark
    ///
    /// By default, the results are put into the working directory
    /// in a file called "results.csv"
    #[arg(value_hint = FilePath)]
    pub output: Option<PathBuf>,
}

impl Cli {
    pub fn output_file(&self) -> PathBuf {
        self.output
            .clone()
            .unwrap_or_else(|| PathBuf::from("results.csv"))
    }
}
