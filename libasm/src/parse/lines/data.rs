use libseis::types::*;

#[derive(Debug)]
pub enum Data {
    Byte(Vec<Byte>),
    Short(Vec<Short>),
    Word(Vec<Word>),
    Float(Vec<f32>),
    String(Vec<String>),
}

#[derive(Debug)]
pub enum RandomData {
    Byte(Byte, Byte, usize, Option<u64>),
    Short(Short, Short, usize, Option<u64>),
    Word(Word, Word, usize, Option<u64>),
    Float(f32, f32, usize, Option<u64>),
}
