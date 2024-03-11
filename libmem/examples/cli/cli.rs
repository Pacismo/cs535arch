use std::path::PathBuf;
use clap::{Parser, Subcommand, ValueHint::FilePath};

fn ranged<const MIN: usize, const MAX: usize>(val: &str) -> Result<usize, String> {
    let val = val.parse::<usize>().map_err(|e| e.to_string())?;

    if val >= MIN && val <= MAX {
        Ok(val)
    } else {
        Err(format!("Value must be between {MIN} and {MAX}"))
    }
}

#[derive(Subcommand, Clone)]
pub enum CacheMode {
    /// Use no cache
    None,
    /// Use an associative cache
    Associative {
        /// How many bits to reserve for the set
        #[arg(value_parser = ranged::<0, 32>)]
        set_bits: usize,
        /// How many bits to reserve for the byte offset
        #[arg(value_parser = ranged::<2, 32>)]
        off_bits: usize,
    },
}

#[derive(Parser)]
#[clap(author, version, about)]
pub struct Args {
    /// How many pages to allocate
    #[arg(value_parser = ranged::<3, 65536>)]
    pub pages: usize,
    /// What kind of cache to use
    #[command(subcommand)]
    pub mode: CacheMode,

    /// The penalty for a miss, in clocks
    #[arg(default_value_t = 100, value_parser = ranged::<1, { usize::MAX }>)]
    pub miss_penalty: usize,
    /// The penalty for a volatile access, in clocks
    #[arg(default_value_t = 20, value_parser = ranged::<1, { usize::MAX }>)]
    pub volatile_penalty: usize,
    /// Whether to have writes go through the cache on miss
    #[arg(short, long)]
    pub writethrough: bool,

    /// The file to read for memory instructions
    #[arg(short, long)]
    #[arg(value_hint = FilePath)]
    pub cmd_file: Option<PathBuf>
}
