use libseis::{
    registers::get_name,
    types::{Register, Word},
};
use serde::Serialize;
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, Copy, Serialize)]
pub struct RegMapPair {
    pub register: Register,
    pub value: Word,
}

/// Simple, low-budget map that uses a binary search to find a desired value.
#[derive(Debug, Clone, Default)]
pub struct RegMap(Vec<RegMapPair>);

impl Serialize for RegMap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_map(
            self.0.iter().filter_map(|&RegMapPair { register, value }| {
                get_name(register).map(|s| (s, value))
            }),
        )
    }
}

impl IntoIterator for RegMap {
    type Item = RegMapPair;

    type IntoIter = std::vec::IntoIter<RegMapPair>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Index<Register> for RegMap {
    type Output = Word;

    fn index(&self, index: Register) -> &Self::Output {
        let idx = self
            .0
            .binary_search_by_key(&index, |pair| pair.register)
            .expect(&format!(
                "Register {} not loaded!",
                libseis::registers::get_name(index).unwrap_or("<unknown>")
            ));

        &self.0[idx].value
    }
}

impl IndexMut<Register> for RegMap {
    fn index_mut(&mut self, index: Register) -> &mut Self::Output {
        let idx = self
            .0
            .binary_search_by_key(&index, |pair| pair.register)
            .expect(&format!(
                "Register {} not loaded!",
                libseis::registers::get_name(index).unwrap_or("<unknown>")
            ));

        &mut self.0[idx].value
    }
}

impl RegMap {
    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> std::slice::Iter<RegMapPair> {
        self.0.iter()
    }
}

impl FromIterator<(Register, Word)> for RegMap {
    fn from_iter<T: IntoIterator<Item = (Register, Word)>>(iter: T) -> Self {
        Self(
            iter.into_iter()
                .map(|(register, value)| RegMapPair { register, value })
                .collect(),
        )
    }
}
