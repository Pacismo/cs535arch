use super::*;

/// This is a dummy unit type implementing [`Cache`].
///
/// It exists *only* to act as an absent cache.
#[derive(Debug)]
pub struct NullCache;

impl Cache for NullCache {
    fn read_byte(&self, _: Word) -> ReadResult<Byte> {
        Err(Status::Disabled)
    }

    fn read_short(&self, _: Word) -> ReadResult<Short> {
        Err(Status::Disabled)
    }

    fn read_word(&self, _: Word) -> ReadResult<Word> {
        Err(Status::Disabled)
    }

    fn get_byte(&mut self, _: Word) -> ReadResult<Byte> {
        Err(Status::Disabled)
    }

    fn get_short(&mut self, _: Word) -> ReadResult<Short> {
        Err(Status::Disabled)
    }

    fn get_word(&mut self, _: Word) -> ReadResult<Word> {
        Err(Status::Disabled)
    }

    fn write_byte(&mut self, _: Word, _: Byte) -> Status {
        Status::Disabled
    }

    fn write_short(&mut self, _: Word, _: Short) -> Status {
        Status::Disabled
    }

    fn write_word(&mut self, _: Word, _: Word) -> Status {
        Status::Disabled
    }

    fn has_address(&self, _: Word) -> bool {
        false
    }

    fn line_len(&self) -> usize {
        0
    }

    fn within_line(&self, _: Word, _: usize) -> bool {
        false
    }

    fn write_line(&mut self, _: Word, _: &mut Memory) -> bool {
        false
    }
}

impl NullCache {
    #[inline(always)]
    #[track_caller]
    pub fn new() -> Self {
        Self
    }

    #[inline(always)]
    #[track_caller]
    pub fn boxed(self) -> Box<dyn Cache> {
        Box::new(self)
    }
}
