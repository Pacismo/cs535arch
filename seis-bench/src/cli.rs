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

    /// How many threads to spawn at most.
    ///
    /// Not specifying this option will spawn the default number of threads (4)
    #[arg(short = 'n', long = "threads")]
    pub threads: Option<usize>,
}

impl Cli {
    pub fn output_file(&self) -> PathBuf {
        self.output
            .clone()
            .unwrap_or_else(|| PathBuf::from("results.csv"))
    }
}
