use libseis::types::{Byte, Short, Word};

use crate::cache::{Cache, LineData, Status};

use super::Line;

/// Represents an N-way set-associative cache.
#[derive(Debug)]
pub struct MultiAssociative {
    set_bits: usize,
    off_bits: usize,
    ways: usize,
    sets: Box<[Option<Box<Line>>]>,
}

impl Cache for MultiAssociative {
    fn get_byte(&mut self, address: Word) -> crate::cache::ReadResult<Byte> {
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

    fn get_short(&mut self, address: Word) -> crate::cache::ReadResult<Short> {
        todo!()
    }

    fn get_word(&mut self, address: Word) -> crate::cache::ReadResult<Word> {
        todo!()
    }

    fn write_byte(&mut self, address: Word, data: Byte) -> crate::cache::Status {
        todo!()
    }

    fn write_short(&mut self, address: Word, data: Short) -> crate::cache::Status {
        todo!()
    }

    fn write_word(&mut self, address: Word, data: Word) -> crate::cache::Status {
        todo!()
    }

    fn has_address(&self, address: Word) -> bool {
        todo!()
    }

    fn line_len(&self) -> usize {
        todo!()
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

    fn write_line(
        &mut self,
        address: Word,
        memory: &mut crate::memory::Memory,
    ) -> crate::cache::LineReadStatus {
        todo!()
    }

    fn get_lines(&self) -> Vec<Option<LineData>> {
        self.sets
            .chunks(self.ways)
            .zip(0..)
            .flat_map(|(line, set)| {
                line.iter().map(move |line| {
                    line.as_ref().map(|line| LineData {
                        address_base: self.construct_address(line.tag, set, 0),
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
        todo!()
    }

    fn word_at(&self, address: Word) -> Option<Word> {
        todo!()
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

    /// Gets a slice representing the set.
    fn set(&self, set: usize) -> &[Option<Box<Line>>] {
        &self.sets[set..set + self.ways]
    }

    /// Gets a slice representing the set, mutably.
    fn set_mut(&mut self, set: usize) -> &mut [Option<Box<Line>>] {
        &mut self.sets[set..set + self.ways]
    }

    /// Splits an address into its constituent *tag*, *set*, and *offset* indices.
    fn split_address(&self, address: Word) -> (Word, usize, usize) {
        let set_shift = self.off_bits;
        let set_mask = (1 << self.set_bits) - 1;
        let tag_shift = self.off_bits + self.set_bits;
        let tag_mask = (1 << self.tag_bits()) - 1;
        let off_mask = (1 << self.off_bits) - 1;

        let tag = (address >> tag_shift) & tag_mask;
        let set = (address >> set_shift) & set_mask;
        let off = address & off_mask;

        (tag, set as usize, off as usize)
    }

    /// Constructs an address from its constituent *tag*, *set*, and *offset* indices.
    fn construct_address(&self, tag: Word, set: Word, off: Word) -> Word {
        let set_shift = self.off_bits;
        let set_mask = (1 << self.set_bits) - 1;
        let tag_shift = self.off_bits + self.set_bits;
        let tag_mask = (1 << self.tag_bits()) - 1;
        let off_mask = (1 << self.off_bits) - 1;

        ((tag & tag_mask) << tag_shift) | ((set & set_mask) << set_shift) | (off & off_mask)
    }

    /// Boxes the self to produce a dyn [`Cache`]
    #[inline(always)]
    #[track_caller]
    pub fn boxed(self) -> Box<dyn Cache> {
        Box::new(self)
    }
}
