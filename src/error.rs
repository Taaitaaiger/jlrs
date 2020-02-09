//! Everything related to errors.

use std::error::Error;
use std::ffi::CStr;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::os::raw::c_char;

/// Julia can throw exceptions, this struct contains the error message.
#[derive(Debug)]
pub struct Exception {
    pub message: String,
}

impl Display for Exception {
    #[cfg_attr(tarpaulin, skip)]
    fn fmt(&self, formatter: &mut Formatter) -> FmtResult {
        write!(formatter, "message: {}", self.message)
    }
}

impl Exception {
    pub(crate) unsafe fn new(message: *const c_char) -> Self {
        let msg = CStr::from_ptr(message);
        let message = msg.to_string_lossy().into_owned();
        Exception { message }
    }
}

/// Alias that will be used for `Result` in this crate.
pub type JlrsResult<T> = Result<T, Box<dyn Error>>;

/// All different errors.
#[derive(Debug)]
pub enum JlrsError {
    AlreadyInitialized,
    ExceptionOccurred(Exception),
    NullData,
    NotAnArray,
    NotAString,
    DifferentNumberOfElements,
    FunctionNotFound(String),
    IncludeNotFound(String),
    InvalidCharacter,
    NotAModule(String),
    StackSizeExceeded,
    WrongType,
    ZeroDimension,
}

impl Display for JlrsError {
    #[cfg_attr(tarpaulin, skip)]
    fn fmt(&self, formatter: &mut Formatter) -> FmtResult {
        match self {
            JlrsError::AlreadyInitialized => {
                write!(formatter, "The runtime was already initialized")
            }
            JlrsError::ExceptionOccurred(exc) => {
                write!(formatter, "Julia threw an exception. {}", exc)
            }
            JlrsError::NullData => write!(formatter, "The array data pointer is null"),
            JlrsError::NotAnArray => write!(formatter, "This is not an array"),
            JlrsError::NotAString => write!(formatter, "This is not a string"),
            JlrsError::DifferentNumberOfElements => write!(
                formatter,
                "The number of elements does not match the size of the array"
            ),
            JlrsError::FunctionNotFound(func) => {
                write!(formatter, "The function {} could not be found", func)
            }
            JlrsError::IncludeNotFound(inc) => {
                write!(formatter, "The file {} could not be found", inc)
            }
            JlrsError::InvalidCharacter => write!(formatter, "Invalid character"),
            JlrsError::NotAModule(module) => write!(formatter, "{} is not a module", module),
            JlrsError::StackSizeExceeded => write!(formatter, "The stack cannot handle more data"),
            JlrsError::WrongType => {
                write!(formatter, "Requested type does not match the found type")
            }
            JlrsError::ZeroDimension => {
                write!(formatter, "Cannot handle arrays with zero dimensions")
            }
        }
    }
}

impl Error for JlrsError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    #[test]
    fn create_exception() {
        let s = CString::new("Error!!").unwrap();
        let e = unsafe { Exception::new(s.as_ptr()) };
        assert_eq!(e.message, "Error!!");
    }
}
