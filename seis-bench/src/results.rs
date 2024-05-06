use std::{fmt::Display, time::Duration};

#[derive(Debug, Default, Clone)]
pub struct RunResult {
    pub bench_name: String,
    pub config_name: String,

    pub clocks: usize,
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
