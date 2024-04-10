use clap::{Parser, ValueEnum};
use std::fmt::Display;

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

#[derive(Debug, ValueEnum, Clone, Copy)]
pub enum Register {
    V0,
    V1,
    V2,
    V3,
    V4,
    V5,
    V6,
    V7,
    V8,
    V9,
    VA,
    VB,
    VC,
    VD,
    VE,
    VF,
    SP,
    BP,
    LP,
    PC,
    ZF,
    OF,
    EPS,
    NAN,
    INF,
}

impl Into<libseis::types::Register> for Register {
    fn into(self) -> libseis::types::Register {
        match self {
            Register::V0 => libseis::registers::V[0x0],
            Register::V1 => libseis::registers::V[0x1],
            Register::V2 => libseis::registers::V[0x2],
            Register::V3 => libseis::registers::V[0x3],
            Register::V4 => libseis::registers::V[0x4],
            Register::V5 => libseis::registers::V[0x5],
            Register::V6 => libseis::registers::V[0x6],
            Register::V7 => libseis::registers::V[0x7],
            Register::V8 => libseis::registers::V[0x8],
            Register::V9 => libseis::registers::V[0x9],
            Register::VA => libseis::registers::V[0xA],
            Register::VB => libseis::registers::V[0xB],
            Register::VC => libseis::registers::V[0xC],
            Register::VD => libseis::registers::V[0xD],
            Register::VE => libseis::registers::V[0xE],
            Register::VF => libseis::registers::V[0xF],
            Register::SP => libseis::registers::SP,
            Register::BP => libseis::registers::BP,
            Register::LP => libseis::registers::LP,
            Register::PC => libseis::registers::PC,
            Register::ZF => libseis::registers::ZF,
            Register::OF => libseis::registers::OF,
            Register::EPS => libseis::registers::EPS,
            Register::NAN => libseis::registers::NAN,
            Register::INF => libseis::registers::INF,
        }
    }
}

impl Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Register::V0 => write!(f, "v0"),
            Register::V1 => write!(f, "v1"),
            Register::V2 => write!(f, "v2"),
            Register::V3 => write!(f, "v3"),
            Register::V4 => write!(f, "v4"),
            Register::V5 => write!(f, "v5"),
            Register::V6 => write!(f, "v6"),
            Register::V7 => write!(f, "v7"),
            Register::V8 => write!(f, "v8"),
            Register::V9 => write!(f, "v9"),
            Register::VA => write!(f, "va"),
            Register::VB => write!(f, "vb"),
            Register::VC => write!(f, "vc"),
            Register::VD => write!(f, "vd"),
            Register::VE => write!(f, "ve"),
            Register::VF => write!(f, "vf"),
            Register::SP => write!(f, "sp"),
            Register::BP => write!(f, "bp"),
            Register::LP => write!(f, "lp"),
            Register::PC => write!(f, "pc"),
            Register::ZF => write!(f, "zf"),
            Register::OF => write!(f, "of"),
            Register::EPS => write!(f, "eps"),
            Register::NAN => write!(f, "nan"),
            Register::INF => write!(f, "inf"),
        }
    }
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
    #[command(aliases = ["disasm"])]
    DisassemblePage {
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
    ShowRegs {
        #[arg(trailing_var_arg = true)]
        regs: Option<Vec<Register>>,
    },
    #[command(alias = "cache")]
    ShowCache {},
    #[command(aliases = ["pipeline", "pipe"])]
    ShowPipeline {},
    #[command(aliases = ["stats", "stat"])]
    Statistics {},
    #[command(alias = "exit")]
    Terminate {},
}
