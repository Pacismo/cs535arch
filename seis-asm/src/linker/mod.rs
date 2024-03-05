mod constants;
pub mod error;
mod labels;

use self::{constants::Constant, error::Error};
use crate::{
    linker::labels::Label,
    parse::{Data, Instruction, Lines, Span},
};
use libseis::{
    pages::{PAGE_SIZE, STACK_PAGE, ZERO_PAGE},
    types::{Short, Word},
};
use std::{
    collections::{HashMap, LinkedList},
    path::Path,
};

macro_rules! byte_len {
    ($container:ident) => {
        ($container.len() * std::mem::size_of_val($container.first().unwrap()))
    };
}

pub struct Page {
    data: [u8; PAGE_SIZE],
    len: Word,
}

impl Default for Page {
    fn default() -> Self {
        Self {
            data: [0; PAGE_SIZE],
            len: 0,
        }
    }
}

impl Page {
    pub fn read(&self, byte: Word) -> u8 {
        self.data[(byte & 0xFFFF) as usize]
    }

    pub fn write(&mut self, byte: Word, value: u8) {
        let addr = byte & 0xFFFF;
        if addr >= self.len {
            self.len = addr + 1;
        }
        self.data[byte as usize] = value;
    }

    pub fn bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn data(&self) -> &[u8] {
        &self.data[..self.len as usize]
    }
}

pub struct PageSet(HashMap<Short, Page>);

impl PageSet {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    fn page(&mut self, page_id: Short) -> &mut Page {
        if !self.0.contains_key(&page_id) {
            self.0.insert(page_id, Default::default());
        }

        self.0.get_mut(&page_id).unwrap()
    }

    fn page_of(&mut self, address: Word) -> &mut Page {
        let page_id = (address >> 16) as Short;
        self.page(page_id)
    }

    pub fn write<A: AsRef<Path>>(self, destination: A) -> std::io::Result<()> {
        todo!()
    }
}

pub fn link_symbols(lines: Lines) -> Result<PageSet, Error> {
    let mut pages: PageSet = PageSet::new();
    let mut constants: HashMap<String, Constant> = HashMap::new();
    let mut to_eval = Lines::new();

    for line in lines.into_iter() {
        use crate::parse::LineType as T;
        match line {
            T::Constant(value, span) => {
                if let Some(constant) = constants.get(&value.ident) {
                    return Err(Error::ExistingConstant {
                        name: value.ident,
                        first: constant.span.clone(),
                        repeat: span,
                    });
                }

                constants.insert(
                    value.ident,
                    Constant {
                        span,
                        value: value.value,
                    },
                );
            }

            x => to_eval.push_back(x),
        }
    }

    let mut labels: HashMap<String, Label> = HashMap::new();

    let mut ip = 0;

    for line in to_eval {
        use crate::parse::LineType as T;

        println!("{line:#?}");

        match line {
            T::Instruction(value, span) => todo!(),

            T::Directive(value, span) => {
                // TODO: put an assertion to ensure that the address does not go past the memory upper-bound.
                use crate::parse::Directive::*;
                match value {
                    Location(address) => {
                        if address & 0xFFFF_0000 == STACK_PAGE {
                            return Err(Error::WritingToStack { span });
                        }
                        if address & 0xFFFF_0000 == ZERO_PAGE {
                            return Err(Error::WritingToZeroPage { span });
                        }

                        if pages.page_of(address).len >= address & 0x0000_FFFF {
                            println!("At {span}:\nBy writing to address {address:#x}, you may potentially be overwriting code or data. It is recommended that you move the code or data elsewhere.")
                        }

                        ip = address;
                    }
                }
            }

            T::Label(name, span) => {
                if let Some(label) = labels.get(&name) {
                    return Err(Error::ExistingLabel {
                        name,
                        first: label.span.clone(),
                        repeat: span,
                    });
                }

                labels.insert(name, Label { address: ip, span });
            }

            T::Data(data, span) => {
                use crate::parse::Data as D;

                match data {
                    D::Byte(bytes) => todo!(),
                    D::Short(shorts) => todo!(),
                    D::Word(words) => todo!(),
                    D::Float(floats) => todo!(),
                    D::String(strings) => todo!(),
                }
            }

            _ => unreachable!("constants are all removed from the list"),
        }
    }

    Ok(pages)
}
