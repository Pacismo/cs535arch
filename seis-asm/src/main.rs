mod cli;

use clap::Parser;
use libasm::linker::link_symbols;
use libasm::parse::{tokenize_file, Error, Lines};
use std::fs::File;

fn main() {
    let cli = cli::Command::parse();
    let output = cli.output.unwrap_or("./a.out".into());

    let lines = match cli
        .files
        .iter()
        .map(tokenize_file)
        .collect::<Result<Vec<Lines>, Error>>()
    {
        Ok(value) => value,
        Err(e) => panic!("{e}"),
    };

    let linked = link_symbols(lines.into()).expect("Failed to link code");

    linked
        .write(File::create(output).expect("Could not open output file"))
        .expect("Failed to write to file");
}
