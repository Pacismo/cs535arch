use std::{fmt::Display, time::Duration};

#[derive(Debug, Default, Clone)]
pub struct RunResult<'a> {
    pub bench_name: &'a str,
    pub pipeline_enable: bool,
    pub cache_enable: bool,

    pub clocks: usize,
    pub rtc: Duration,
}

impl<'a> Display for RunResult<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\"{}\", {}, {}, {}, {}",
            self.bench_name,
            self.pipeline_enable,
            self.cache_enable,
            self.clocks,
            self.rtc.as_secs_f64()
        )
    }
}
