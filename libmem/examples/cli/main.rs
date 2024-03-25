mod cli;
mod interactive;

use clap::Parser;
use cli::Args;
use inquire::{
    ui::{Attributes, Color, StyleSheet},
    Text,
};
use interactive::{Command, Sign, Type};
use libmem::{
    cache::*,
    memory::Memory,
    module::{CacheData, MemoryModule, SingleLevel, Status},
};
use libseis::{
    pages::PAGE_SIZE,
    types::{Byte, SByte, SShort, SWord, Short, Word},
};
use serde::Serialize;
use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
};

fn process_input(
    command: Command,
    module: &mut dyn MemoryModule,
    total_clocks: &mut usize,
    manual: bool,
) -> Option<bool> {
    match command {
        Command::Exit => None,
        Command::Clock { amount } => {
            module.clock(amount);
            *total_clocks += amount;
            Some(false)
        }
        Command::FlushCache => {
            match module.flush_cache() {
                Status::Idle => println!("No cache lines were flushed"),
                Status::Busy(clocks) => {
                    module.clock(clocks);
                    *total_clocks += clocks;

                    println!(
                        "Flushed all dirty lines in the cache, taking {clocks} {}",
                        if clocks == 1 { "clock" } else { "clocks" }
                    )
                }
            }
            Some(true)
        }
        Command::Read { sign, ty, address } => {
            match (sign, ty) {
                (Sign::Unsigned, Type::Byte) => {
                    match module.read_byte(address) {
                        Ok(value) => {
                            println!(
                                "[{address:#010X} ({sign} {ty})] {} (cache hit)",
                                value as Byte
                            )
                        }
                        Err(Status::Busy(clocks)) if manual => {
                            println!(
                                "Memory subsystem requires {clocks} {} to complete an operation",
                                if clocks == 1 { "clock" } else { "clocks" }
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
                        Err(Status::Busy(clocks)) if manual => {
                            println!(
                                "Memory subsystem requires {clocks} {} to complete an operation",
                                if clocks == 1 { "clock" } else { "clocks" }
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
                        Err(Status::Busy(clocks)) if manual => {
                            println!(
                                "Memory subsystem requires {clocks} {} to complete an operation",
                                if clocks == 1 { "clock" } else { "clocks" }
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
                        Err(Status::Busy(clocks)) if manual => {
                            println!(
                                "Memory subsystem requires {clocks} {} to complete an operation",
                                if clocks == 1 { "clock" } else { "clocks" }
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
                        Err(Status::Busy(clocks)) if manual => {
                            println!(
                                "Memory subsystem requires {clocks} {} to complete an operation",
                                if clocks == 1 { "clock" } else { "clocks" }
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
                        Err(Status::Busy(clocks)) if manual => {
                            println!(
                                "Memory subsystem requires {clocks} {} to complete an operation",
                                if clocks == 1 { "clock" } else { "clocks" }
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
            }
            Some(true)
        }
        Command::Write { ty, address, value } => {
            match ty {
                Type::Byte => match module.write_byte(address, value as Byte) {
                    Status::Busy(clocks) if manual => {
                        println!(
                            "Memory subsystem requires {clocks} {} to complete an operation",
                            if clocks == 1 { "clock" } else { "clocks" }
                        )
                    }
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
                    Status::Busy(clocks) if manual => {
                        println!(
                            "Memory subsystem requires {clocks} {} to complete an operation",
                            if clocks == 1 { "clock" } else { "clocks" }
                        )
                    }
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
                    Status::Busy(clocks) if manual => {
                        println!(
                            "Memory subsystem requires {clocks} {} to complete an operation",
                            if clocks == 1 { "clock" } else { "clocks" }
                        )
                    }
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
            }
            Some(true)
        }
        Command::VolatileRead { sign, ty, address } => {
            match (sign, ty) {
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
            }
            Some(true)
        }
        Command::VolatileWrite { ty, address, value } => {
            match ty {
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
            }
            Some(true)
        }
        Command::ShowCache => {
            for CacheData { name, lines } in module.cache_state() {
                println!("{name}: ");

                for (i, line) in lines.iter().enumerate() {
                    match line {
                        Some(data) => {
                            println!(
                                "\t[{i:>4}] {:#010X}{}:",
                                data.base_address,
                                if data.dirty { " (dirty)" } else { "" }
                            );
                            for bytes in data.data.chunks(8) {
                                print!("\t\t");
                                for (i, byte) in bytes.iter().enumerate() {
                                    if i % 4 == 0 && i != 0 {
                                        print!(" {byte:02X} ");
                                    } else {
                                        print!("{byte:02X} ");
                                    }
                                }
                                println!();
                            }
                        }
                        None => println!("\t[{i:>4}] Invalid"),
                    }
                }
            }
            Some(false)
        }
        Command::Statistics => {
            let hits = module.cache_hits();
            let cold = module.cold_misses();
            let miss = module.total_misses();

            println!(
                "The cache was hit {hits} {} and missed {miss} {} (where {cold} {} cold)\nThe clock was ticked {total_clocks} {}",
                if hits == 1 { "time" } else { "times" },
                if miss == 1 { "time" } else { "times" },
                if cold == 1 { "was" } else { "were" },
                if *total_clocks == 1 { "time" } else { "times" },
            );
            Some(false)
        }
        Command::Debug { pretty } => {
            if pretty {
                println!("{module:#?}")
            } else {
                println!("{module:?}")
            }
            Some(false)
        }
        Command::JSON { pretty } => {
            if pretty {
                let mut jser = serde_json::Serializer::pretty(std::io::stdout());
                module.cache_state().serialize(&mut jser).unwrap();
                module.memory().serialize(&mut jser).unwrap();
            } else {
                let mut jser = serde_json::Serializer::new(std::io::stdout());
                module.cache_state().serialize(&mut jser).unwrap();
                module.memory().serialize(&mut jser).unwrap();
            }
            println!();

            Some(false)
        }
        Command::ShowMemory { page } => {
            for i in page..page + PAGE_SIZE as Word {
                if i % 256 == 0 && i != 0 {
                    match (inquire::CustomType {
                        message: "",
                        starting_input: None,
                        default: None,
                        placeholder: None,
                        help_message: Some("Press [ENTER] to continue or [ESC] to stop"),
                        formatter: &|_: ()| String::new(),
                        default_value_formatter: &|_| String::new(),
                        parser: &|_| Ok(()),
                        validators: vec![Box::new(|_: &()| {
                            Ok(inquire::validator::Validation::Valid)
                        })],
                        error_message: "".into(),
                        render_config: inquire::ui::RenderConfig {
                            prompt_prefix: "".into(),
                            answered_prompt_prefix: "".into(),
                            prompt: StyleSheet::new()
                                .with_attr(Attributes::ITALIC)
                                .with_fg(Color::Grey),
                            ..Default::default()
                        },
                    }
                    .prompt())
                    {
                        Ok(_) => (),
                        Err(_) => break,
                    }
                }

                if i % 256 == 0 {
                    print!("{i:#010X}:\n\t")
                }

                print!("{:>02X} ", module.memory().read_byte(i));
                if i % 4 == 3 {
                    print!(" ");
                }
                if i % 32 == 31 {
                    if i % 256 == 255 {
                        println!();
                    } else {
                        if i % 128 == 127 {
                            println!()
                        }
                        print!("\n\t");
                    }
                }
            }

            Some(false)
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = Args::parse();

    let mut module: Box<dyn MemoryModule> = match args.mode {
        cli::CacheMode::None => {
            args.writethrough = true;
            args.manual_clock = false;

            Box::new(SingleLevel::new(
                NullCache::new(),
                NullCache::new(),
                Memory::new(args.pages),
                args.miss_penalty,
                args.volatile_penalty,
                args.writethrough,
            ))
        }
        cli::CacheMode::Associative {
            set_bits,
            off_bits,
            ways,
        } => {
            if set_bits + off_bits > 32 {
                return Err("set_bits + off_bits must sum up to 32".into());
            } else if ways == 1 {
                Box::new(SingleLevel::new(
                    Associative::new(off_bits, set_bits),
                    Associative::new(off_bits, set_bits),
                    Memory::new(args.pages),
                    args.miss_penalty,
                    args.volatile_penalty,
                    args.writethrough,
                ))
            } else {
                Box::new(SingleLevel::new(
                    MultiAssociative::new(off_bits, set_bits, ways),
                    MultiAssociative::new(off_bits, set_bits, ways),
                    Memory::new(args.pages),
                    args.miss_penalty,
                    args.volatile_penalty,
                    args.writethrough,
                ))
            }
        }
    };

    let mut total_clocks = 0;

    if let Some(file) = args.cmd_file {
        let reader = BufReader::new(File::open(file)?);

        for (i, line) in reader
            .lines()
            .map(|l| l.expect("Failed to read line"))
            .enumerate()
        {
            print!("{:>3} > ", i + 1);
            if line.is_empty() {
                println!();
            } else {
                process_input(
                    Command::parse_from(line.split_whitespace()),
                    module.as_mut(),
                    &mut total_clocks,
                    false,
                );
                total_clocks += 1;
            }
        }
    } else {
        loop {
            if let Some(input) = Text::new("")
                .with_placeholder("enter command")
                .with_autocomplete(Command::autocompleter())
                .prompt_skippable()
                .expect("Could not read input")
            {
                match Command::try_parse_from(input.split_whitespace()) {
                    Ok(command) => match process_input(
                        command,
                        module.as_mut(),
                        &mut total_clocks,
                        args.manual_clock,
                    ) {
                        Some(true) => total_clocks += 1,
                        Some(false) => (),
                        None => break,
                    },
                    Err(e) => println!("{e}"),
                }
            }
        }
    }

    println!("Total clocks: {}", total_clocks);

    module
        .cache_state()
        .serialize(&mut serde_json::Serializer::pretty(File::create(
            "./cache.json",
        )?))?;
    module
        .memory()
        .serialize(&mut serde_json::Serializer::pretty(File::create(
            "./memory.json",
        )?))?;

    Ok(())
}
