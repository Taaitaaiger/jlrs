//! Everything related to errors.

use std::error::Error;
use crate::value::array::Dimensions;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// Alias that is used for most `Result`s in this crate. 
pub type JlrsResult<T> = Result<T, Box<JlrsError>>;

/// All different errors.
#[derive(Debug)]
pub enum JlrsError {
    Other(Box<dyn Error + Send + Sync>),
    AlreadyInitialized,
    NotAnArray,
    NotAString,
    FunctionNotFound(String),
    IncludeNotFound(String),
    IncludeError(String, String),
    NoSuchField(String),
    InvalidArrayType,
    InvalidCharacter,
    NotAModule(String),
    AllocError(AllocError),
    WrongType,
    NotInline,
    Inline,
    ZeroDimension,
    OutOfBounds(usize, usize),
    InvalidIndex(Dimensions, Dimensions),
}

impl Display for JlrsError {
    #[cfg_attr(tarpaulin, skip)]
    fn fmt(&self, formatter: &mut Formatter) -> FmtResult {
        match self {
            JlrsError::Other(other) => write!(formatter, "An error occurred: {}", other),
            JlrsError::AlreadyInitialized => {
                write!(formatter, "The runtime was already initialized")
            }
            JlrsError::NotAnArray => write!(formatter, "This is not an array"),
            JlrsError::NotAString => write!(formatter, "This is not a string"),
            JlrsError::FunctionNotFound(func) => {
                write!(formatter, "The function {} could not be found", func)
            }
            JlrsError::NoSuchField(field) => {
                write!(formatter, "The field {} could not be found", field)
            }
            JlrsError::IncludeNotFound(inc) => {
                write!(formatter, "The file {} could not be found", inc)
            }
            JlrsError::IncludeError(inc, err_type) => write!(
                formatter,
                "The file {} could not be included successfully. Exception type: {}",
                inc, err_type
            ),
            JlrsError::InvalidArrayType => write!(formatter, "Invalid array type"),
            JlrsError::InvalidCharacter => write!(formatter, "Invalid character"),
            JlrsError::NotInline => write!(formatter, "Not inline"),
            JlrsError::Inline => write!(formatter, "Inline"),
            JlrsError::NotAModule(module) => write!(formatter, "{} is not a module", module),
            JlrsError::AllocError(AllocError::FrameOverflow(n, cap)) => write!(
                formatter,
                "The frame cannot handle more data. Tried to allocate: {}; capacity: {}",
                n, cap,
            ),
            JlrsError::AllocError(AllocError::StackOverflow(n, cap)) => write!(
                formatter,
                "The stack cannot handle more data. Tried to allocate: {}; capacity: {}",
                n, cap,
            ),
            JlrsError::WrongType => {
                write!(formatter, "Requested type does not match the found type")
            }
            JlrsError::ZeroDimension => {
                write!(formatter, "Cannot handle arrays with zero dimensions")
            }
            JlrsError::OutOfBounds(idx, sz) => write!(
                formatter,
                "Cannot access value at index {} because the number of values is {}",
                idx, sz
            ),
            JlrsError::InvalidIndex(idx, sz) => write!(
                formatter,
                "Inde {} is not valid for array with shape {}",
                idx, sz
            )
        }
    }
}

impl Error for JlrsError {}

impl Into<Box<JlrsError>> for Box<dyn Error + Send + Sync + 'static> {
    fn into(self) -> Box<JlrsError> {
        Box::new(JlrsError::Other(self))
    }
}

/// Frames and data they protect have a memory cost. If the memory set aside for containing frames
/// or the frame itself is exhausted, this error is returned.
#[derive(Copy, Clone, Debug)]
pub enum AllocError {
    //            desired, cap
    StackOverflow(usize, usize),
    FrameOverflow(usize, usize),
}

impl Into<JlrsError> for AllocError {
    fn into(self) -> JlrsError {
        JlrsError::AllocError(self)
    }
}

impl Into<Box<JlrsError>> for AllocError {
    fn into(self) -> Box<JlrsError> {
        Box::new(self.into())
    }
}
