use super::*;
use ReadRegister::*;

#[derive(Debug, Clone)]
enum ReadRegister {
    Populated(Word, [u8; 4]),
    Empty,
}

impl Serialize for ReadRegister {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_none()
    }
}

/// This is a special cache type that only contains a single "read register".
///
/// It only contains a single register with an address and a value.
#[derive(Debug)]
pub struct NullCache(ReadRegister);

impl<'a> Cache<'a> for NullCache {
    fn get_byte(&mut self, address: Word) -> ReadResult<Byte> {
        if let Populated(a, b) = self.0 {
            if a == address {
                self.0 = Empty;
                Ok(b[0])
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

                Ok(Short::from_be_bytes([b[0], b[1]]))
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
                Ok(Word::from_be_bytes(b))
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

    fn check_address(&self, _: Word) -> Status {
        Status::Disabled
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
        self.0 = Populated(address, memory.read_word(address).to_be_bytes());
        LineReadStatus::Disabled
    }

    fn flush(&mut self, _: &mut Memory) -> usize {
        0
    }

    fn dirty_lines(&self) -> usize {
        0
    }

    fn get_lines(&self) -> Vec<Option<LineData>> {
        match &self.0 {
            Populated(addr, val) => {
                vec![Some((*addr, false, val.as_ref()).into())]
            }
            Empty => vec![None],
        }
    }

    fn byte_at(&self, _: Word) -> Option<Byte> {
        None
    }

    fn short_at(&self, _: Word) -> Option<Short> {
        None
    }

    fn word_at(&self, _: Word) -> Option<Word> {
        None
    }
}

impl Serialize for NullCache {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;

        let mut seq = serializer.serialize_seq(Some(1))?;
        seq.serialize_element(&self.0)?;
        seq.end()
    }
}

impl<'a> NullCache {
    #[inline(always)]
    pub fn new() -> Self {
        Self(ReadRegister::Empty)
    }

    #[inline(always)]
    pub fn boxed(self) -> Box<dyn Cache<'a>> {
        Box::new(self)
    }
}
