//! Enforce layout requirements of bits union fields.
//!
//! If a field of a Julia type is a union of several bits types, this field is stored in an
//! interesting way: the field will have the same alignment as the type in the union with the
//! largest alignment, and the same size as the type with the largest size, after the data a
//! single-byte flag is stored which indicates what the active variant is. Unlike normal struct
//! fields, the size does not have to be a multiple of the alignment. This is unlike structs in
//! Rust, whose size is a multiple of their alignment.
//!
//! In order represent such a union in Rust, `JlrsReflect.jl` uses three separate fields: an
//! zero-sized alignment field which enforces the alignment, a container to store the raw bytes,
//! and a flag to indicate the active variant.
use std::fmt::Debug;

/// Trait implemented by the aligning structs, which ensure bits unions are properly aligned.
/// Used in combination with `BitsUnion` and `Flag` to ensure bits unions are inserted correctly.
pub unsafe trait Align: Copy + Debug {
    /// The alignment in bytes
    const ALIGNMENT: usize;
}

/// Trait implemented by structs that can contain a bits union.
/// Used in combination with `Align` and `Flag` to ensure bits unions are inserted correctly.
pub unsafe trait BitsUnionContainer: Copy + Debug {}

/// Trait implemented by structs that can contain the flag of a bits union.
/// Used in combination with `Align` and `BitsUnion` to ensure bits unions are inserted correctly.
pub unsafe trait Flag: Copy + Debug {}

unsafe impl Flag for u8 {}
