//! Datastructures to represent the register set utilized by the processor
//!
//! [`Named`] provides the registers by name, while [`Indexed`] provides
//! the registers by ID.
//!
//! This makes serialization and decoding trivially easy at runtime.

use libseis::{
    registers::COUNT,
    types::{Register, Word},
};
use serde::Serialize;
use std::{
    fmt::Debug,
    ops::{Deref, DerefMut, Index, IndexMut},
};

/// Named registers
#[repr(C)]
#[derive(Serialize, Debug, Clone, Copy)]
pub struct Named {
    /// Variable registers
    pub v: [Word; 16],
    /// Stack pointer
    pub sp: Word,
    /// Stack base pointer
    pub bp: Word,
    /// Link pointer
    pub lp: Word,
    /// Program counter
    pub pc: Word,
    /// Zero flag
    pub zf: Word,
    /// Overflow flag
    pub of: Word,
    /// Epsilon equality flag
    pub eps: Word,
    /// NaN flag
    pub nan: Word,
    /// Infinity flag
    pub inf: Word,
}

/// Enables ID indexing
type Indexed = [Word; COUNT];

/// Represents the registers of the processor.
#[repr(C)]
#[derive(Clone, Copy)]
pub union Registers {
    by_name: Named,
    by_id: Indexed,
}

impl Registers {
    /// Creates an iterator over the registers
    pub fn iter(&self) -> std::slice::Iter<Word> {
        unsafe { self.by_id.iter() }
    }
}

impl Default for Registers {
    fn default() -> Self {
        Self {
            by_id: Indexed::default(),
        }
    }
}

impl Debug for Registers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(unsafe { &self.by_name }, f)
    }
}

impl Deref for Registers {
    type Target = Named;

    fn deref(&self) -> &Self::Target {
        unsafe { &self.by_name }
    }
}

impl DerefMut for Registers {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut self.by_name }
    }
}

impl Index<Register> for Registers {
    type Output = Word;

    fn index(&self, index: Register) -> &Self::Output {
        unsafe { &self.by_id[index as usize] }
    }
}

impl IndexMut<Register> for Registers {
    fn index_mut(&mut self, index: Register) -> &mut Self::Output {
        unsafe { &mut self.by_id[index as usize] }
    }
}

impl Serialize for Registers {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        unsafe { self.by_name.serialize(serializer) }
    }
}
