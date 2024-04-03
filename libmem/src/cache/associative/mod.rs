mod line;
mod multi;
mod single;

use libseis::types::Word;
pub use multi::MultiAssociative;
pub use single::Associative;

fn get_masks(set_bits: usize, off_bits: usize) -> (Word, Word, Word) {
    (
        ((1 << (32 - (set_bits + off_bits))) - 1) as Word,
        ((1 << set_bits) - 1) as Word,
        ((1 << off_bits) - 1) as Word,
    )
}

fn construct_address(tag: Word, set: Word, off: Word, set_bits: usize, off_bits: usize) -> Word {
    let (tag_mask, set_mask, off_mask) = get_masks(set_bits, off_bits);

    ((tag & tag_mask) << (off_bits + set_bits)) | ((set & set_mask) << off_bits) | (off & off_mask)
}

fn split_address(address: Word, set_bits: usize, off_bits: usize) -> (Word, usize, usize) {
    let (tag_mask, set_mask, off_mask) = get_masks(set_bits, off_bits);

    let tag = (address >> (off_bits + set_bits)) & tag_mask;
    let set = (address >> off_bits) & set_mask;
    let off = address & off_mask;

    (tag, set as usize, off as usize)
}
