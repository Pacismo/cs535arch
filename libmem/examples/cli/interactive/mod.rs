mod autocomplete;

use self::autocomplete::{ArgumentField, StringCompleter};
use autocomplete::Node;
use clap::{Parser, ValueEnum};
use inquire::Autocomplete;
use libseis::types::{SWord, Word};
use std::{fmt::Display, rc::Rc};

/// This represents the auto-completion for the interactive console as a forest of node trees.
///
/// Each node is capable of providing a suggestion and completing the suggestion.
/// Nodes have branches that are navigated to provide a complete suggestion or completion.
#[derive(Debug, Clone)]
pub struct CommandCompleter {
    root: Vec<Rc<dyn Node>>,
}

fn recurse_suggestions<'a>(
    path: &[&str],
    node: &'a [Rc<dyn Node>],
    next_level: bool,
) -> Vec<&'a dyn Node> {
    if node.len() == 0 {
        vec![]
    } else if path.is_empty() {
        node.iter().map(AsRef::as_ref).collect()
    } else if path.len() == 1 {
        if next_level {
            node.iter()
                .find_map(|r| {
                    r.exact(path[0])
                        .then(|| r.subtree().iter().map(AsRef::as_ref).collect())
                })
                .unwrap_or_default()
        } else {
            node.iter()
                .filter_map(|r| r.matches(path[0]).then(|| r.as_ref()))
                .collect()
        }
    } else {
        node.iter()
            .filter_map(|r| {
                r.exact(path[0])
                    .then(|| recurse_suggestions(&path[1..], r.subtree(), next_level))
            })
            .reduce(|mut a, e| {
                a.extend_from_slice(&e);
                a
            })
            .unwrap_or_default()
    }
}

fn recurse_navigate<'a>(path: &[&str], node: &'a dyn Node) -> Option<&'a dyn Node> {
    if path.len() == 1 {
        node.matches(path[0]).then_some(node)
    } else if node.exact(path[0]) && path.len() > 1 {
        node.subtree()
            .iter()
            .find_map(|r| recurse_navigate(&path[1..], r.as_ref()))
    } else {
        None
    }
}

impl Autocomplete for CommandCompleter {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, inquire::CustomUserError> {
        let lowercase = input.to_lowercase();
        let separated = lowercase.split_whitespace().collect::<Vec<&str>>();

        if separated.len() == 0 {
            Ok(self.root.iter().map(|r| format!("{r}")).collect())
        } else {
            let next_level = input.ends_with(' ');
            let all_but_last = separated
                .iter()
                .take(separated.len() - (!next_level) as usize)
                .map(|s| s.to_string())
                .reduce(|l, r| format!("{l} {r}"))
                .unwrap_or_default();

            Ok(recurse_suggestions(&separated, &self.root, next_level)
                .into_iter()
                .map(|s| {
                    if !all_but_last.is_empty() {
                        s.complete(input)
                    } else {
                        s.to_string()
                    }
                })
                .collect())
        }
    }

    fn get_completion(
        &mut self,
        input: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<inquire::autocompletion::Replacement, inquire::CustomUserError> {
        let lowercase = input.to_lowercase();
        let mut separated = lowercase.split_whitespace().collect::<Vec<_>>();

        if separated.len() == 0 {
            return Ok(None);
        }

        if let Some(s) = highlighted_suggestion {
            if input.ends_with(' ') {
                separated.push(s.as_str());
            }

            if let Some(suggestion) = self
                .root
                .iter()
                .find_map(|r| recurse_navigate(&separated, r.as_ref()))
            {
                if separated.len() == 1 {
                    Ok(Some(suggestion.complete(separated.first().unwrap())))
                } else {
                    Ok(Some(format!(
                        "{} {}",
                        separated
                            .iter()
                            .take(separated.len() - 1)
                            .fold(String::new(), |a, e| if a.is_empty() {
                                e.to_string()
                            } else {
                                format!("{a} {e}")
                            }),
                        suggestion.complete(separated.last().unwrap())
                    )))
                }
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum Type {
    /// Specify the type as an 8-bit integer
    Byte,
    /// Specify the type as a 16-bit integer
    Short,
    /// Specify the type as a 32-bit integer
    Word,
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Type::*;

        match self {
            Byte => write!(f, "byte"),
            Short => write!(f, "short"),
            Word => write!(f, "word"),
        }
    }
}

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum Sign {
    /// Specify that the type is signed
    Signed,
    /// Specify that the type is unsigned
    Unsigned,
}

impl Display for Sign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Sign::*;

        match self {
            Signed => write!(f, "signed"),
            Unsigned => write!(f, "unsigned"),
        }
    }
}

fn address_parser(value: &str) -> Result<Word, String> {
    if value == "0" {
        Ok(0)
    } else if value.starts_with("0x") {
        Word::from_str_radix(&value[2..], 16).map_err(|e| e.to_string())
    } else if value.starts_with("0") {
        Word::from_str_radix(&value[1..], 8).map_err(|e| e.to_string())
    } else {
        value.parse::<Word>().map_err(|e| e.to_string())
    }
}

fn memval_parser(value: &str) -> Result<Word, String> {
    if value.starts_with('-') {
        Ok(value.parse::<SWord>().map_err(|e| e.to_string())? as Word)
    } else {
        value.parse::<Word>().map_err(|e| e.to_string())
    }
}

#[derive(Parser, Debug)]
#[clap(multicall = true, disable_help_flag = true)]
pub enum Command {
    /// Read values from memory
    Read {
        /// The address to read from
        #[clap(value_parser = address_parser)]
        address: Word,

        /// Specify the width of the type
        #[clap(name = "TYPE", default_value_t = Type::Byte)]
        ty: Type,
        /// Specify whether the type is signed or unsigned
        #[clap(default_value_t = Sign::Unsigned)]
        sign: Sign,
    },
    /// Write values to memory
    Write {
        /// The address to write to
        #[clap(value_parser = address_parser)]
        address: Word,

        /// The value to write to the address
        #[clap(value_parser = memval_parser)]
        #[arg(allow_hyphen_values = true)]
        value: Word,

        /// Specify the width of the type
        #[clap(name = "TYPE", default_value_t = Type::Byte)]
        ty: Type,
    },
    /// Stop the runtime
    Exit,
}

impl Command {
    pub fn autocompleter() -> CommandCompleter {
        let types = vec![
            Rc::new(StringCompleter {
                string: "byte".to_owned(),
                subtree: vec![],
            }),
            Rc::new(StringCompleter {
                string: "short".to_owned(),
                subtree: vec![],
            }),
            Rc::new(StringCompleter {
                string: "word".to_owned(),
                subtree: vec![],
            }),
        ];
        let signs = vec![
            Rc::new(StringCompleter {
                string: "signed".to_owned(),
                subtree: vec![],
            }),
            Rc::new(StringCompleter {
                string: "unsigned".to_owned(),
                subtree: vec![],
            }),
        ];
        CommandCompleter {
            root: vec![
                Rc::new(StringCompleter {
                    string: "exit".to_owned(),
                    subtree: vec![],
                }),
                Rc::new(StringCompleter {
                    string: "read".to_owned(),
                    subtree: vec![Rc::new(ArgumentField {
                        string: "<ADDRESS>".to_owned(),
                        subtree: types
                            .iter()
                            .map(|s| -> Rc<dyn Node> {
                                Rc::new(StringCompleter {
                                    string: s.string.clone(),
                                    subtree: signs
                                        .iter()
                                        .map(|t| -> Rc<dyn Node> { t.clone() })
                                        .collect(),
                                })
                            })
                            .collect(),
                    })],
                }),
                Rc::new(StringCompleter {
                    string: "write".to_owned(),
                    subtree: vec![Rc::new(ArgumentField {
                        string: "<ADDRESS>".to_owned(),
                        subtree: vec![Rc::new(ArgumentField {
                            string: "<VALUE>".to_owned(),
                            subtree: types
                                .iter()
                                .map(|t| -> Rc<dyn Node> { t.clone() })
                                .collect(),
                        })],
                    })],
                }),
                Rc::new(StringCompleter {
                    string: "help".to_owned(),
                    subtree: vec![Rc::new("read"), Rc::new("write"), Rc::new("exit")],
                }),
            ],
        }
    }
}
