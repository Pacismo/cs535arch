//! The memory module, [`Memory`], for use with the memory module.
//!
//! The datastructure in this module contains a set of dynamically-allocated pages,
//! which are allocated on write.

use std::{
    iter::{Enumerate, FlatMap, Map},
    mem::take,
    ops::Deref,
    slice::Iter,
};
use libseis::{
    pages::PAGE_SIZE,
    types::{Byte, Short, Word},
};
use serde::{ser::SerializeSeq, Serialize};

type Page = [Byte; PAGE_SIZE];
/// An iterator over the pages of the [`Memory`] datastructure
pub type PageIterator<'a> =
Map<Iter<'a, Option<Box<Page>>>, fn(&'a Option<Box<Page>>) -> Option<&'a [u8]>>;
/// An iterator over the allocated pages of the [`Memory`] datastructure
pub type AllocatedPageIterator<'a> = FlatMap<
    Enumerate<PageIterator<'a>>,
    Option<(usize, &'a [u8])>,
    fn((usize, Option<&'a [u8]>)) -> Option<(usize, &'a [u8])>,
>;

fn allocate_page() -> Box<Page> {
    Box::new([0; PAGE_SIZE])
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
        let byte = address & 0xFFFF;

        if let Some(page) = &self.pages[page] {
            page[byte]
        } else {
            0
        }
    }

    pub fn read_short(&self, address: Word) -> Short {
        let address = address as usize % (self.pages.len() << 16);

        match address & 0xFFFF != 0xFFFF {
            // does not cross a page boundary
            true => {
                let page = (address & 0xFFFF_0000) >> 16;
                let byte = address & 0xFFFF;

                if let Some(page) = &self.pages[page] {
                    let mut bytes = [0; 2];
                    bytes.copy_from_slice(&page[byte..]);
                    Short::from_be_bytes(bytes)
                } else {
                    0
                }
            }
            false => {
                let page = (address & 0xFFFF_0000) >> 16;
                let mut bytes = [0; 2];

                bytes[0] = if let Some(page) = &self.pages[page as usize] {
                    page[0xFFFF]
                } else {
                    0
                };
                bytes[1] = if let Some(page) = &self.pages[page as usize + 1] {
                    page[0]
                } else {
                    0
                };

                Short::from_be_bytes(bytes)
            }
        }
    }

    pub fn read_word(&self, address: Word) -> Word {
        let address = address as usize % (self.pages.len() << 16);
        match address & 0xFFFF < 0xFFFD {
            // is a word
            true => {
                let page = (address & 0xFFFF_0000) >> 16;
                let byte = address & 0xFFFF;

                if let Some(page) = &self.pages[page] {
                    let mut bytes = [0; 4];
                    bytes.copy_from_slice(&page[byte..]);
                    Word::from_be_bytes(bytes)
                } else {
                    0
                }
            }
            // crosses a page boundary
            false => {
                let page = (address & 0xFFFF_0000) >> 16;
                let byte = address & 0xFFFF;
                let mut bytes = [0; 4];

                if let Some(page) = &self.pages[page] {
                    bytes[..byte & 0x3].copy_from_slice(&page[byte..]);
                }
                if let Some(page) = &self.pages[page + 1] {
                    bytes[byte & 0x3..].copy_from_slice(&page[..byte & 3]);
                }

                Word::from_be_bytes(bytes)
            }
        }
    }

    pub fn write_byte(&mut self, address: Word, value: Byte) {
        let address = address as usize % (self.pages.len() << 16);

        let page = (address & 0xFFFF_0000) >> 16;
        let byte = address & 0xFFFF;

        if let Some(page) = &mut self.pages[page] {
            page[byte] = value;
        } else {
            let mut alloc = allocate_page();

            alloc[byte] = value;
            self.pages[page] = Some(alloc)
        }
    }

    pub fn write_short(&mut self, address: Word, value: Short) {
        let address = address as usize % (self.pages.len() << 16);
        let page = (address & 0xFFFF_0000) >> 16;
        let byte = address & 0xFFFF;
        let v = value.to_be_bytes();

        match address & 0xFFFF != 0xFFFF {
            // does not cross a page boundary
            true => {
                if let Some(page) = &mut self.pages[page] {
                    page[byte] = v[0];
                    page[byte + 1] = v[1];
                } else {
                    let mut alloc = allocate_page();
                    alloc[byte] = v[0];
                    alloc[byte + 1] = v[1];
                    self.pages[page] = Some(alloc);
                }
            }
            // crosses a page boundary
            false => {
                let mut first = take(&mut self.pages[page]).unwrap_or_else(allocate_page);
                let mut second = take(&mut self.pages[page + 1]).unwrap_or_else(allocate_page);

                first[byte] = v[0];
                second[0] = v[1];

                self.pages[page] = Some(first);
                self.pages[page + 1] = Some(second);
            }
        }
    }

    pub fn write_word(&mut self, address: Word, value: Word) {
        let address = address as usize % (self.pages.len() << 16);
        let page = (address & 0xFFFF_0000) >> 16;
        let byte = address & 0xFFFF;
        let v = value.to_be_bytes();

        match address & 0xFFFF < 0xFFFD {
            // is a word
            true => {
                if let Some(page) = &mut self.pages[page] {
                    page[byte..byte + 4].copy_from_slice(&v);
                } else {
                    let mut alloc = allocate_page();
                    alloc[byte..byte + 4].copy_from_slice(&v);
                    self.pages[page] = Some(alloc);
                }
            }
            // crosses a page boundary
            false => {
                let mut first = take(&mut self.pages[page]).unwrap_or_else(allocate_page);
                let mut second = take(&mut self.pages[page + 1]).unwrap_or_else(allocate_page);

                first[byte..].copy_from_slice(&v[..byte & 3]);
                second[..byte & 3].copy_from_slice(&v[byte & 3..]);

                self.pages[page] = Some(first);
                self.pages[page + 1] = Some(second);
            }
        }
    }

    pub fn read_words(&self, address: Word, amount: usize) -> Box<[u8]> {
        let mut data = vec![0; amount].into_boxed_slice();
        self.read_words_to(address, &mut data);
        data
    }

    pub fn read_words_to(&self, address: Word, to: &mut [u8]) {
        (address..(address + to.len() as Word))
            .enumerate()
            .for_each(|(i, a)| to[i] = self.read_byte(a))
    }

    pub fn erase(&mut self) {
        for page in self.pages.iter_mut() {
            *page = None;
        }
    }

    pub fn pages(&self) -> PageIterator {
        self.pages
            .iter()
            .map(|p| p.as_ref().map(|p| p.deref().as_ref()))
    }

    pub fn allocated_pages(&self) -> AllocatedPageIterator {
        self.pages()
            .enumerate()
            .flat_map(|(i, p)| p.map(|p| (i, p)))
    }
}

impl Serialize for Memory {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.pages.len()))?;

        for page in self.pages() {
            seq.serialize_element(&page)?;
        }

        seq.end()
    }
}
