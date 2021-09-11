use std::error::Error;
use std::fmt;

#[cfg(feature = "async-std-rt")]
pub mod async_std_rt;

#[cfg(any(feature = "tokio-rt", feature = "docs"))]
pub mod tokio_rt;

#[derive(Debug)]
pub struct SendError<T>(pub T);

impl<T> fmt::Display for SendError<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "channel closed")
    }
}

impl<T: fmt::Debug> std::error::Error for SendError<T> {}

#[derive(Debug)]
pub enum TrySendError<T> {
    Full(T),
    Closed(T),
}

impl<T: fmt::Debug> Error for TrySendError<T> {}

impl<T> fmt::Display for TrySendError<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            fmt,
            "{}",
            match self {
                TrySendError::Full(..) => "no available capacity",
                TrySendError::Closed(..) => "channel closed",
            }
        )
    }
}
