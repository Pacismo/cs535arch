use crate::types::Word;

pub const PAGE_SIZE: usize = 2usize.pow(16);
pub const STACK_PAGE: Word = 0x0001_0000;
pub const ZERO_PAGE: Word = 0x0002_0000;
