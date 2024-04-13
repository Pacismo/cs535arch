//! Register locking

use std::{
    fmt::Debug,
    ops::{Deref, DerefMut, Index, IndexMut},
};
use libseis::{registers::COUNT, types::Register};
use serde::Serialize;

/// Registers by name
#[repr(C)]
#[derive(Serialize, Debug, Clone, Copy)]
pub struct Named {
    /// Variable registers
    pub v: [u8; 16],
    /// Stack pointer
    pub sp: u8,
    /// Stack base pointer
    pub bp: u8,
    /// Link pointer
    pub lp: u8,
    /// Program counter
    pub pc: u8,
    /// Zero flag
    pub zf: u8,
    /// Overflow flag
    pub of: u8,
    /// Epsilon equality flag
    pub eps: u8,
    /// NaN flag
    pub nan: u8,
    /// Infinity flag
    pub inf: u8,
}

/// Register indexing by ID
type Indexed = [u8; COUNT];

/// Represents the locks on the processor's registers
#[repr(C)]
pub union Locks {
    by_name: Named,
    by_id: Indexed,
}

impl Default for Locks {
    fn default() -> Self {
        Self {
            by_id: Indexed::default(),
        }
    }
}

impl Debug for Locks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe { Debug::fmt(&self.by_name, f) }
    }
}

impl Deref for Locks {
    type Target = Named;

    fn deref(&self) -> &Self::Target {
        unsafe { &self.by_name }
    }
}

impl DerefMut for Locks {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut self.by_name }
    }
}

impl Index<Register> for Locks {
    type Output = u8;

    fn index(&self, index: Register) -> &Self::Output {
        unsafe { &self.by_id[index as usize] }
    }
}

impl IndexMut<Register> for Locks {
    fn index_mut(&mut self, index: Register) -> &mut Self::Output {
        unsafe { &mut self.by_id[index as usize] }
    }
}

impl Locks {
    /// Returns true if the register lock count is not zero
    pub fn is_locked(&self, reg: Register) -> bool {
        !self.is_unlocked(reg)
    }

    /// Returns true if the register lock count is zero
    pub fn is_unlocked(&self, reg: Register) -> bool {
        self[reg] == 0
    }
}
