//! Traits used to protect Julia data from being garbage collected, managing their lifetimes, and
//! controlling the garbage collector.

#[cfg(all(feature = "async", target_os = "linux"))]
pub mod as_unrooted;
pub mod frame;
pub mod gc;
pub mod mode;
pub(crate) mod root;
pub mod scope;
