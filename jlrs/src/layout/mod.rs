//! Traits for checking layout compatibility and enforcing layout requirements.

#[cfg(feature = "jlrs-derive")]
pub mod bits_union;
pub mod typecheck;
pub mod valid_layout;
