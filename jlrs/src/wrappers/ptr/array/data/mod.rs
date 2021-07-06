//! The data of a Julia array.
//!
//! Arrays in Julia store their contents in one of three ways: inline, as a pointer, or as a bits
//! union. Structs that provide access to their contents can be found in the this module's
//! submodules.

pub mod copied;
pub mod inline;
pub mod union;
pub mod value;
