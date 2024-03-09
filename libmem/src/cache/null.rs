use super::*;

/// This is a dummy unit type implementing [`Cache`].
///
/// It exists *only* to act as an absent cache.
#[derive(Debug)]
pub struct NullCache;

impl Cache for NullCache {
    #[inline(always)]
    fn read_byte(&self, address: Word) -> ReadResult<Byte> {
        Err(Status::Disabled)
    }

    #[inline(always)]
    fn read_short(&self, address: Word) -> ReadResult<Short> {
        Err(Status::Disabled)
    }

    #[inline(always)]
    fn read_word(&self, address: Word) -> ReadResult<Word> {
        Err(Status::Disabled)
    }

    #[inline(always)]
    fn get_byte(&mut self, _: Word) -> ReadResult<Byte> {
        Err(Status::Disabled)
    }

    #[inline(always)]
    fn get_short(&mut self, _: Word) -> ReadResult<Short> {
        Err(Status::Disabled)
    }

    #[inline(always)]
    fn get_word(&mut self, _: Word) -> ReadResult<Word> {
        Err(Status::Disabled)
    }

    #[inline(always)]
    fn write_byte(&mut self, _: Word, _: Byte) -> bool {
        false
    }

    #[inline(always)]
    fn write_short(&mut self, _: Word, _: Short) -> bool {
        false
    }

    #[inline(always)]
    fn write_word(&mut self, _: Word, _: Word) -> bool {
        false
    }

    #[inline(always)]
    fn has_address(&self, _: Word) -> bool {
        false
    }

    #[inline(always)]
    fn line_len(&self) -> usize {
        0
    }

    #[inline(always)]
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
