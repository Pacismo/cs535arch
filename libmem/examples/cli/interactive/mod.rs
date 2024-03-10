mod autocomplete;

use autocomplete::Node;
use clap::{Parser, ValueEnum};
use inquire::Autocomplete;
use libseis::types::{SWord, Word};
use std::fmt::Display;

use self::autocomplete::{ArgumentField, StringCompleter};

#[derive(Debug, Clone)]
pub struct CommandCompleter {
    root: Vec<Box<dyn Node>>,
}

fn recurse_suggestions(path: &[&str], node: &[Box<dyn Node>], next_level: bool) -> Vec<String> {
    if node.len() == 0 {
        vec![]
    } else if path.is_empty() {
        node.iter().map(ToString::to_string).collect()
    } else if path.len() == 1 {
        if next_level {
            node.iter()
                .find_map(|r| {
                    r.exact(path[0])
                        .then(|| r.subtree().iter().map(ToString::to_string).collect())
                })
                .unwrap_or_default()
        } else {
            node.iter()
                .filter_map(|r| r.matches(path[0]).then(|| r.to_string()))
                .collect()
        }
    } else {
        node.iter()
            .filter_map(|r| {
                r.matches(path[0])
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
            Ok(recurse_suggestions(
                &separated,
                &self.root,
                input.ends_with(' '),
            ))
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
#[clap(multicall = true)]
pub enum Command {
    Read {
        #[clap(value_parser = address_parser)]
        address: Word,

        #[clap(default_value_t = Sign::Unsigned)]
        sign: Sign,
        #[clap(name = "TYPE", default_value_t = Type::Byte)]
        ty: Type,
    },
    Write {
        #[clap(value_parser = address_parser)]
        address: Word,
        #[clap(value_parser = memval_parser)]
        #[arg(allow_hyphen_values = true)]
        value: Word,

        #[clap(name = "TYPE", default_value_t = Type::Byte)]
        ty: Type,
    },
    Exit,
}

impl Command {
    pub fn autocompleter() -> CommandCompleter {
        let types = vec![
            StringCompleter {
                string: "byte".to_owned(),
                subtree: vec![],
            },
            StringCompleter {
                string: "short".to_owned(),
                subtree: vec![],
            },
            StringCompleter {
                string: "word".to_owned(),
                subtree: vec![],
            },
        ];
        let signed = vec![
            StringCompleter {
                string: "signed".to_owned(),
                subtree: vec![],
            },
            StringCompleter {
                string: "unsigned".to_owned(),
                subtree: vec![],
            },
        ];
        CommandCompleter {
            root: vec![
                Box::new(StringCompleter {
                    string: "exit".to_owned(),
                    subtree: vec![],
                }),
                Box::new(StringCompleter {
                    string: "read".to_owned(),
                    subtree: vec![Box::new(ArgumentField {
                        string: "<ADDRESS>".to_owned(),
                        subtree: signed
                            .iter()
                            .map(|s| -> Box<dyn Node> {
                                Box::new(StringCompleter {
                                    string: s.string.clone(),
                                    subtree: types.iter().map(|t| t.box_clone()).collect(),
                                })
                            })
                            .collect(),
                    })],
                }),
                Box::new(StringCompleter {
                    string: "write".to_owned(),
                    subtree: vec![Box::new(ArgumentField {
                        string: "<ADDRESS>".to_owned(),
                        subtree: vec![Box::new(ArgumentField {
                            string: "<VALUE>".to_owned(),
                            subtree: types
                                .iter()
                                .map(|s| -> Box<dyn Node> {
                                    Box::new(StringCompleter {
                                        string: s.string.clone(),
                                        subtree: vec![],
                                    })
                                })
                                .collect(),
                        })],
                    })],
                }),
                Box::new(StringCompleter {
                    string: "help".to_owned(),
                    subtree: vec![Box::new("read"), Box::new("write"), Box::new("exit")],
                }),
            ],
        }
    }
}
