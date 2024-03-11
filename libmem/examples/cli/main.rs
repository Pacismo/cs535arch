mod cli;
mod interactive;

use clap::Parser;
use cli::Args;
use inquire::Text;
use interactive::{Command, Sign, Type};
use libmem::{
    cache::*,
    memory::Memory,
    module::{MemoryModule, SingleLevel, Status},
};
use libseis::types::{Byte, SByte, SShort, SWord, Short, Word};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let cache = match args.mode {
        cli::CacheMode::None => NullCache.boxed(),
        cli::CacheMode::Associative { set_bits, off_bits } => {
            if set_bits + off_bits > 32 {
                return Err("set_bits + off_bits must sum up to 32".into());
            } else {
                Associative::new(off_bits, set_bits).boxed()
            }
        }
    };

    let memory = Memory::new(args.pages);

    // TODO: use the module to simulate memory
    let mut module =
        SingleLevel::new_with_boxed(cache, memory, args.miss_penalty, args.writethrough);

    loop {
        if let Some(input) = Text::new("")
            .with_placeholder("enter command")
            .with_autocomplete(Command::autocompleter())
            .with_help_message("Provide a command using the list above for guidance")
            .prompt_skippable()
            .expect("Could not read input")
        {
            match Command::try_parse_from(input.split_whitespace()) {
                Ok(Command::Exit) => break,
                Ok(Command::Read { sign, ty, address }) => match (sign, ty) {
                    (Sign::Unsigned, Type::Byte) => {
                        match module.read_byte(address) {
                            Ok(value) => {
                                println!(
                                    "[{address:#010X} ({sign} {ty})] {} (cache hit)",
                                    value as Byte
                                )
                            }
                            Err(Status::Busy(clocks)) => {
                                module.clock(clocks);
                                let value = module.read_byte(address).unwrap();
                                println!(
                                    "[{address:#010X} ({sign} {ty})] {} (took {clocks} {})",
                                    value as Byte,
                                    if clocks == 1 { "clock" } else { "clocks" }
                                )
                            }
                            _ => unreachable!(),
                        };
                    }
                    (Sign::Signed, Type::Byte) => {
                        match module.read_byte(address) {
                            Ok(value) => {
                                println!(
                                    "[{address:#010X} ({sign} {ty})] {} (cache hit)",
                                    value as SByte,
                                )
                            }
                            Err(Status::Busy(clocks)) => {
                                module.clock(clocks);
                                let value = module.read_byte(address).unwrap();
                                println!(
                                    "[{address:#010X} ({sign} {ty})] {} (took {clocks} {})",
                                    value as SByte,
                                    if clocks == 1 { "clock" } else { "clocks" }
                                )
                            }
                            _ => unreachable!(),
                        };
                    }
                    (Sign::Signed, Type::Short) => {
                        match module.read_short(address) {
                            Ok(value) => {
                                println!(
                                    "[{address:#010X} ({sign} {ty})] {} (cache hit)",
                                    value as Short,
                                )
                            }
                            Err(Status::Busy(clocks)) => {
                                module.clock(clocks);
                                let value = module.read_short(address).unwrap();
                                println!(
                                    "[{address:#010X} ({sign} {ty})] {} (took {clocks} {})",
                                    value as Short,
                                    if clocks == 1 { "clock" } else { "clocks" }
                                )
                            }
                            _ => unreachable!(),
                        };
                    }
                    (Sign::Signed, Type::Word) => {
                        match module.read_short(address) {
                            Ok(value) => {
                                println!(
                                    "[{address:#010X} ({sign} {ty})] {} (cache hit)",
                                    value as SShort,
                                )
                            }
                            Err(Status::Busy(clocks)) => {
                                module.clock(clocks);
                                let value = module.read_short(address).unwrap();
                                println!(
                                    "[{address:#010X} ({sign} {ty})] {} (took {clocks} {})",
                                    value as SShort,
                                    if clocks == 1 { "clock" } else { "clocks" }
                                )
                            }
                            _ => unreachable!(),
                        };
                    }
                    (Sign::Unsigned, Type::Short) => {
                        match module.read_word(address) {
                            Ok(value) => {
                                println!(
                                    "[{address:#010X} ({sign} {ty})] {} (cache hit)",
                                    value as Word,
                                )
                            }
                            Err(Status::Busy(clocks)) => {
                                module.clock(clocks);
                                let value = module.read_word(address).unwrap();
                                println!(
                                    "[{address:#010X} ({sign} {ty})] {} (took {clocks} {})",
                                    value as Word,
                                    if clocks == 1 { "clock" } else { "clocks" }
                                )
                            }
                            _ => unreachable!(),
                        };
                    }
                    (Sign::Unsigned, Type::Word) => {
                        match module.read_word(address) {
                            Ok(value) => {
                                println!(
                                    "[{address:#010X} ({sign} {ty})] {} (cache hit)",
                                    value as SWord,
                                )
                            }
                            Err(Status::Busy(clocks)) => {
                                module.clock(clocks);
                                let value = module.read_word(address).unwrap();
                                println!(
                                    "[{address:#010X} ({sign} {ty})] {} (took {clocks} {})",
                                    value as SWord,
                                    if clocks == 1 { "clock" } else { "clocks" }
                                )
                            }
                            _ => unreachable!(),
                        };
                    }
                },
                Ok(Command::Write { ty, address, value }) => match ty {
                    Type::Byte => match module.write_byte(address, value as Byte) {
                        Status::Busy(clocks) => {
                            module.clock(clocks);
                            println!(
                                "Write took {clocks} {}",
                                if clocks == 1 { "clock" } else { "clocks" }
                            )
                        }
                        Status::Idle => {
                            println!("Write hit the cache")
                        }
                    },
                    Type::Short => match module.write_short(address, value as Short) {
                        Status::Busy(clocks) => {
                            module.clock(clocks);
                            println!(
                                "Write took {clocks} {}",
                                if clocks == 1 { "clock" } else { "clocks" }
                            )
                        }
                        Status::Idle => {
                            println!("Write hit the cache")
                        }
                    },
                    Type::Word => match module.write_word(address, value as Word) {
                        Status::Busy(clocks) => {
                            module.clock(clocks);
                            println!(
                                "Write took {clocks} {}",
                                if clocks == 1 { "clock" } else { "clocks" }
                            )
                        }
                        Status::Idle => {
                            println!("Write hit the cache")
                        }
                    },
                },
                Err(e) => println!("{e}"),
            }
        }
    }

    Ok(())
}
