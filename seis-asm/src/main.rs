mod cli;
mod linker;
mod parse;

use clap::Parser;
use parse::{tokenize, Error, Lines};

use crate::linker::link_symbols;

fn main() {
    let cli = cli::Command::parse();

    let tokens = match cli
        .files
        .iter()
        .map(tokenize)
        .collect::<Result<Vec<Lines>, Error>>()
    {
        Ok(value) => value
            .into_iter()
            .reduce(|mut l, r| {
                l.extend(r.into_iter());
                l
            })
            .unwrap(),
        Err(e) => panic!("{e}"),
    };

    // TOKENS!
    println!("{tokens:#?}");

    let linked = link_symbols(tokens);
}
