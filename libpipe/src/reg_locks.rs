use std::{
    fmt::Debug,
    ops::{Deref, DerefMut, Index, IndexMut},
};

use libseis::{registers::COUNT, types::Register};
use serde::Serialize;

#[repr(C)]
#[derive(Serialize, Debug, Clone, Copy)]
pub struct Named {
    pub v: [bool; 16],
    pub sp: bool,
    pub bp: bool,
    pub lp: bool,
    pub pc: bool,
    pub zf: bool,
    pub of: bool,
    pub eps: bool,
    pub nan: bool,
    pub inf: bool,
}

type Indexed = [bool; COUNT];

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
    type Output = bool;

    fn index(&self, index: Register) -> &Self::Output {
        unsafe { &self.by_id[index as usize] }
    }
}

impl IndexMut<Register> for Locks {
    fn index_mut(&mut self, index: Register) -> &mut Self::Output {
        unsafe { &mut self.by_id[index as usize] }
    }
}
