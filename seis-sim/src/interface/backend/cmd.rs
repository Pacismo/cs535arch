use clap::{Parser, ValueEnum};

#[derive(Debug, ValueEnum, Clone)]
pub enum Info {
    Pages,
    Cache,
    Pipeline,
    Configuration,
}

#[derive(Debug, Parser, Clone)]
#[clap(multicall = true, name = "", disable_help_flag = true)]
pub enum Command {
    Information { what: Info },
    ReadPage { page: usize },
    Clock { count: usize },
    Run { clock_rate: Option<u64> },
    Stop {},
    ShowRegs {},
    ShowCache {},
    ShowPipeline {},
    Statistics {},
    Terminate {},
}
