use crate::parse::Span;
use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    ExistingLabel {
        name: String,
        first: Span,
        repeat: Span,
    },
    NonExistingLabel {
        name: String,
        usage: Span,
    },
    ExistingConstant {
        name: String,
        first: Span,
        repeat: Span,
    },
    NonExistingConstant {
        name: String,
        usage: Span,
    },
    WritingToZeroPage {
        span: Span,
    },
    WritingToStack {
        span: Span,
    },
    JumpTooLong {
        label: String,
        span: Span,
    },
    IntTypeMismatch {
        name: String,
        span: Span,
    },
    ConstTooLong {
        name: String,
        span: Span,
    },
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;

        match self {
            ExistingLabel {
                name,
                first,
                repeat,
            } => write!(
                f,
                "Label {name} has already been defined at {first} and got redefined at {repeat}"
            ),
            NonExistingLabel { name, usage } => write!(
                f,
                "Label {name} has not been defined, but is used at {usage}"
            ),
            ExistingConstant {
                name,
                first,
                repeat,
            } => write!(
                f,
                "Constant {name} has already been defined at {first} and got redefined at {repeat}"
            ),
            NonExistingConstant { name, usage } => write!(
                f,
                "Constant {name} has not been defined, but is used at {usage}"
            ),
            WritingToZeroPage { span } => {
                write!(f, "Attempting to write code or data to the zero page at {span}\nPlease write to a different page")
            }
            WritingToStack { span } => {
                write!(f, "Attempting to write code or data to the stack page at {span}\nPlease write to a different page")
            }
            JumpTooLong { label, span } => write!(f, "Label {label} is too far away from the instruction at {span} to jump to\nLoad the label to a register and do an absolute jump"),
            IntTypeMismatch { name, span } => write!(f, "Tried to load {name} as an integer at {span}"),
            ConstTooLong { name, span } => write!(f, "The constant {name} at {span} is too long to be put in the immediate field")
        }
    }
}
