//! The memory module, [`Memory`], for use with the memory module.
//!
//! The datastructure in this module contains a set of dynamically-allocated pages,
//! which are allocated on write.

use std::mem::{size_of, take, transmute};

use libseis::{
    pages::PAGE_SIZE,
    types::{Byte, Short, Word},
};

type Page = [Word; PAGE_SIZE / size_of::<Word>()];

fn allocate_page() -> Box<Page> {
    Box::new([0; PAGE_SIZE / size_of::<Word>()])
}

/// Memory representation that only allocates pages when they're written to.
///
/// Cannot add new pages after construction -- addressing out of bounds will lead to a [`panic`].
///
/// This takes extensive advantage of Rust's [`u32::from_be_bytes`] and [`u32::to_be_bytes`].
pub struct Memory {
    pages: Box<[Option<Box<Page>>]>,
}

impl std::fmt::Debug for Memory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Memory")
            .field(
                "pages",
                &self.pages.iter().map(Option::is_some).collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl Memory {
    pub fn new(count: usize) -> Self {
        assert!(count > 0, "Count must be greater than 0");

        Self {
            pages: vec![None; count].into_boxed_slice(),
        }
    }

    pub fn max_address(&self) -> Word {
        ((self.pages.len() as Word) << 16) - 1
    }

    pub fn read_byte(&self, address: Word) -> Byte {
        let address = address as usize % (self.pages.len() << 16);
        let page = (address & 0xFFFF_0000) >> 16;

        if let Some(page) = &self.pages[page] {
            let offset = address & 0x3;
            let word = page[(address & 0xFFFC) >> 2].to_be_bytes();

            word[offset]
        } else {
            0
        }
    }

    pub fn read_short(&self, address: Word) -> Short {
        let address = address as usize % (self.pages.len() << 16);

        match (address & 0x3 != 0x3, address & 0xFFFF != 0xFFFF) {
            // neither crosses a word boundary nor crosses a page boundary
            (true, true) => {
                let page = (address & 0xFFFF_0000) >> 16;
                if let Some(page) = &self.pages[page] {
                    let offset = address & 0x3;
                    let word = page[(address & 0xFFFC) >> 2].to_be_bytes();

                    Short::from_be_bytes([word[offset], word[offset + 1]])
                } else {
                    0
                }
            }
            // crosses a word boundary, but not a page boundary
            (false, true) => {
                let page = (address & 0xFFFF_0000) >> 16;

                if let Some(page) = &self.pages[page as usize] {
                    let word = ((address & 0xFFFC) >> 2) as usize;

                    Short::from_be_bytes([
                        page[word].to_be_bytes()[3],
                        page[word + 1].to_be_bytes()[0],
                    ])
                } else {
                    0
                }
            }
            // crosses a page boundary
            (false, false) => {
                let page = (address & 0xFFFF_0000) >> 16;
                let mut bytes = [0; 2];

                bytes[0] = if let Some(page) = &self.pages[page as usize] {
                    page[0x3FFF].to_be_bytes()[3]
                } else {
                    0
                };
                bytes[1] = if let Some(page) = &self.pages[page as usize + 1] {
                    page[0].to_be_bytes()[0]
                } else {
                    0
                };

                Short::from_be_bytes(bytes)
            }

            _ => unreachable!(),
        }
    }

    pub fn read_word(&self, address: Word) -> Word {
        let address = address as usize % (self.pages.len() << 16);
        match (address & 0x3 == 0, address & 0xFFFF < 0xFFFD) {
            // is a word
            (true, true) => {
                let page = (address & 0xFFFF_0000) >> 16;

                if let Some(page) = &self.pages[page] {
                    // No need to convert -- value is already in native-endian encoding
                    page[(address & 0xFFFC) >> 2]
                } else {
                    0
                }
            }
            // crosses a word boundary, but not a page boundary
            (false, true) => {
                let page = (address & 0xFFFF_0000) >> 16;

                if let Some(page) = &self.pages[page] {
                    let word = (address & 0xFFFC) >> 2;
                    let bytes: [u8; 8] = unsafe {
                        transmute([page[word].to_be_bytes(), page[word + 1].to_be_bytes()])
                    };

                    let off = address & 0x3;

                    Word::from_be_bytes([
                        bytes[off],
                        bytes[off + 1],
                        bytes[off + 2],
                        bytes[off + 3],
                    ])
                } else {
                    0
                }
            }
            // crosses a page boundary
            (false, false) => {
                let page = (address & 0xFFFF_0000) >> 16;
                let off = address & 0x3;
                let mut words = [[0; 4]; 2];

                if let Some(page) = &self.pages[page] {
                    words[0] = page[0x3FFF].to_be_bytes();
                }

                if let Some(page) = &self.pages[page + 1] {
                    words[1] = page[0].to_be_bytes()
                }

                let words: [u8; 8] = unsafe { transmute(words) };

                Word::from_be_bytes([words[off], words[off + 1], words[off + 2], words[off + 3]])
            }

            _ => unreachable!(),
        }
    }

    pub fn write_byte(&mut self, address: Word, value: Byte) {
        let address = address as usize % (self.pages.len() << 16);

        let page = (address & 0xFFFF_0000) >> 16;
        let word = (address & 0xFFFC) >> 2;
        let offset = address & 0x3;

        if let Some(page) = &mut self.pages[page] {
            let mut o = page[word].to_be_bytes();

            o[offset] = value;

            page[word] = Word::from_be_bytes(o);
        } else {
            let mut alloc = allocate_page();
            let mut bytes = [0; 4];
            bytes[offset] = value;

            alloc[word] = Word::from_be_bytes(bytes);

            self.pages[page] = Some(alloc)
        }
    }

    pub fn write_short(&mut self, address: Word, value: Short) {
        let address = address as usize % (self.pages.len() << 16);
        let page = (address & 0xFFFF_0000) >> 16;
        let word = (address & 0x0000_FFFC) >> 2;
        let offset = 8 * (address & 0x3);
        let v = value.to_be_bytes();

        match (address & 0x3 != 0x3, address & 0xFFFF != 0xFFFF) {
            // neither crosses a word boundary nor crosses a page boundary
            (true, true) => {
                if let Some(page) = &mut self.pages[page] {
                    let mut o = page[word].to_be_bytes();

                    o[offset] = v[0];
                    o[offset + 1] = v[1];

                    page[word] = Word::from_be_bytes(o);
                } else {
                    let mut alloc = allocate_page();
                    let mut bytes = [0; 4];
                    bytes[offset] = v[0];
                    bytes[offset + 1] = v[1];

                    alloc[word] = Word::from_be_bytes(bytes);

                    self.pages[page] = Some(alloc);
                }
            }
            // crosses a word boundary, but not a page boundary
            (false, true) => {
                if let Some(page) = &mut self.pages[page] {
                    let o = page[word].to_be_bytes();
                    let p = page[word + 1].to_be_bytes();

                    page[word] = Word::from_be_bytes([o[0], o[1], o[2], v[0]]);
                    page[word + 1] = Word::from_be_bytes([v[1], p[1], p[2], p[3]]);
                } else {
                    let mut alloc = allocate_page();

                    alloc[word] = Word::from_be_bytes([0, 0, 0, v[0]]);
                    alloc[word + 1] = Word::from_be_bytes([v[1], 0, 0, 0]);

                    self.pages[page] = Some(alloc);
                }
            }
            // crosses a page boundary
            (false, false) => {
                if let Some(page) = &mut self.pages[page] {
                    let mut o = page[0x3FFF].to_be_bytes();
                    o[3] = v[0];

                    page[0x3FFF] = Word::from_be_bytes(o);
                } else {
                    let mut alloc = allocate_page();

                    alloc[0x3FFF] = Word::from_be_bytes([0, 0, 0, v[0]]);

                    self.pages[page] = Some(alloc);
                }
                if let Some(page) = &mut self.pages[page + 1] {
                    let mut o = page[0].to_be_bytes();
                    o[0] = v[1];

                    page[0] = Word::from_be_bytes(o);
                } else {
                    let mut alloc = allocate_page();

                    alloc[0] = Word::from_be_bytes([v[1], 0, 0, 0]);

                    self.pages[page + 1] = Some(alloc);
                }
            }

            _ => unreachable!(),
        }
    }

    pub fn write_word(&mut self, address: Word, value: Word) {
        let address = address as usize % (self.pages.len() << 16);
        let page = (address & 0xFFFF_0000) >> 16;
        let word = (address & 0xFFFC) >> 2;
        let v = value.to_be_bytes();

        match (address & 0x3 == 0, address & 0xFFFF < 0xFFFD) {
            // is a word
            (true, true) => {
                if let Some(page) = &mut self.pages[page] {
                    // No need to convert -- value is already in native-endian encoding
                    page[word] = value;
                } else {
                    let mut alloc = allocate_page();

                    alloc[word] = value;

                    self.pages[page] = Some(alloc);
                }
            }
            // crosses a word boundary, but not a page boundary
            (false, true) => {
                if let Some(page) = &mut self.pages[page] {
                    let mut bytes: [u8; 8] = unsafe {
                        transmute([page[word].to_be_bytes(), page[word + 1].to_be_bytes()])
                    };

                    let off = address & 0x3;

                    bytes[off] = v[0];
                    bytes[off + 1] = v[1];
                    bytes[off + 2] = v[2];
                    bytes[off + 3] = v[3];

                    let bytes: [[u8; 4]; 2] = unsafe { transmute(bytes) };

                    page[word] = Word::from_be_bytes(bytes[0]);
                    page[word + 1] = Word::from_be_bytes(bytes[1]);
                } else {
                    let mut alloc = allocate_page();

                    let mut bytes: [u8; 8] = [0; 8];

                    let off = address & 0x3;

                    bytes[off] = v[0];
                    bytes[off + 1] = v[1];
                    bytes[off + 2] = v[2];
                    bytes[off + 3] = v[3];

                    let bytes: [[u8; 4]; 2] = unsafe { transmute(bytes) };

                    alloc[word] = Word::from_be_bytes(bytes[0]);
                    alloc[word + 1] = Word::from_be_bytes(bytes[1]);

                    self.pages[page] = Some(alloc);
                }
            }
            // crosses a page boundary
            (false, false) => {
                let off = address & 0x3;

                let mut first =
                    take(&mut self.pages[page]).unwrap_or_else(|| allocate_page());

                let mut second =
                    take(&mut self.pages[page + 1]).unwrap_or_else(|| allocate_page());

                let mut bytes: [u8; 8] =
                    unsafe { transmute([first[0x3FFF].to_be_bytes(), second[0].to_be_bytes()]) };

                bytes[off] = v[0];
                bytes[off + 1] = v[1];
                bytes[off + 2] = v[2];
                bytes[off + 3] = v[3];

                let bytes: [[u8; 4]; 2] = unsafe { transmute(bytes) };

                first[0x3FFF] = Word::from_be_bytes(bytes[0]);
                second[0] = Word::from_be_bytes(bytes[1]);

                self.pages[page] = Some(first);
                self.pages[page + 1] = Some(second);
            }

            _ => unreachable!(),
        }
    }

    pub fn read_words(&self, address: Word, amount: usize) -> Box<[u8]> {
        (address..(address + amount as Word))
            .map(|a| self.read_byte(a))
            .collect()
    }

    pub fn erase(&mut self) {
        for page in self.pages.iter_mut() {
            *page = None;
        }
    }
}
