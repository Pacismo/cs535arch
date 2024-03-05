use libseis::types::*;

#[derive(Debug)]
pub enum Data {
    Byte(Vec<Byte>),
    Short(Vec<Short>),
    Word(Vec<Word>),
    Float(Vec<f32>),
    String(Vec<String>),
}
