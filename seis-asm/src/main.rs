mod cli;
mod linker;
mod parse;
#[cfg(test)]
mod test;

use clap::Parser;
use linker::link_symbols;
use parse::{tokenize, Error, Lines};
use std::fs::File;

fn main() {
    let cli = cli::Command::parse();
    let output = cli.output.unwrap_or("./a.out".into());

    let lines = match cli
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

    let linked = link_symbols(lines).expect("Failed to link code");

    linked
        .write(File::create(output).expect("Could not open output file"))
        .expect("Failed to write to file");
}
