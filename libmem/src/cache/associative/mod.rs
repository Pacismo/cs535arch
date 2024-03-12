mod single;
mod multi;

pub use single::Associative;
use libseis::types::Word;
pub use multi::MultiAssociative;
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct Line {
    dirty: bool,
    tag: Word,
    data: Box<[u8]>,
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
