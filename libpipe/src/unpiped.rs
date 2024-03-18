//! This module contains the implementation of a non-pipelined processor.
//!
//! The [`Pipeline`] trait simply enables consistent interfacing.

use crate::{Pipeline, Registers};
use libmem::{cache::Cache, memory::Memory};
use serde::Serialize;

/// Represents an unpipelined processor
#[derive(Debug, Serialize)]
pub struct Unpipelined<'a> {
    memory: Memory,
    registers: Registers,
    cache: Box<dyn Cache<'a>>,
}

impl<'a> Pipeline<'a> for Unpipelined<'a> {
    fn clock(&mut self) -> crate::ClockResult {
        todo!()
    }

    fn memory_module(&self) -> &dyn libmem::module::MemoryModule {
        todo!()
    }

    fn registers(&self) -> &crate::Registers {
        todo!()
    }
}
