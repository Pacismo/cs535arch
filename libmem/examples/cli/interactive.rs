use clap::{Parser, ValueEnum, ColorChoice::Always};
use inquire::Autocomplete;
use libseis::types::Word;
use std::fmt::Display;

#[derive(Debug, Default, Clone)]
pub struct CommandCompleter {}

impl Autocomplete for CommandCompleter {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, inquire::CustomUserError> {
        let lowercase = input.to_lowercase();
        let separated = lowercase.split_whitespace().collect::<Vec<&str>>();

        if separated.len() == 0 {
            Ok(Command::SUGGESTIONS
                .iter()
                .map(ToString::to_string)
                .collect())
        } else {
            Ok(Command::SUGGESTIONS
                .into_iter()
                .filter_map(|s| s.command.contains(separated[0]).then_some(s.to_string()))
                .collect())
        }
    }

    fn get_completion(
        &mut self,
        input: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<inquire::autocompletion::Replacement, inquire::CustomUserError> {
        let lowercase = input.to_lowercase();
        let separated = lowercase.split_whitespace().collect::<Vec<_>>();

        // TODO: handle the different commands

        if separated.len() == 0 {
            return Ok(None);
        }

        if let Some(s) = highlighted_suggestion {
            let mut rem = separated.into_iter();
            rem.next();
            let f = rem.next().map(|s| s.to_owned()).unwrap_or_default();

            Ok(Some(format!(
                "{} {}",
                s.split_whitespace().next().unwrap(),
                rem.fold(f, |a, e| format!("{a} {e}"))
            )))
        } else {
            Ok(None)
        }
    }
}

#[derive(ValueEnum, Clone, Copy, Debug)]
#[clap(disable_help_subcommand = true)]
pub enum ValTy {
    Byte,
    Short,
    Word,
}

#[derive(Parser, Debug)]
#[clap(multicall = true, color = Always)]
pub enum Command {
    Exit,
    Read {
        address: usize,
    },
    Write {
        #[clap(name = "TYPE")]
        ty: ValTy,
        address: usize,
        value: Word,
    },
}

impl Command {
    const SUGGESTIONS: [Suggestion; 4] = [
        Suggestion {
            command: "help",
            args: "",
        },
        Suggestion {
            command: "exit",
            args: "",
        },
        Suggestion {
            command: "read",
            args: "<ADDRESS>",
        },
        Suggestion {
            command: "write",
            args: "<TYPE> <ADDRESS> <VALUE>",
        },
    ];
}

struct Suggestion {
    command: &'static str,
    args: &'static str,
}

impl Display for Suggestion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.command, self.args)
    }
}
