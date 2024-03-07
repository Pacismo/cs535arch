use libseis::{
    pages::PAGE_SIZE,
    types::{Byte, Short, Word},
};

type Page = [Byte; PAGE_SIZE];

/// Memory representation that only allocates pages when they're written to.
///
/// Cannot add new pages after construction -- addressing out of bounds will lead to a [`panic`].
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
        Self {
            pages: (0..count).map(|_| None).collect(),
        }
    }

    pub fn read_byte(&self, address: Word) -> Byte {
        let page = (address & 0xFFFF_0000) >> 16;
        if let Some(page) = &self.pages[page as usize] {
            page[(address & 0x0000_FFFF) as usize]
        } else {
            0
        }
    }

    pub fn read_short(&self, address: Word) -> Short {
        Short::from_be_bytes([self.read_byte(address), self.read_byte(address + 1)])
    }

    pub fn read_word(&self, address: Word) -> Word {
        Word::from_be_bytes([
            self.read_byte(address),
            self.read_byte(address + 1),
            self.read_byte(address + 2),
            self.read_byte(address + 3),
        ])
    }

    pub fn write_byte(&mut self, address: Word, value: Byte) {
        let page_addr = (address & 0xFFFF_0000) >> 16;

        if let Some(page) = self.pages[page_addr as usize].as_mut() {
            page[(address & 0x0000_FFFF) as usize] = value;
        } else {
            let mut page = Box::new([0; PAGE_SIZE]);

            page[(address & 0x0000_FFFF) as usize] = value;

            self.pages[page_addr as usize] = Some(page);
        }
    }

    pub fn write_short(&mut self, address: Word, value: Short) {
        value
            .to_be_bytes()
            .into_iter()
            .zip(address..)
            .for_each(|(v, a)| self.write_byte(a, v));
    }

    pub fn write_word(&mut self, address: Word, value: Word) {
        value
            .to_be_bytes()
            .into_iter()
            .zip(address..)
            .for_each(|(v, a)| self.write_byte(a, v));
    }

    pub fn read_bytes(&self, address: Word, amount: usize) -> Box<[u8]> {
        (address..(address + amount as Word))
            .map(|a| self.read_byte(a))
            .collect()
    }
}
