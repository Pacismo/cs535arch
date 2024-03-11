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
use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
};

fn process_input(
    command: Command,
    module: &mut dyn MemoryModule,
    total_clocks: &mut usize,
) -> bool {
    match command {
        Command::Exit => return false,
        Command::Read { sign, ty, address } => match (sign, ty) {
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
                        *total_clocks += clocks;

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
                        *total_clocks += clocks;

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
                        *total_clocks += clocks;

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
                        *total_clocks += clocks;

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
                        *total_clocks += clocks;

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
                        *total_clocks += clocks;

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
        Command::Write { ty, address, value } => match ty {
            Type::Byte => match module.write_byte(address, value as Byte) {
                Status::Busy(clocks) => {
                    module.clock(clocks);
                    *total_clocks += clocks;

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
                    *total_clocks += clocks;

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
                    *total_clocks += clocks;

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
        Command::VolatileRead { sign, ty, address } => match (sign, ty) {
            (Sign::Unsigned, Type::Byte) => {
                match module.read_byte_volatile(address) {
                    Ok(_) => unreachable!(),
                    Err(Status::Busy(clocks)) => {
                        module.clock(clocks);
                        *total_clocks += clocks;

                        let value = module.read_byte_volatile(address).unwrap();
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
                match module.read_byte_volatile(address) {
                    Ok(_) => unreachable!(),
                    Err(Status::Busy(clocks)) => {
                        module.clock(clocks);
                        *total_clocks += clocks;

                        let value = module.read_byte_volatile(address).unwrap();
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
                match module.read_short_volatile(address) {
                    Ok(_) => unreachable!(),
                    Err(Status::Busy(clocks)) => {
                        module.clock(clocks);
                        *total_clocks += clocks;

                        let value = module.read_short_volatile(address).unwrap();
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
                match module.read_short_volatile(address) {
                    Ok(_) => unreachable!(),
                    Err(Status::Busy(clocks)) => {
                        module.clock(clocks);
                        *total_clocks += clocks;

                        let value = module.read_short_volatile(address).unwrap();
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
                match module.read_word_volatile(address) {
                    Ok(_) => unreachable!(),
                    Err(Status::Busy(clocks)) => {
                        module.clock(clocks);
                        *total_clocks += clocks;

                        let value = module.read_word_volatile(address).unwrap();
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
                match module.read_word_volatile(address) {
                    Ok(_) => unreachable!(),
                    Err(Status::Busy(clocks)) => {
                        module.clock(clocks);
                        *total_clocks += clocks;

                        let value = module.read_word_volatile(address).unwrap();
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
        Command::VolatileWrite { ty, address, value } => match ty {
            Type::Byte => match module.write_byte_volatile(address, value as Byte) {
                Status::Busy(clocks) => {
                    module.clock(clocks);
                    *total_clocks += clocks;

                    println!(
                        "Write took {clocks} {}",
                        if clocks == 1 { "clock" } else { "clocks" }
                    )
                }
                Status::Idle => unreachable!(),
            },
            Type::Short => match module.write_short_volatile(address, value as Short) {
                Status::Busy(clocks) => {
                    module.clock(clocks);
                    *total_clocks += clocks;

                    println!(
                        "Write took {clocks} {}",
                        if clocks == 1 { "clock" } else { "clocks" }
                    )
                }
                Status::Idle => unreachable!(),
            },
            Type::Word => match module.write_word_volatile(address, value as Word) {
                Status::Busy(clocks) => {
                    module.clock(clocks);
                    *total_clocks += clocks;

                    println!(
                        "Write took {clocks} {}",
                        if clocks == 1 { "clock" } else { "clocks" }
                    )
                }
                Status::Idle => unreachable!(),
            },
        },
    }
    true
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = Args::parse();

    let (dcache, icache) = match args.mode {
        cli::CacheMode::None => {
            args.writethrough = true;
            (NullCache::new().boxed(), NullCache::new().boxed())
        }
        cli::CacheMode::Associative { set_bits, off_bits } => {
            if set_bits + off_bits > 32 {
                return Err("set_bits + off_bits must sum up to 32".into());
            } else {
                (
                    Associative::new(off_bits, set_bits).boxed(),
                    Associative::new(off_bits, set_bits).boxed(),
                )
            }
        }
    };

    let memory = Memory::new(args.pages);

    // TODO: use the module to simulate memory
    let mut module = SingleLevel::new_with_boxed(
        dcache,
        icache,
        memory,
        args.miss_penalty,
        args.volatile_penalty,
        args.writethrough,
    );
    let mut total_clocks = 0;

    if let Some(file) = args.cmd_file {
        let reader = BufReader::new(File::open(file)?);

        for line in reader.lines() {
            process_input(
                Command::parse_from(line?.split_whitespace()),
                &mut module,
                &mut total_clocks,
            );
        }
    } else {
        loop {
            if let Some(input) = Text::new("")
                .with_placeholder("enter command")
                .with_autocomplete(Command::autocompleter())
                .with_help_message("Provide a command using the list above for guidance")
                .prompt_skippable()
                .expect("Could not read input")
            {
                match Command::try_parse_from(input.split_whitespace()) {
                    Ok(command) => {
                        if !process_input(command, &mut module, &mut total_clocks) {
                            break;
                        }
                    }
                    Err(e) => println!("{e}"),
                }
            }
        }
    }

    println!("Total clocks: {}", total_clocks);

    Ok(())
}
