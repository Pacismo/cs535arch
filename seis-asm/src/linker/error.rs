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
    WritingCodeToZeroPage {
        span: Span,
    },
    WritingCodeToStack {
        span: crate::parse::Span,
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
            WritingCodeToZeroPage { span } => {
                write!(f, "Attempting to write code to the zero page at {span}\nPlease move the code to a different page.")
            }
            WritingCodeToStack { span } => {
                write!(f, "Attempting to write code to the stack page at {span}\nPlease move the code to a different page.")
            }
        }
    }
}
