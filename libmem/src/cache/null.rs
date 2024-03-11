use super::*;

#[derive(Debug, Clone, Copy)]
enum ReadRegister {
    Populated(Word, Word),
    Empty,
}

use ReadRegister::*;

/// This is a special cache type that only contains a single "read register".
///
/// It only contains a single register with an address and a value.
#[derive(Debug)]
pub struct NullCache(ReadRegister);

impl Cache for NullCache {
    fn get_byte(&mut self, address: Word) -> ReadResult<Byte> {
        if let Populated(a, b) = self.0 {
            if a == address {
                self.0 = Empty;
                Ok(b.to_be_bytes()[0])
            } else {
                Err(Status::Disabled)
            }
        } else {
            Err(Status::Disabled)
        }
    }

    fn get_short(&mut self, address: Word) -> ReadResult<Short> {
        if let Populated(a, b) = self.0 {
            if a == address {
                self.0 = Empty;

                let bytes = b.to_be_bytes();
                Ok(Short::from_be_bytes([bytes[0], bytes[1]]))
            } else {
                Err(Status::Disabled)
            }
        } else {
            Err(Status::Disabled)
        }
    }

    fn get_word(&mut self, address: Word) -> ReadResult<Word> {
        if let Populated(a, b) = self.0 {
            if a == address {
                self.0 = Empty;
                Ok(b)
            } else {
                Err(Status::Disabled)
            }
        } else {
            Err(Status::Disabled)
        }
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

    fn invalidate_line(&mut self, _: Word) -> bool {
        false
    }

    fn write_line(&mut self, address: Word, memory: &mut Memory) -> LineReadStatus {
        self.0 = Populated(address, memory.read_word(address));
        LineReadStatus::Disabled
    }
}

impl NullCache {
    #[inline(always)]
    pub fn new() -> Self {
        Self(ReadRegister::Empty)
    }

    #[inline(always)]
    pub fn boxed(self) -> Box<dyn Cache> {
        Box::new(self)
    }
}
