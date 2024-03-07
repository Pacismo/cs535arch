use super::*;

#[derive(Debug)]
pub struct Line {
    valid: bool,
    dirty: bool,
    tag: Word,
    data: Box<[u8]>,
}

impl Default for Line {
    fn default() -> Self {
        Self {
            valid: false,
            dirty: false,
            tag: 0,
            data: vec![].into_boxed_slice(),
        }
    }
}

#[derive(Debug)]
pub struct MappedLru {
    tag_bits: usize,
    set_bits: usize,
    off_bits: usize,
    lines: Box<[Line]>,
}

impl Cache for MappedLru {
    fn get_byte(&self, address: Word) -> Option<Byte> {
        let (tag, set, off) = self.split_address(address);

        let set = &self.lines[set];
        if set.valid && set.tag == tag {
            Some(set.data[off])
        } else {
            None
        }
    }

    fn get_short(&self, address: Word) -> Option<Short> {
        Some(Short::from_be_bytes([
            self.get_byte(address)?,
            self.get_byte(address + 1)?,
        ]))
    }

    fn get_word(&self, address: Word) -> Option<Word> {
        Some(Word::from_be_bytes([
            self.get_byte(address)?,
            self.get_byte(address + 1)?,
            self.get_byte(address + 2)?,
            self.get_byte(address + 3)?,
        ]))
    }

    fn write_byte(&mut self, address: Word, data: Byte) -> bool {
        let (tag, set, off) = self.split_address(address);

        let set = &mut self.lines[set];
        if set.valid && set.tag == tag {
            set.data[off] = data;
            set.dirty = true;
            true
        } else {
            false
        }
    }

    fn write_short(&mut self, address: Word, data: Short) -> bool {
        if self.has_address(address) && self.has_address(address + 1) {
            data.to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(|(data, address)| {
                    self.write_byte(address, data);
                });

            true
        } else {
            false
        }
    }

    fn write_word(&mut self, address: Word, data: Word) -> bool {
        if (address..address + 4).all(|v| self.has_address(v)) {
            data.to_be_bytes()
                .into_iter()
                .zip(address..)
                .for_each(|(data, address)| {
                    self.write_byte(address, data);
                });

            true
        } else {
            false
        }
    }

    fn has_address(&self, address: Word) -> bool {
        let (tag, set, _) = self.split_address(address);

        let set = &self.lines[set];
        set.valid && set.tag == tag
    }

    fn line_len(&self) -> usize {
        2usize.pow(self.off_bits as u32)
    }

    fn write_line(&mut self, address: Word, memory: &Memory) -> Option<(Word, Box<[u8]>)> {
        let (tag, set, _) = self.split_address(address);

        let line = &mut self.lines[set];

        let old_line_data = if line.dirty {
            let old_line = std::mem::take(line);
            Some(old_line)
        } else {
            None
        };

        *line = Line {
            valid: true,
            dirty: false,
            tag,
            data: memory.read_bytes(address, 2usize.pow(self.off_bits as u32)),
        };

        old_line_data.map(|old| (self.construct_address(old.tag, set as Word, 0), old.data))
    }
}

impl MappedLru {
    pub fn new(off_bits: usize, set_bits: usize) -> Self {
        assert!(off_bits > 4, "off_bits must be greater than 4");
        assert!(set_bits > 0, "set_bits must be greater than 0");
        let tag_bits = 32 - (off_bits + set_bits);

        let mut lines = vec![];

        lines.resize_with(2usize.pow(set_bits as u32), || Line {
            data: vec![0; 2usize.pow(off_bits as u32)].into_boxed_slice(),
            ..Default::default()
        });

        Self {
            lines: lines.into_boxed_slice(),
            tag_bits,
            set_bits,
            off_bits,
        }
    }

    fn split_address(&self, address: Word) -> (Word, usize, usize) {
        let set_shift = self.off_bits;
        let set_mask = (1 << self.set_bits) - 1;
        let tag_shift = self.off_bits + self.set_bits;
        let tag_mask = (1 << self.tag_bits) - 1;
        let off_mask = (1 << self.off_bits) - 1;

        let tag = (address >> tag_shift) & tag_mask;
        let set = (address >> set_shift) & set_mask;
        let off = address & off_mask;

        (tag, set as usize, off as usize)
    }

    fn construct_address(&self, tag: Word, set: Word, off: Word) -> Word {
        let set_shift = self.off_bits;
        let set_mask = (1 << self.set_bits) - 1;
        let tag_shift = self.off_bits + self.set_bits;
        let tag_mask = (1 << self.tag_bits) - 1;
        let off_mask = (1 << self.off_bits) - 1;

        ((tag & tag_mask) << tag_shift) | ((set & set_mask) << set_shift) | (off & off_mask)
    }
}
