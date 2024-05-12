//! Embed Julia in a Rust application.
//!
//! There are several ways Julia can be embedded in a Rust application using jlrs, see the
//! [crate-level docs] for more information. To start the Julia runtime runtime, you must
//! use a [`Builder`]. See the [`builder`] module for more information.
//!
//! [`Builder`]: crate::runtime::builder::Builder

#[cfg(any(feature = "local-rt", feature = "async-rt", feature = "multi-rt"))]
pub mod builder;
#[cfg(feature = "async")]
pub mod executor;
pub mod handle;
pub mod state;
#[cfg(feature = "local-rt")]
pub mod sync_rt;
