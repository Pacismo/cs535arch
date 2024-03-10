mod cli;
mod interactive;

use clap::Parser;
use cli::Args;
use inquire::Text;
use interactive::{Command, Type};
use libmem::{cache::*, memory::Memory};
use libseis::types::{Byte, Short};
use std::{error::Error, slice::from_ref};

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let mut cache = match args.mode {
        cli::CacheMode::None => NullCache.boxed(),
        cli::CacheMode::Associative {
            ways,
            set_bits,
            off_bits,
        } => {
            if set_bits + off_bits > 32 {
                return Err("set_bits + off_bits must sum up to 32".into());
            } else if ways == 1 {
                Associative::new(off_bits, set_bits).boxed()
            } else {
                unimplemented!()
            }
        }
    };

    let mut memory = Memory::new(args.pages);

    loop {
        if let Some(input) = Text::new("MEM >")
            .with_autocomplete(Command::autocompleter())
            .prompt_skippable()
            .expect("Could not read input")
        {
            match Command::try_parse_from(input.split_whitespace()) {
                Ok(Command::Exit) => break,
                Ok(Command::Read { sign, ty, address }) => {
                    println!("{address:#010X} ({sign} {ty})")
                }
                Ok(Command::Write { ty, address, value }) => match ty {
                    Type::Word => println!("{address:#010X} = {value}"),
                    Type::Short => println!("{address:#010X} = {}", value as Short),
                    Type::Byte => println!("{address:#010X} = {}", value as Byte),
                },
                Err(e) => println!("{e}"),
            }
        }
    }

    Ok(())
}
