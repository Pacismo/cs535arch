use crate::parse::{ConstantValue, Span};

#[derive(Debug)]
pub struct Constant {
    pub span: Span,
    pub value: ConstantValue,
    pub bits: usize,
}
