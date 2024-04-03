use super::{construct_address, line::Line, split_address};
use crate::{
    cache::{Cache, LineData, LineReadStatus, ReadResult, Status},
    memory::Memory,
};
use libseis::types::{Byte, Short, Word};
use std::mem::take;

/// Represents a one-way set-associative cache.
///
/// The number of sets and words that can be stored in the cache are determined at runtime.
#[derive(Debug)]
pub struct Associative {
    set_bits: usize,
    off_bits: usize,
    sets: Box<[Option<Box<Line>>]>,
}

impl Cache for Associative {
    fn get_byte(&mut self, address: Word) -> ReadResult<Byte> {
        let (tag, set, off) = self.split_address(address);

        match &self.sets[set] {
            Some(set) if set.tag == tag => Ok(set[off]),
            Some(_) => Err(Status::Conflict),
            None => Err(Status::Cold),
        }
    }

    fn get_short(&mut self, address: Word) -> ReadResult<Short> {
        let (tag, set, off) = self.split_address(address);

        if off < self.line_len() - 1 {
            match &self.sets[set] {
                Some(set) if set.tag == tag => Ok(Short::from_be_bytes([set[off], set[off + 1]])),
                Some(_) => Err(Status::Conflict),
                None => Err(Status::Cold),
            }
        } else {
            let (otag, oset) = if set + 1 < self.sets.len() {
                (tag, set + 1)
            } else {
                (tag + 1, 0)
            };
            match (&self.sets[set], &self.sets[oset]) {
                (Some(set1), Some(set2)) if set1.tag == tag && set2.tag == otag => {
                    Ok(Short::from_be_bytes([set1[set1.len() - 1], set2[0]]))
                }
                (Some(_), Some(_)) => Err(Status::Conflict),
                _ => Err(Status::Cold),
            }
        }
    }

    fn get_word(&mut self, address: Word) -> ReadResult<Word> {
        let (tag, set, off) = self.split_address(address);

        if off < self.line_len() - 3 {
            match &self.sets[set] {
                Some(set) if set.tag == tag => Ok(Word::from_be_bytes([
                    set[off],
                    set[off + 1],
                    set[off + 2],
                    set[off + 3],
                ])),
                Some(_) => Err(Status::Conflict),
                None => Err(Status::Cold),
            }
        } else {
            let (otag, oset) = if set + 1 < self.sets.len() {
                (tag, set + 1)
            } else {
                (tag + 1, 0)
            };
            let former = (self.off_bits << 16) - off;

            match (&self.sets[set], &self.sets[oset]) {
                (Some(set1), Some(set2)) if set1.tag == tag && set2.tag == otag => {
                    let mut bytes = [0; 4];
                    for i in 0..4 {
                        if i > former {
                            bytes[i] = set2[i - former];
                        } else {
                            bytes[i] = set1[i];
                        }
                    }
                    Ok(Word::from_be_bytes(bytes))
                }
                (Some(_), Some(_)) => Err(Status::Conflict),
                _ => Err(Status::Cold),
            }
        }
    }

    fn write_byte(&mut self, address: Word, data: Byte) -> Status {
        let (tag, set, off) = self.split_address(address);

        if let Some(set) = &mut self.sets[set] {
            if set.tag == tag {
                set.data[off] = data;
                set.dirty = true;
                Status::Hit
            } else {
                Status::Conflict
            }
        } else {
            Status::Cold
        }
    }

    fn write_short(&mut self, address: Word, data: Short) -> Status {
        let (tag, set, off) = self.split_address(address);

        if off < self.line_len() - 1 {
            if let Some(set) = &mut self.sets[set] {
                if set.tag == tag {
                    let bytes = data.to_be_bytes();
                    set.data[off] = bytes[0];
                    set.data[off + 1] = bytes[1];

                    set.dirty = true;

                    Status::Hit
                } else {
                    Status::Conflict
                }
            } else {
                Status::Cold
            }
        } else {
            let (otag, oset) = if set + 1 < self.sets.len() {
                (tag, set + 1)
            } else {
                (tag + 1, 0)
            };

            let mut first = take(&mut self.sets[set]);
            let mut second = take(&mut self.sets[oset]);

            let status = if let Some((first, second)) = first.as_mut().zip(second.as_mut()) {
                if first.tag == tag && second.tag == otag {
                    let bytes = data.to_be_bytes();
                    first.data[off] = bytes[0];
                    second.data[0] = bytes[1];

                    first.dirty = true;
                    second.dirty = true;

                    Status::Hit
                } else {
                    Status::Conflict
                }
            } else {
                Status::Cold
            };

            self.sets[set] = first;
            self.sets[oset] = second;
            status
        }
    }

    fn write_word(&mut self, address: Word, data: Word) -> Status {
        let (tag, set, off) = self.split_address(address);
        let off_mask = (1 << self.off_bits) - 1;

        if off < off_mask - 2 {
            if let Some(set) = &mut self.sets[set] {
                if set.tag == tag {
                    let bytes = data.to_be_bytes();
                    set.data[off] = bytes[0];
                    set.data[off + 1] = bytes[1];
                    set.data[off + 2] = bytes[2];
                    set.data[off + 3] = bytes[3];

                    set.dirty = true;

                    Status::Hit
                } else {
                    Status::Conflict
                }
            } else {
                Status::Cold
            }
        } else {
            let (otag, oset) = if set + 1 < self.sets.len() {
                (tag, set + 1)
            } else {
                (tag + 1, 0)
            };

            let mut first = take(&mut self.sets[set]);
            let mut second = take(&mut self.sets[oset]);

            let status = if let Some((first, second)) = first.as_mut().zip(second.as_mut()) {
                if first.tag == tag && second.tag == otag {
                    let bytes = data.to_be_bytes();

                    for i in off..off + 4 {
                        if i <= off_mask {
                            first.data[i] = bytes[i - off];
                        } else {
                            second.data[i - off_mask] = bytes[i - off];
                        }
                    }

                    first.dirty = true;
                    second.dirty = true;

                    Status::Hit
                } else {
                    Status::Conflict
                }
            } else {
                Status::Cold
            };

            self.sets[set] = first;
            self.sets[oset] = second;
            status
        }
    }

    fn check_address(&self, address: Word) -> Status {
        let (tag, set, _) = self.split_address(address);

        if let Some(set) = &self.sets[set] {
            if set.tag == tag {
                Status::Hit
            } else {
                Status::Conflict
            }
        } else {
            Status::Cold
        }
    }

    fn line_len(&self) -> usize {
        2usize.pow(self.off_bits as u32)
    }

    fn within_line(&self, address: Word, length: usize) -> bool {
        let (.., off) = self.split_address(address);

        // Asserts that a read of length bytes from the address will not overflow to the next set.
        off + length - 1 < 2usize.pow(self.off_bits as u32)
    }

    fn invalidate_line(&mut self, address: Word) -> bool {
        let (tag, set, _) = self.split_address(address);

        if matches!(&self.sets[set], Some(line) if line.tag == tag) {
            // Delete the line in question
            self.sets[set] = None;
            true
        } else {
            false
        }
    }

    fn write_line(&mut self, address: Word, memory: &mut Memory) -> LineReadStatus {
        if self.check_address(address).is_hit() {
            return LineReadStatus::Skipped;
        }

        let (tag, set, _) = self.split_address(address);

        // Flush a previously-existing line if it is dirty. Otherwise, purge its contents.
        if let Some(mut line) = take(&mut self.sets[set]) {
            if line.dirty {
                let addr = self.construct_address(line.tag, set as u32, 0);

                line.data
                    .iter()
                    .zip(addr..)
                    .for_each(|(&byte, address)| memory.write_byte(address, byte));
            }

            line.tag = tag;
            line.dirty = false;
            memory.read_words_to(address, &mut line.data);

            self.sets[set] = Some(line);

            LineReadStatus::Evicted
        } else {
            let new_line = Box::new(Line {
                tag,
                dirty: false,
                data: memory.read_words(address, self.line_len()),
            });

            self.sets[set] = Some(new_line);

            LineReadStatus::Inserted
        }
    }

    fn flush(&mut self, memory: &mut Memory) -> usize {
        let set_bits = self.set_bits;
        let off_bits = self.off_bits;

        self.sets
            .iter_mut()
            .enumerate()
            .filter_map(|(i, s)| match s {
                Some(s) if s.dirty => {
                    let addr = construct_address(s.tag, i as u32, 0, set_bits, off_bits);

                    s.data
                        .iter()
                        .zip(addr..)
                        .for_each(|(&byte, address)| memory.write_byte(address, byte));

                    s.dirty = false;

                    Some(())
                }
                _ => None,
            })
            .count()
    }

    fn dirty_lines(&self) -> usize {
        self.sets
            .iter()
            .filter(|s| matches!(s, Some(s) if s.dirty))
            .count()
    }

    fn get_lines(&self) -> Vec<Option<LineData>> {
        self.sets
            .iter()
            .zip(0..)
            .map(|(line, set)| {
                line.as_ref().map(|line| LineData {
                    base_address: self.construct_address(line.tag, set, 0),
                    dirty: line.dirty,
                    data: line.data.as_ref(),
                })
            })
            .collect()
    }

    fn byte_at(&self, address: Word) -> Option<Byte> {
        let (tag, set, off) = self.split_address(address);

        match &self.sets[set] {
            Some(line) if line.tag == tag => Some(line.data[off]),
            _ => None,
        }
    }

    fn short_at(&self, address: Word) -> Option<Short> {
        let (tag, set, off) = self.split_address(address);

        if off < self.line_len() - 1 {
            match &self.sets[set] {
                Some(set) if set.tag == tag => Some(Short::from_be_bytes([set[off], set[off + 1]])),
                _ => None,
            }
        } else {
            let (otag, oset) = if set + 1 < self.sets.len() {
                (tag, set + 1)
            } else {
                (tag + 1, 0)
            };
            match self.sets[set].as_ref().zip(self.sets[oset].as_ref()) {
                Some((set1, set2)) if set1.tag == tag && set2.tag == otag => {
                    Some(Short::from_be_bytes([set1[off], set2[0]]))
                }
                _ => None,
            }
        }
    }

    fn word_at(&self, address: Word) -> Option<Word> {
        let (tag, set, off) = self.split_address(address);

        if off < self.line_len() - 3 {
            match &self.sets[set] {
                Some(set) if set.tag == tag => Some(Word::from_be_bytes([
                    set[off],
                    set[off + 1],
                    set[off + 2],
                    set[off + 3],
                ])),
                _ => None,
            }
        } else {
            let (otag, oset) = if set + 1 < self.sets.len() {
                (tag, set + 1)
            } else {
                (tag + 1, 0)
            };
            let former = (self.off_bits << 16) - off;

            match self.sets[set].as_ref().zip(self.sets[oset].as_ref()) {
                Some((set1, set2)) if set1.tag == tag && set2.tag == otag => {
                    let mut bytes = [0; 4];
                    for i in 0..4 {
                        if i > former {
                            bytes[i] = set2[i - former];
                        } else {
                            bytes[i] = set1[i];
                        }
                    }
                    Some(Word::from_be_bytes(bytes))
                }
                _ => None,
            }
        }
    }
}

impl Associative {
    /// Creates a new [`Associative`] with an offset bitfield width and a set bitfield width set at runtime.
    ///
    /// `off_bits` must be between 2 and 32, inclusive.
    ///
    /// `set_bits` must be between 0 and 30, inclusive.
    ///
    /// `off_bits + set_bits` must be at most 32.
    ///
    /// The remaining bits are used for the tag field.
    pub fn new(off_bits: usize, set_bits: usize) -> Self {
        assert!(off_bits >= 2, "off_bits must be at least 2");
        assert!(
            off_bits + set_bits <= 32,
            "off_bits + set_bits cannot exceed 32"
        );

        let mut lines = vec![];

        lines.resize_with(2usize.pow(set_bits as u32), || None);

        Self {
            sets: lines.into_boxed_slice(),
            set_bits,
            off_bits,
        }
    }

    /// Returns the number of bits used for the tag.
    pub fn tag_bits(&self) -> usize {
        32 - (self.off_bits + self.set_bits)
    }

    /// Returns the number of bits used for the set.
    pub fn set_bits(&self) -> usize {
        self.set_bits
    }

    /// Returns the number of bits used for the offset.
    pub fn off_bits(&self) -> usize {
        self.off_bits
    }

    /// Splits an address into its constituent *tag*, *set*, and *offset* indices.
    fn split_address(&self, address: Word) -> (Word, usize, usize) {
        split_address(address, self.set_bits, self.off_bits)
    }

    /// Constructs an address from its constituent *tag*, *set*, and *offset* indices.
    fn construct_address(&self, tag: Word, set: Word, off: Word) -> Word {
        construct_address(tag, set, off, self.set_bits, self.off_bits)
    }
}
