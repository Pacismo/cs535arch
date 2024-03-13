mod autocomplete;

use self::autocomplete::{ArgumentField, StringCompleter};
use autocomplete::Node;
use clap::{Parser, ValueEnum};
use inquire::Autocomplete;
use libseis::types::{SWord, Word};
use std::{fmt::Display, mem::take, sync::Arc};

/// This represents the auto-completion for the interactive console as a forest of node trees.
///
/// Each node is capable of providing a suggestion and completing the suggestion.
/// Nodes have branches that are navigated to provide a complete suggestion or completion.
#[derive(Debug, Clone)]
pub struct CommandCompleter {
    nodes: Arc<[Arc<dyn Node>]>,
}

impl CommandCompleter {
    fn complete(&self, input: &[&str], display: bool) -> Vec<String> {
        self.nodes
            .iter()
            .flat_map(|n| n.complete(input, display))
            .collect()
    }
}

impl Autocomplete for CommandCompleter {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, inquire::CustomUserError> {
        let lowercase = input.to_lowercase();
        let mut separated = lowercase.split_whitespace().collect::<Vec<_>>();

        if separated.len() == 0 {
            Ok(self.complete(&[""], true))
        } else {
            if input.ends_with(' ') {
                separated.push("")
            }
            Ok(self.complete(&separated, true))
        }
    }

    fn get_completion(
        &mut self,
        input: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<inquire::autocompletion::Replacement, inquire::CustomUserError> {
        if let Some(s) = highlighted_suggestion {
            Ok(Some(
                s.split_whitespace()
                    .take_while(|s| !s.ends_with('>'))
                    .map(|s| s.to_owned() + " ")
                    .collect(),
            ))
        } else {
            let lowercase = input.to_lowercase();
            let separated = lowercase.split_whitespace().collect::<Vec<_>>();

            Ok(self.complete(&separated, false).first_mut().map(take))
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
#[clap(multicall = true, disable_help_flag = true, name = "")]
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
    /// Read values from memory, bypassing the cache
    VolatileRead {
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
    /// Write values to memory, bypassing the cache
    VolatileWrite {
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
    /// Flush the contents of cache into memory
    FlushCache,
    /// Show the contents of the cache
    ShowCache,
    /// Shows the statistics of the cache
    Statistics,
    /// Clock the memory subsystem
    Clock { amount: usize },
    /// Reads data from memory
    ShowMemory { page: Word },
    /// Stop the runtime
    Exit,
}

impl Command {
    /// Constructs a completer tree and returns a CommandCompleter
    pub fn autocompleter() -> CommandCompleter {
        static mut TREE: Option<Arc<[Arc<dyn Node>]>> = None;
        if let Some(root) = unsafe { &TREE } {
            CommandCompleter {
                nodes: root.clone(),
            }
        } else {
            let empty: Arc<[Arc<dyn Node>]> = [].into();

            let types = Arc::new([
                Arc::new(StringCompleter {
                    string: "byte",
                    subtree: empty.clone(),
                }),
                Arc::new(StringCompleter {
                    string: "short",
                    subtree: empty.clone(),
                }),
                Arc::new(StringCompleter {
                    string: "word",
                    subtree: empty.clone(),
                }),
            ]);

            let signs = Arc::new([
                Arc::new(StringCompleter {
                    string: "signed",
                    subtree: empty.clone(),
                }),
                Arc::new(StringCompleter {
                    string: "unsigned",
                    subtree: empty.clone(),
                }),
            ]);

            let root: Arc<[Arc<dyn Node>]> = Arc::new([
                Arc::new(StringCompleter {
                    string: "exit",
                    subtree: empty.clone(),
                }),
                Arc::new(StringCompleter {
                    string: "help",
                    subtree: Arc::new([Arc::new("read"), Arc::new("write"), Arc::new("exit")]),
                }),
                Arc::new(StringCompleter {
                    string: "read",
                    subtree: Arc::new([Arc::new(ArgumentField {
                        string: "<address>",
                        subtree: types
                            .iter()
                            .map(|s| -> Arc<dyn Node> {
                                Arc::new(StringCompleter {
                                    string: s.string,
                                    subtree: signs
                                        .iter()
                                        .map(|t| -> Arc<dyn Node> { t.clone() })
                                        .collect(),
                                })
                            })
                            .collect(),
                    })]),
                }),
                Arc::new(StringCompleter {
                    string: "write",
                    subtree: Arc::new([Arc::new(ArgumentField {
                        string: "<address>",
                        subtree: Arc::new([Arc::new(ArgumentField {
                            string: "<value>",
                            subtree: types
                                .iter()
                                .map(|t| -> Arc<dyn Node> { t.clone() })
                                .collect(),
                        })]),
                    })]),
                }),
                Arc::new(StringCompleter {
                    string: "volatile-read",
                    subtree: Arc::new([Arc::new(ArgumentField {
                        string: "<address>",
                        subtree: types
                            .iter()
                            .map(|s| -> Arc<dyn Node> {
                                Arc::new(StringCompleter {
                                    string: s.string,
                                    subtree: signs
                                        .iter()
                                        .map(|t| -> Arc<dyn Node> { t.clone() })
                                        .collect(),
                                })
                            })
                            .collect(),
                    })]),
                }),
                Arc::new(StringCompleter {
                    string: "volatile-write",
                    subtree: Arc::new([Arc::new(ArgumentField {
                        string: "<address>",
                        subtree: Arc::new([Arc::new(ArgumentField {
                            string: "<value>",
                            subtree: types
                                .iter()
                                .map(|t| -> Arc<dyn Node> { t.clone() })
                                .collect(),
                        })]),
                    })]),
                }),
                Arc::new("flush-cache"),
                Arc::new("show-cache"),
                Arc::new("statistics"),
                Arc::new(StringCompleter {
                    string: "show-memory",
                    subtree: Arc::new([Arc::new(ArgumentField {
                        string: "<page-id>",
                        subtree: empty.clone(),
                    })]),
                }),
                Arc::new(StringCompleter {
                    string: "clock",
                    subtree: Arc::new([Arc::new(ArgumentField {
                        string: "<count>",
                        subtree: empty,
                    })]),
                }),
            ]);

            unsafe {
                TREE = Some(root.clone());
            }

            CommandCompleter { nodes: root }
        }
    }
}
