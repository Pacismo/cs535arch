//! Decoding errors

use crate::{
    registers,
    types::{Register, Word},
};
use std::{error::Error, fmt::Display};

/// An enumerator representing the kinds of errors that may
/// happen in decoding
#[derive(Debug, Clone)]
pub enum DecodeError {
    /// The operation type was invalid
    InvalidOpType(Word),

    /// The control operation was invalid
    InvalidControlOp(Word),

    /// The integer operation was invalid
    InvalidIntegerOp(Word),
    /// The floating-point operation was invalid
    InvalidFloatingPointOp(Word),

    /// The register operation was invalid
    InvalidRegisterOp(Word),
    /// Invalid addressing mode
    InvalidAddressingMode(Word),
    /// Invalid push operation
    InvalidPushOp(Word),
    /// Invalid pop operation
    InvalidPopOp(Word),
    /// Invalid register operation
    InvalidRegister(Register, Register),
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

            &InvalidRegister(src, dst) => {
                if (src as usize) >= registers::COUNT && (dst as usize) >= registers::COUNT {
                    write!(f, "Invalid registers: src={src}, dst={dst}")
                } else if (dst as usize) >= registers::COUNT {
                    write!(f, "Invalid register: dst={dst}")
                } else {
                    write!(f, "Invalid register: src={src}")
                }
            }
        }
    }
}

/// Type alias for a decode result
pub type DecodeResult<T> = std::result::Result<T, DecodeError>;
