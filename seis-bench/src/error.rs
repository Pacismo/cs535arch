use std::{any::Any, error::Error as StdError, fmt::Display, io, sync::mpsc};

use crate::bench;

#[derive(Debug)]
pub enum Error {
    String(String),
    IoError(io::Error),
    ThreadPoolBuildError(rayon::ThreadPoolBuildError),
    TomlDeserializeError(toml::de::Error),
    SendError(mpsc::SendError<bench::State>),
    ThreadError(Box<dyn Any + Send + 'static>),
}

unsafe impl Send for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::String(s) => write!(f, "{s}"),
            Error::IoError(e) => write!(f, "{e}"),
            Error::ThreadPoolBuildError(e) => write!(f, "{e}"),
            Error::TomlDeserializeError(e) => write!(f, "{e}"),
            Error::SendError(e) => write!(f, "{e}"),
            Error::ThreadError(_) => write!(f, "Thread Error"),
        }
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

impl From<rayon::ThreadPoolBuildError> for Error {
    fn from(value: rayon::ThreadPoolBuildError) -> Self {
        Self::ThreadPoolBuildError(value)
    }
}

impl From<toml::de::Error> for Error {
    fn from(value: toml::de::Error) -> Self {
        Self::TomlDeserializeError(value)
    }
}

impl From<mpsc::SendError<bench::State>> for Error {
    fn from(value: mpsc::SendError<bench::State>) -> Self {
        Self::SendError(value)
    }
}

impl From<Box<dyn Any + Send + 'static>> for Error {
    fn from(value: Box<dyn Any + Send + 'static>) -> Self {
        Self::ThreadError(value)
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::String(_) => None,
            Error::IoError(e) => Some(e),
            Error::ThreadPoolBuildError(e) => Some(e),
            Error::TomlDeserializeError(e) => Some(e),
            Error::SendError(e) => Some(e),
            Error::ThreadError(_) => None,
        }
    }
}
