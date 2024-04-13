//! A library containing definitions and code for the memory
//! unit of the simulated processor.
//!
//! [`cache`] contains the [`Cache`](cache::Cache) trait and
//! datastructures implementing the trait.
//!
//! [`memory`] contains the [`Memory`](memory::Memory) datastructure.
//!
//! [`module`] contains the [`Module`](module::MemoryModule) trait and
//! datastructures implementing the trait.

#![warn(missing_docs)]

pub mod cache;
pub mod memory;
pub mod module;
