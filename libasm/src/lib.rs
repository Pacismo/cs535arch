pub mod linker;
pub mod parse;
#[cfg(test)]
mod test;

use linker::link_symbols;
use parse::{tokenize, Lines};
use std::{fmt::Display, io::Cursor, path::Path};

#[derive(Debug)]
pub enum Error {
    Linker(linker::error::Error),
    Parser(parse::Error),
    Io(std::io::Error),
}

impl From<linker::error::Error> for Error {
    fn from(value: linker::error::Error) -> Self {
        Self::Linker(value)
    }
}

impl From<parse::Error> for Error {
    fn from(value: parse::Error) -> Self {
        Self::Parser(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Linker(e) => e.fmt(f),
            Error::Parser(e) => e.fmt(f),
            Error::Io(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Linker(ref e) => Some(e),
            Error::Parser(ref e) => Some(e),
            Error::Io(ref e) => Some(e),
        }
    }
}

pub struct Input<'a> {
    pub data: &'a str,
    pub path: &'a str,
}

pub fn compile<'a, I>(inputs: I) -> Result<Vec<u8>, Error>
where
    I: IntoIterator<Item = Input<'a>>,
{
    let tokens = inputs
        .into_iter()
        .map(|i| tokenize(i.data, Path::new(i.path)))
        .collect::<Result<Vec<Lines>, parse::Error>>()?;

    let linked = link_symbols(tokens.into())?;

    let mut result = vec![];

    linked.write(Cursor::new(&mut result))?;

    Ok(result)
}
