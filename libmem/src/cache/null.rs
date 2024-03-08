use super::*;

#[derive(Debug)]
pub struct NullCache;

impl Cache for NullCache {
    fn get_byte(&mut self, _: Word) -> ReadResult<Byte> {
        Err(Status::Disabled)
    }

    fn get_short(&mut self, _: Word) -> ReadResult<Short> {
        Err(Status::Disabled)
    }

    fn get_word(&mut self, _: Word) -> ReadResult<Word> {
        Err(Status::Disabled)
    }

    fn write_byte(&mut self, _: Word, _: Byte) -> bool {
        false
    }

    fn write_short(&mut self, _: Word, _: Short) -> bool {
        false
    }

    fn write_word(&mut self, _: Word, _: Word) -> bool {
        false
    }

    fn has_address(&self, _: Word) -> bool {
        false
    }

    fn line_len(&self) -> usize {
        0
    }

    fn write_line(&mut self, _: Word, _: &mut Memory) -> bool {
        false
    }
}

impl NullCache {
    #[inline]
    pub fn new() -> Self {
        Self
    }
}
