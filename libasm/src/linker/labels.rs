use libseis::types::Word;
use crate::parse::Span;

#[derive(Debug)]
pub struct Label {
    pub address: Word,
    pub span: Span,
}
