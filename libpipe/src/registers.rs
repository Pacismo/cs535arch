use std::mem::transmute;

use libseis::{
    registers::COUNT,
    types::{Register, Word},
};
use serde::Serialize;

#[repr(C)]
#[derive(Serialize, Debug)]
pub struct Registers {
    v: [Word; 16],
    sp: Word,
    bp: Word,
    lp: Word,
    pc: Word,
    zf: Word,
    of: Word,
    eps: Word,
    nan: Word,
    inf: Word,
}

impl Registers {
    pub fn reg(&self, id: Register) -> Word {
        let id = id as usize;

        if id < COUNT {
            let regs: &[Word; COUNT] = unsafe { transmute(self) };
            regs[id]
        } else {
            0
        }
    }

    pub fn set_reg(&mut self, id: Register, value: Word) {
        let id = id as usize;

        if id < COUNT {
            let regs: &mut [Word; COUNT] = unsafe { transmute(self) };
            regs[id] = value;
        }
    }
}
