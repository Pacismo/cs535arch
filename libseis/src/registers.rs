use serde::Serialize;

use crate::types::{Register, Word};
use std::{
    fmt::{Display, Write},
    ops::{BitOr, BitOrAssign},
};

pub const V: [Register; 16] = [
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
];

pub const SP: Register = 0x10;
pub const BP: Register = 0x11;
pub const LP: Register = 0x12;
pub const PC: Register = 0x13;
pub const ZF: Register = 0x14;
pub const OF: Register = 0x15;
pub const EPS: Register = 0x16;
pub const NAN: Register = 0x17;
pub const INF: Register = 0x18;

pub const COUNT: usize = (INF as usize) + 1;

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

impl RegFlagIterator {
    pub fn to_vec(self) -> Vec<Register> {
        self.collect()
    }
}

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

    pub fn to_vec(self) -> Vec<Register> {
        self.into_iter().collect()
    }

    pub fn count(&self) -> usize {
        self.into_iter().count()
    }
}

impl FromIterator<Register> for RegisterFlags {
    fn from_iter<T: IntoIterator<Item = Register>>(iter: T) -> Self {
        let mut flags = 0;
        for reg in iter {
            if reg < COUNT as Register {
                flags |= 1 << reg as Word;
            }
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
        serializer.collect_seq(self.registers().map(|r| get_name(r).unwrap_or("<?>")))
    }
}

impl BitOr<Register> for RegisterFlags {
    type Output = RegisterFlags;

    fn bitor(self, rhs: Register) -> Self::Output {
        assert!(rhs < COUNT as u8);
        Self(self.0 | (1 << rhs))
    }
}

impl BitOr for RegisterFlags {
    type Output = RegisterFlags;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign<Register> for RegisterFlags {
    fn bitor_assign(&mut self, rhs: Register) {
        assert!(rhs < COUNT as u8);
        self.0 |= 1 << rhs;
    }
}

impl BitOrAssign for RegisterFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl<const COUNT: usize> From<[Register; COUNT]> for RegisterFlags {
    fn from(regs: [Register; COUNT]) -> Self {
        regs.into_iter().collect()
    }
}
