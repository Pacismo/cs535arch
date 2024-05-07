//! Representation of the results of each run
use std::{fmt::Display, time::Duration};

/// The result of a run consisting of a benchmark and a configuration
#[derive(Debug, Default, Clone)]
pub struct RunResult {
    /// The name of the benchmark that was run
    pub bench_name: String,
    /// The name of the configuration that was used
    pub config_name: String,

    /// The number of clocks elapsed when running the benchmark and configuration
    pub clocks: usize,
    /// The amount of time elapsed while running the benchmark, in real time
    pub rtc: Duration,
}

impl Display for RunResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{},{},{},{}",
            self.bench_name,
            self.config_name,
            self.clocks,
            self.rtc.as_secs_f64()
        )
    }
}
