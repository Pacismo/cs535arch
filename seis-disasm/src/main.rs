use clap::{Parser, ValueHint::FilePath};
use libseis::{
    instruction_set::{decode, Instruction},
    types::Word,
};
use std::{fs::read, io::{stdin, stdout, Write}, path::PathBuf};

#[derive(Parser, Debug, Clone)]
pub struct Cli {
    #[arg(value_hint = FilePath)]
    pub file: PathBuf,

    #[arg(short)]
    pub binary: bool,
}

fn main() {
    let Cli { file, binary } = Cli::parse();

    let content = read(file).expect("Failed to read file");

    let header = if binary {
        format!(
            "{:<10} | {:>8} | {:>8} | {:>8} | {:>8} | {}\n{}",
            "Address",
            "+0",
            "+1",
            "+2",
            "+3",
            "Instruction",
            "-----------|----------|----------|----------|----------|------------------"
        )
    } else {
        format!(
            "{:<10} | {:>2} | {:>2} | {:>2} | {:>2} | {}\n{}",
            "Address",
            "+0",
            "+1",
            "+2",
            "+3",
            "Instruction",
            "-----------|----|----|----|----|------------------"
        )
    };

    for (i, word) in content.chunks(4).enumerate() {
        if i * 4 % 0x28 == 0 {
            if i != 0 {
                print!("[PRESS ENTER]");
                stdout().flush().expect("Failed to flush to stdout");
                stdin()
                    .read_line(&mut String::new())
                    .expect("Failed to read a line");
                println!();
            }
            println!("{header}");
        }

        if word.len() == 4 {
            let instruction = match decode::<Instruction>(Word::from_be_bytes([
                word[0], word[1], word[2], word[3],
            ])) {
                Ok(instruction) => instruction.to_string(),
                Err(_) => "unknown".to_string(),
            };

            if binary {
                println!(
                    "{:#010X} | {:08b} | {:08b} | {:08b} | {:08b} | {instruction}",
                    i * 4,
                    word[0],
                    word[1],
                    word[2],
                    word[3]
                );
            } else {
                println!(
                    "{:#010X} | {:02X} | {:02X} | {:02X} | {:02X} | {instruction}",
                    i * 4,
                    word[0],
                    word[1],
                    word[2],
                    word[3]
                );
            }
        } else {
            print!("{i:#010X}");
            for byte in word {
                if binary {
                    print!(" | {byte:08b}");
                } else {
                    print!(" | {byte:02X}");
                }
            }
            for _ in word.len()..4 {
                if binary {
                    print!(" |         ");
                } else {
                    print!(" |   ");
                }
            }
            println!();
        }
    }
}
