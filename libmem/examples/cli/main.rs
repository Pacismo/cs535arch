mod cli;
mod interactive;

use clap::Parser;
use cli::Args;
use inquire::Text;
use libmem::{cache::*, memory::Memory};
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
        let input = Text::new("MEM >")
            .with_autocomplete(interactive::CommandCompleter::default())
            .prompt()
            .expect("Could not read input");

        match interactive::Command::try_parse_from(input.split_whitespace()) {
            Ok(cmd) => match cmd {
                interactive::Command::Exit => break,
                interactive::Command::Read { address } => println!("address: {address}"),
                interactive::Command::Write { address, ty, value } => {
                    println!("{address} = {value}:{ty:?}")
                }
            },
            Err(e) if input.contains("help") => println!("{e}"),
            Err(e) => println!("Type \"help\" for help"),
        }
    }

    Ok(())
}
