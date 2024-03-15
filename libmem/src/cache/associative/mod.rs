mod multi;
mod single;
mod line;

use libseis::types::Word;
pub use multi::MultiAssociative;
pub use single::Associative;


fn construct_address(tag: Word, set: Word, off: Word, set_bits: usize, off_bits: usize) -> Word {
    let set_mask = (1 << set_bits) - 1;
    let tag_mask = (1 << 32 - (set_bits + off_bits)) - 1;
    let off_mask = (1 << off_bits) - 1;

    ((tag & tag_mask) << (off_bits + set_bits)) | ((set & set_mask) << off_bits) | (off & off_mask)
}

fn split_address(address: Word, set_bits: usize, off_bits: usize) -> (Word, usize, usize) {
    let set_mask = (1 << set_bits) - 1;
    let tag_mask = (1 << 32 - (set_bits + off_bits)) - 1;
    let off_mask = (1 << off_bits) - 1;

    let tag = address >> off_bits + set_bits & tag_mask;
    let set = address >> off_bits & set_mask;
    let off = address & off_mask;

    (tag, set as usize, off as usize)
}
