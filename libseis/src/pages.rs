//! Represents constants related to how pages work in the architecture
use crate::types::Word;

/// The size of a page
pub const PAGE_SIZE: usize = 2usize.pow(16);
/// The offset of the stack page
pub const STACK_PAGE: Word = 0x0001_0000;
/// The offset of the zero (short) page
pub const ZERO_PAGE: Word = 0x0002_0000;
