use crate::types::Word;
use std::{error::Error, fmt::Display};

#[derive(Debug, Clone)]
pub enum DecodeError {
    InvalidOpType(Word),

    InvalidControlOp(Word),

    InvalidIntegerOp(Word),
    InvalidFloatingPointOp(Word),

    InvalidRegisterOp(Word),
    InvalidAddressingMode(Word),
    InvalidPushOp(Word),
    InvalidPopOp(Word),
}

impl Error for DecodeError {}

impl Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use DecodeError::*;

        match self {
            &InvalidOpType(word) => write!(f, "Could not decode optype {word:#x}"),

            &InvalidControlOp(word) => write!(f, "Could not decode control op type {word:#x}"),

            &InvalidIntegerOp(word) => write!(f, "Could not decode integer op type {word:#x}"),

            &InvalidFloatingPointOp(word) => {
                write!(f, "Could not decode floating-point op type {word:#x}")
            }

            &InvalidRegisterOp(word) => write!(f, "Could not decode register op type {word:#x}"),
            &InvalidAddressingMode(word) => write!(f, "Could not decode addressing mode {word:#x}"),
            &InvalidPushOp(word) => write!(f, "Could not decode push op type {word:#x}"),
            &InvalidPopOp(word) => write!(f, "Could not decode pop op type {word:#x}"),
        }
    }
}

pub type DecodeResult<T> = std::result::Result<T, DecodeError>;
