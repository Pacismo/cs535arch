use std::{
    fmt::Debug,
    ops::{Deref, DerefMut, Index, IndexMut},
};

use libseis::{registers::COUNT, types::Register};
use serde::Serialize;

#[repr(C)]
#[derive(Serialize, Debug, Clone, Copy)]
pub struct Named {
    pub v: [u8; 16],
    pub sp: u8,
    pub bp: u8,
    pub lp: u8,
    pub pc: u8,
    pub zf: u8,
    pub of: u8,
    pub eps: u8,
    pub nan: u8,
    pub inf: u8,
}

type Indexed = [u8; COUNT];

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
    pub fn is_locked(&self, reg: Register) -> bool {
        self[reg] != 0
    }
}
