use libseis::types::Word;
use serde::Serialize;
use std::ops::{Deref, DerefMut};

use crate::cache::associative::construct_address;

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

pub struct LineSer<'a> {
    pub line: &'a Option<Box<Line>>,
    pub set: Word,
    pub set_bits: usize,
    pub off_bits: usize,
}

impl<'a> Serialize for LineSer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(if self.line.is_some() {
            Some(4)
        } else {
            Some(0)
        })?;
        if let Some(line) = self.line {
            let address = construct_address(line.tag, self.set, 0, self.set_bits, self.off_bits);
            map.serialize_entry("address", &address)?;
            map.serialize_entry("tag", &line.tag)?;
            map.serialize_entry("dirty", &line.dirty)?;
            map.serialize_entry("data", &line.data)?;
        }
        map.end()
    }
}
