use libseis::types::Word;
use serde::Serialize;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Serialize)]
pub struct Line {
    pub dirty: bool,
    pub tag: Word,
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
