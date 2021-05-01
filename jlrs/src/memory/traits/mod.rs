//! Traits used to protect Julia data from being garbage collected, managing their lifetimes, and
//! controlling the garbage collector.

#[cfg(feature = "async")]
pub mod as_unrooted;
pub mod frame;
pub mod gc;
pub mod mode;
pub(crate) mod root_pending;
pub mod scope;
