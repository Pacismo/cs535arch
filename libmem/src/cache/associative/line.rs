//! Represents a line of data
use libseis::types::Word;
use serde::Serialize;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Serialize)]
pub struct Line {
    /// Whether the line is dirty
    pub dirty: bool,
    /// The tag of the line
    pub tag: Word,
    /// The data stored in the line
    pub data: Box<[u8]>,
}

impl Deref for Line {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for Line {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
