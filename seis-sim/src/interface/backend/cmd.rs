use clap::{Parser, ValueEnum};

#[derive(Debug, ValueEnum, Clone)]
pub enum Info {
    #[value(alias = "page")]
    Pages,
    Cache,
    #[value(alias = "pipe")]
    Pipeline,
    #[value(alias = "config")]
    Configuration,
}

#[derive(Debug, Parser, Clone)]
#[clap(
    multicall = true,
    name = "",
    disable_help_flag = true,
    disable_help_subcommand = true
)]
pub enum Command {
    Decode {
        value: u32,
    },
    #[command(alias = "info")]
    Information {
        what: Info,
    },
    #[command(aliases = ["read", "page"])]
    ReadPage {
        page: usize,
    },
    Clock {
        count: usize,
    },
    Run {
        clock_rate: Option<u64>,
    },
    Stop {},
    #[command(alias = "regs")]
    ShowRegs {},
    #[command(alias = "cache")]
    ShowCache {},
    #[command(aliases = ["pipeline", "pipe"])]
    ShowPipeline {},
    #[command(aliases = ["stats", "stat"])]
    Statistics {},
    #[command(alias = "exit")]
    Terminate {},
}
