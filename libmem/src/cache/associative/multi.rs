use super::{construct_address, line::{Line, LineSer}, split_address};
use crate::{
    cache::{Cache, LineData, LineReadStatus, ReadResult, Status},
    memory::Memory,
};
use libseis::types::{Byte, Short, Word};
use serde::Serialize;
use std::mem::take;

/// Represents an N-way set-associative cache.
#[derive(Debug)]
pub struct MultiAssociative {
    set_bits: usize,
    off_bits: usize,
    ways: usize,
    sets: Box<[Option<Box<Line>>]>,
}

impl<'a> Cache<'a> for MultiAssociative {
    fn get_byte(&mut self, address: Word) -> ReadResult<Byte> {
        let (tag, set, off) = self.split_address(address);
        let set = self.set_mut(set);

        // Count the number of null pointers in the set
        let mut nulls = 0;

        if let Some((i, s)) = set.iter_mut().enumerate().find_map(|(i, s)| match s {
            Some(s) if s.tag == tag => Some((i, s)),
            Some(_) => None,
            None => {
                nulls += 1;
                None
            }
        }) {
            // Take the desired value
            let v = s[off];
            // Rotate the set right to put the taken value in the front
            set[0..=i].rotate_right(1);

            Ok(v)
        } else if nulls == 0 {
            Err(Status::Conflict)
        } else {
            Err(Status::Cold)
        }
    }

    fn get_short(&mut self, address: Word) -> ReadResult<Short> {
        let (tag, set, off) = self.split_address(address);

        if off < self.line_len() - 1 {
            let set = self.set_mut(set);
            let mut nulls = 0;

            if let Some((i, s)) = set.iter_mut().enumerate().find_map(|(i, s)| match s {
                Some(s) if s.tag == tag => Some((i, s)),
                Some(_) => None,
                None => {
                    nulls += 1;
                    None
                }
            }) {
                let v = [s[off], s[off + 1]];
                set[0..=i].rotate_right(1);
                Ok(Short::from_be_bytes(v))
            } else if nulls == 0 {
                Err(Status::Conflict)
            } else {
                Err(Status::Cold)
            }
        } else {
            let (otag, oset) = if set + 1 < self.sets.len() / self.ways {
                (tag, set + 1)
            } else {
                (tag + 1, 0)
            };

            let mut set = self.set_mut(set);
            let mut nulls = 0;
            let mut bytes = [0; 2];

            if let Some((i, s)) = set.iter_mut().enumerate().find_map(|(i, s)| match s {
                Some(s) if s.tag == tag => Some((i, s)),
                Some(_) => None,
                None => {
                    nulls += 1;
                    None
                }
            }) {
                bytes[0] = s[off];
                set[0..=i].rotate_right(1);
            } else if nulls == 0 {
                return Err(Status::Conflict);
            } else {
                return Err(Status::Cold);
            }

            set = self.set_mut(oset);
            nulls = 0;

            if let Some((i, s)) = set.iter_mut().enumerate().find_map(|(i, s)| match s {
                Some(s) if s.tag == otag => Some((i, s)),
                Some(_) => None,
                None => {
                    nulls += 1;
                    None
                }
            }) {
                bytes[1] = s[0];
                set[0..=i].rotate_right(1);
            } else if nulls == 0 {
                return Err(Status::Conflict);
            } else {
                return Err(Status::Cold);
            }

            Ok(Short::from_be_bytes(bytes))
        }
    }

    fn get_word(&mut self, address: Word) -> ReadResult<Word> {
        let (tag, set, off) = self.split_address(address);

        if off < self.line_len() - 3 {
            let set = self.set_mut(set);
            let mut nulls = 0;

            if let Some((i, s)) = set.iter_mut().enumerate().find_map(|(i, s)| match s {
                Some(s) if s.tag == tag => Some((i, s)),
                Some(_) => None,
                None => {
                    nulls += 1;
                    None
                }
            }) {
                let v = [s[off], s[off + 1], s[off + 2], s[off + 3]];
                set[0..=i].rotate_right(1);
                Ok(Word::from_be_bytes(v))
            } else if nulls == 0 {
                Err(Status::Conflict)
            } else {
                Err(Status::Cold)
            }
        } else {
            let (otag, oset) = if set + 1 < self.sets.len() / self.ways {
                (tag, set + 1)
            } else {
                (tag + 1, 0)
            };

            let mut set = self.set_mut(set);
            let mut nulls = 0;
            let mut bytes = [0; 4];
            let mut index = 0;

            if let Some((i, s)) = set.iter_mut().enumerate().find_map(|(i, s)| match s {
                Some(s) if s.tag == tag => Some((i, s)),
                Some(_) => None,
                None => {
                    nulls += 1;
                    None
                }
            }) {
                for &byte in &s[off..] {
                    bytes[index] = byte;
                    index += 1;
                }

                set[0..=i].rotate_right(1);
            } else if nulls == 0 {
                return Err(Status::Conflict);
            } else {
                return Err(Status::Cold);
            }

            set = self.set_mut(oset);
            nulls = 0;

            if let Some((i, s)) = set.iter_mut().enumerate().find_map(|(i, s)| match s {
                Some(s) if s.tag == otag => Some((i, s)),
                Some(_) => None,
                None => {
                    nulls += 1;
                    None
                }
            }) {
                for &byte in &s[0..4 - index] {
                    bytes[index] = byte;
                    index += 1;
                }

                set[0..=i].rotate_right(1);
            } else if nulls == 0 {
                return Err(Status::Conflict);
            } else {
                return Err(Status::Cold);
            }

            Ok(Word::from_be_bytes(bytes))
        }
    }

    fn write_byte(&mut self, address: Word, data: Byte) -> Status {
        let (tag, set, off) = self.split_address(address);
        let sets = self.set_mut(set);
        let mut nulls = 0;

        if let Some((i, s)) = sets.iter_mut().enumerate().find_map(|(i, s)| match s {
            Some(s) if s.tag == tag => Some((i, s)),
            Some(_) => None,
            None => {
                nulls += 1;
                None
            }
        }) {
            s[off] = data;
            s.dirty = true;
            sets[..=i].rotate_right(1);
            Status::Hit
        } else if nulls == 0 {
            Status::Conflict
        } else {
            Status::Cold
        }
    }

    fn write_short(&mut self, address: Word, data: Short) -> Status {
        let bytes = data.to_be_bytes();
        let (tag, set, off) = self.split_address(address);

        if off < self.line_len() - 1 {
            let set = self.set_mut(set);
            let mut nulls = 0;

            if let Some((i, s)) = set.iter_mut().enumerate().find_map(|(i, s)| match s {
                Some(s) if s.tag == tag => Some((i, s)),
                Some(_) => None,
                None => {
                    nulls += 1;
                    None
                }
            }) {
                s[off] = bytes[0];
                s[off + 1] = bytes[1];
                s.dirty = true;
                set[0..=i].rotate_right(1);
                Status::Hit
            } else if nulls == 0 {
                Status::Conflict
            } else {
                Status::Cold
            }
        } else {
            let (otag, oset) = if set + 1 < self.sets.len() / self.ways {
                (tag, set + 1)
            } else {
                (tag + 1, 0)
            };

            let mut nulls = [0, 0];

            let first = self
                .set_mut(set)
                .iter_mut()
                .enumerate()
                .find_map(|(i, s)| match s {
                    Some(set) if set.tag == tag => Some((i, take(s).unwrap())),
                    Some(_) => None,
                    None => {
                        nulls[0] += 1;
                        None
                    }
                });
            let second = self
                .set_mut(oset)
                .iter_mut()
                .enumerate()
                .find_map(|(i, s)| match s {
                    Some(set) if set.tag == otag => Some((i, take(s).unwrap())),
                    Some(_) => None,
                    None => {
                        nulls[1] += 1;
                        None
                    }
                });

            if first.is_some() && second.is_some() {
                let (i, mut s) = first.unwrap();
                let (j, mut t) = second.unwrap();

                s[off] = bytes[0];
                s.dirty = true;
                t[0] = bytes[1];
                t.dirty = true;

                let mut sets = self.set_mut(set);
                sets[..=i].rotate_right(1);
                sets[0] = Some(s);

                sets = self.set_mut(oset);
                sets[..=j].rotate_right(1);
                sets[0] = Some(t);

                Status::Hit
            } else {
                if let Some((i, s)) = first {
                    self.set_mut(set)[i] = Some(s);
                }
                if let Some((j, s)) = second {
                    self.set_mut(oset)[j] = Some(s);
                }

                if nulls[0] == 0 || nulls[1] == 0 {
                    Status::Conflict
                } else {
                    Status::Cold
                }
            }
        }
    }

    fn write_word(&mut self, address: Word, data: Word) -> Status {
        let (tag, set, off) = self.split_address(address);
        let bytes = data.to_be_bytes();

        if off < self.line_len() - 3 {
            let set = self.set_mut(set);
            let mut nulls = 0;

            if let Some((i, s)) = set.iter_mut().enumerate().find_map(|(i, s)| match s {
                Some(s) if s.tag == tag => Some((i, s)),
                Some(_) => None,
                None => {
                    nulls += 1;
                    None
                }
            }) {
                s[off..off + 4].copy_from_slice(&bytes);
                s.dirty = true;

                set[0..=i].rotate_right(1);
                Status::Hit
            } else if nulls == 0 {
                Status::Conflict
            } else {
                Status::Cold
            }
        } else {
            let (otag, oset) = if set + 1 < self.sets.len() / self.ways {
                (tag, set + 1)
            } else {
                (tag + 1, 0)
            };

            let mut nulls = [0, 0];

            let first = self
                .set_mut(set)
                .iter_mut()
                .enumerate()
                .find_map(|(i, s)| match s {
                    Some(set) if set.tag == tag => Some((i, take(s).unwrap())),
                    Some(_) => None,
                    None => {
                        nulls[0] += 1;
                        None
                    }
                });
            let second = self
                .set_mut(oset)
                .iter_mut()
                .enumerate()
                .find_map(|(i, s)| match s {
                    Some(set) if set.tag == otag => Some((i, take(s).unwrap())),
                    Some(_) => None,
                    None => {
                        nulls[1] += 1;
                        None
                    }
                });

            if first.is_some() && second.is_some() {
                let (i, mut s) = first.unwrap();
                let (j, mut t) = second.unwrap();
                let mut index = 0;

                for byte in &mut s[off..] {
                    *byte = bytes[index];
                    index += 1;
                }

                t[..4 - index].copy_from_slice(&bytes[index..]);

                s.dirty = true;
                t.dirty = true;

                self.set_mut(set)[..=i].rotate_right(1);
                self.set_mut(oset)[..=j].rotate_right(1);

                Status::Hit
            } else {
                if let Some((i, s)) = first {
                    self.set_mut(set)[i] = Some(s);
                }
                if let Some((j, s)) = second {
                    self.set_mut(oset)[j] = Some(s);
                }

                if nulls[0] == 0 || nulls[1] == 0 {
                    Status::Conflict
                } else {
                    Status::Cold
                }
            }
        }
    }

    fn check_address(&self, address: Word) -> Status {
        let (tag, set, _) = self.split_address(address);
        let mut lines = 0;

        for line in self.set(set).iter().filter_map(|s| s.as_ref()) {
            lines += 1;
            if line.tag == tag {
                return Status::Hit;
            }
        }

        if lines == self.ways {
            Status::Conflict
        } else {
            Status::Cold
        }
    }

    fn line_len(&self) -> usize {
        2usize.pow(self.off_bits as u32)
    }

    fn within_line(&self, address: Word, length: usize) -> bool {
        let (.., off) = self.split_address(address);
        off + length - 1 < 2usize.pow(self.off_bits as u32)
    }

    fn invalidate_line(&mut self, address: Word) -> bool {
        let (tag, set, _) = self.split_address(address);
        let sets = self.set_mut(set);

        if let Some(i) = sets.iter_mut().enumerate().find_map(|(i, s)| match s {
            Some(s) if s.tag == tag => Some(i),
            _ => None,
        }) {
            // Delete the boxed data (Rust drop semantics de-allocate the memory)
            sets[i] = None;
            // Rotate the slice from the index onward left, such that the deallocated pointer is at the end
            sets[i..].rotate_left(1);
            true
        } else {
            false
        }
    }

    fn write_line(&mut self, address: Word, memory: &mut Memory) -> LineReadStatus {
        if self.check_address(address).is_hit() {
            return LineReadStatus::Skipped;
        }

        let set_bits = self.set_bits;
        let off_bits = self.off_bits;

        let (tag, set, _) = self.split_address(address);
        let line_len = self.line_len();

        let sets = self.set_mut(set);
        let last = &mut sets[sets.len() - 1];

        if let Some(mut line) = take(last) {
            if line.dirty {
                let construct_address =
                    construct_address(line.tag, set as u32, 0, set_bits, off_bits);

                line.data
                    .iter()
                    .zip(construct_address..)
                    .for_each(|(&byte, address)| memory.write_byte(address, byte));
            }

            line.tag = tag;
            line.dirty = false;
            memory.read_words_to(address, &mut line.data);

            *last = Some(line);

            sets.rotate_right(1);

            LineReadStatus::Evicted
        } else {
            let new_line = Box::new(Line {
                tag,
                dirty: false,
                data: memory.read_words(address, line_len),
            });

            *last = Some(new_line);

            sets.rotate_right(1);

            LineReadStatus::Inserted
        }
    }

    fn flush(&mut self, memory: &mut Memory) -> usize {
        let set_bits = self.set_bits;
        let off_bits = self.off_bits;

        self.sets
            .chunks_mut(self.ways)
            .enumerate()
            .flat_map(|(i, s)| {
                s.iter_mut().filter_map(move |s| match s {
                    Some(s) if s.dirty => Some((i, s)),
                    _ => None,
                })
            })
            .map(|(i, s)| {
                let addr = construct_address(s.tag, i as u32, 0, set_bits, off_bits);

                s.data
                    .iter()
                    .zip(addr..)
                    .for_each(|(&byte, address)| memory.write_byte(address, byte));
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
            .chunks(self.ways)
            .zip(0..)
            .flat_map(|(line, set)| {
                line.iter().map(move |line| {
                    line.as_ref().map(|line| LineData {
                        base_address: self.construct_address(line.tag, set, 0),
                        dirty: line.dirty,
                        data: line.data.as_ref(),
                    })
                })
            })
            .collect()
    }

    fn byte_at(&self, address: Word) -> Option<Byte> {
        let (tag, set, off) = self.split_address(address);

        self.set(set).iter().find_map(|s| match s {
            Some(s) if s.tag == tag => Some(s[off]),
            _ => None,
        })
    }

    fn short_at(&self, address: Word) -> Option<Short> {
        let (tag, set, off) = self.split_address(address);

        if off < self.line_len() - 1 {
            if let Some(s) = self.set(set).iter().find_map(|s| match s {
                Some(s) if s.tag == tag => Some(s),
                _ => None,
            }) {
                let v = [s[off], s[off + 1]];

                Some(Short::from_be_bytes(v))
            } else {
                None
            }
        } else {
            let (otag, oset) = if set + 1 < self.sets.len() / self.ways {
                (tag, set + 1)
            } else {
                (tag + 1, 0)
            };

            let mut bytes = [0; 2];

            if let Some(s) = self.set(set).iter().find_map(|s| match s {
                Some(s) if s.tag == tag => Some(s),
                _ => None,
            }) {
                bytes[0] = s[off];
            } else {
                return None;
            }

            if let Some(s) = self.set(oset).iter().find_map(|s| match s {
                Some(s) if s.tag == otag => Some(s),
                _ => None,
            }) {
                bytes[1] = s[0];
            } else {
                return None;
            }

            Some(Short::from_be_bytes(bytes))
        }
    }

    fn word_at(&self, address: Word) -> Option<Word> {
        let (tag, set, off) = self.split_address(address);

        if off < self.line_len() - 3 {
            if let Some(s) = self.set(set).iter().find_map(|s| match s {
                Some(s) if s.tag == tag => Some(s),
                _ => None,
            }) {
                let v = [s[off], s[off + 1], s[off + 2], s[off + 3]];
                Some(Word::from_be_bytes(v))
            } else {
                None
            }
        } else {
            let (otag, oset) = if set + 1 < self.sets.len() / self.ways {
                (tag, set + 1)
            } else {
                (tag + 1, 0)
            };

            let mut bytes = [0; 4];
            let mut index = 0;

            if let Some(s) = self.set(set).iter().find_map(|s| match s {
                Some(s) if s.tag == tag => Some(s),
                _ => None,
            }) {
                for &byte in &s[off..] {
                    bytes[index] = byte;
                    index += 1;
                }
            } else {
                return None;
            }

            if let Some(s) = self.set(oset).iter().find_map(|s| match s {
                Some(s) if s.tag == otag => Some(s),
                _ => None,
            }) {
                for &byte in &s[0..4 - index] {
                    bytes[index] = byte;
                    index += 1;
                }
            } else {
                return None;
            }

            Some(Word::from_be_bytes(bytes))
        }
    }
}

impl Serialize for MultiAssociative {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_seq(self.sets.iter().enumerate().map(|(i, line)| LineSer {
            line,
            set: (i / self.ways) as Word,
            set_bits: self.set_bits,
            off_bits: self.off_bits,
        }))
    }
}

impl MultiAssociative {
    /// Creates a new [`MultiAssociative`] with an offset bitfield width and a set bitfield width set at runtime.
    ///
    /// `off_bits` must be between 2 and 32, inclusive.
    ///
    /// `set_bits` must be between 0 and 30, inclusive.
    ///
    /// `off_bits + set_bits` must be at most 32.
    ///
    /// `ways` must be at least 1.
    ///
    /// The remaining bits are used for the tag field.
    pub fn new(off_bits: usize, set_bits: usize, ways: usize) -> Self {
        assert!(off_bits >= 2, "off_bits must be at least 2");
        assert!(off_bits <= 32, "off_bits must be at most 32");
        assert!(set_bits <= 30, "set_bits can be at most 30");
        assert!(
            off_bits + set_bits <= 32,
            "off_bits + set_bits cannot exceed 32"
        );
        assert!(ways > 0, "ways must be greater than 0");

        let mut lines = vec![];

        lines.resize_with(2usize.pow(set_bits as u32) * ways, || None);

        Self {
            sets: lines.into_boxed_slice(),
            ways,
            set_bits,
            off_bits,
        }
    }

    /// Returns the number of ways in this set.
    pub fn ways(&self) -> usize {
        self.ways
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

    /// Gets range representing the set.
    fn set(&self, set: usize) -> &[Option<Box<Line>>] {
        let base = set * self.ways;
        &self.sets[base..base + self.ways]
    }

    /// Gets a slice representing the set, mutably.
    fn set_mut(&mut self, set: usize) -> &mut [Option<Box<Line>>] {
        let base = set * self.ways;
        &mut self.sets[base..base + self.ways]
    }

    /// Splits an address into its constituent *tag*, *set*, and *offset* indices.
    fn split_address(&self, address: Word) -> (Word, usize, usize) {
        split_address(address, self.set_bits, self.off_bits)
    }

    /// Constructs an address from its constituent *tag*, *set*, and *offset* indices.
    #[inline]
    fn construct_address(&self, tag: Word, set: Word, off: Word) -> Word {
        construct_address(tag, set, off, self.set_bits, self.off_bits)
    }

    /// Boxes the self to produce a dyn [`Cache`]
    #[inline(always)]
    #[track_caller]
    pub fn boxed<'a>(self) -> Box<dyn Cache<'a>> {
        Box::new(self)
    }
}
