use clap::Parser;
use std::error::Error;

mod cli;
mod parse;

fn main() -> Result<(), Box<dyn Error>> {
    let cli = cli::Command::parse();

    match parse::tokenize(&cli.files[0]).map_err(|e| e.to_string()) {
        Ok(_) => (),
        Err(e) => println!("{e}"),
    }

    Ok(())
}
