mod cli;
mod linker;
mod parse;
use clap::Parser;
use parse::tokenize;

use crate::linker::link_symbols;

fn main() {
    let cli = cli::Command::parse();

    let tokens = match tokenize(&cli.files[0]) {
        Ok(value) => value,
        Err(e) => panic!("{e}"),
    };

    // TOKENS!
    println!("{tokens:#?}");

    let linked = link_symbols(tokens);
}
