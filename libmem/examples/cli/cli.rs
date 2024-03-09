use clap::{Parser, Subcommand};

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
    None,
    Associative {
        #[arg(value_parser = ranged::<1, 16>)]
        ways: usize,
        #[arg(value_parser = ranged::<0, 32>)]
        set_bits: usize,
        #[arg(value_parser = ranged::<2, 32>)]
        off_bits: usize,
    },
}

#[derive(Parser)]
pub struct Args {
    #[arg(value_parser = ranged::<3, 65536>)]
    pub pages: usize,
    #[command(subcommand)]
    pub mode: CacheMode,
}
