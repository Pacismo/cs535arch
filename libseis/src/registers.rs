use serde::Serialize;

use crate::types::{Register, Word};
use std::fmt::{Display, Write};

pub const V: [Register; 16] = [
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
];

pub const SP: Register = 0x10;
pub const BP: Register = 0x11;
pub const LP: Register = 0x12;
pub const PC: Register = 0x13;
pub const ZF: Register = 0x14;
pub const OF: Register = 0x14;
pub const EPS: Register = 0x15;
pub const NAN: Register = 0x16;
pub const INF: Register = 0x17;

pub const COUNT: usize = 25;

pub const NAME: [&'static str; COUNT] = [
    "V0", "V1", "V2", "V3", "V4", "V5", "V6", "V7", "V8", "V9", "VA", "VB", "VC", "VD", "VE", "VF",
    "SP", "BP", "LP", "PC", "ZF", "OF", "EPS", "NAN", "INF",
];

pub const fn get_name(reg: Register) -> Option<&'static str> {
    if (reg as usize) >= COUNT {
        None
    } else {
        Some(NAME[reg as usize])
    }
}

pub fn get_id(name: &str) -> Option<Register> {
    let target = name.to_uppercase();

    NAME.into_iter()
        .enumerate()
        .find(|&(_, s)| s == target)
        .map(|(i, _)| i as Register)
}

#[derive(Debug, Clone)]
pub struct RegFlagIterator(Word, Register);

impl Iterator for RegFlagIterator {
    type Item = Register;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.1 >= COUNT as Register {
                break None;
            } else if self.0 & (1 << self.1) != 0 {
                let temp = self.1;
                self.1 += 1;
                break Some(temp);
            } else {
                self.1 += 1;
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // We only want to count the bits within the range between COUNT and self.1
        let count = (self.0 & ((1 << COUNT) - (1 << self.1))).count_ones() as usize;
        (count, Some(count))
    }
}

impl ExactSizeIterator for RegFlagIterator {}

#[derive(Debug, Clone, Copy, Default)]
pub struct RegisterFlags(pub Word);

impl From<Register> for RegisterFlags {
    fn from(value: Register) -> Self {
        Self(1 << value)
    }
}

impl IntoIterator for RegisterFlags {
    type Item = Register;

    type IntoIter = RegFlagIterator;

    fn into_iter(self) -> Self::IntoIter {
        RegFlagIterator(self.0, 0)
    }
}

impl RegisterFlags {
    pub fn has_register(self, reg_id: Register) -> bool {
        self.0 & (1 << reg_id as Word) != 0
    }

    pub fn registers(self) -> RegFlagIterator {
        self.into_iter()
    }
}

impl FromIterator<Register> for RegisterFlags {
    fn from_iter<T: IntoIterator<Item = Register>>(iter: T) -> Self {
        let mut flags = 0;
        for reg in iter {
            assert!(reg <= LP);
            flags |= 1 << reg as Word;
        }
        Self(flags)
    }
}

impl Display for RegisterFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut string = "".to_owned();

        for v in V.into_iter().filter(|&b| self.has_register(b)) {
            if string.is_empty() {
                write!(string, "V{v:X}")?;
            } else {
                write!(string, ", V{v:X}")?;
            }
        }

        if self.has_register(SP) {
            if string.is_empty() {
                write!(string, "SP")?;
            } else {
                write!(string, ", SP")?;
            }
        }

        if self.has_register(BP) {
            if string.is_empty() {
                write!(string, "BP")?;
            } else {
                write!(string, ", BP")?;
            }
        }

        if self.has_register(LP) {
            if string.is_empty() {
                write!(string, "LP")?;
            } else {
                write!(string, ", LP")?;
            }
        }

        write!(f, "{{{string}}}")
    }
}

impl Serialize for RegisterFlags {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_seq(self.registers())
    }
}
