//! Enforce layout requirements of bits union fields.

/// Trait implemented by the aligning structs, which ensure bits unions are properly aligned.
/// Used in combination with `BitsUnion` and `Flag` to ensure bits unions are inserted correctly.
pub unsafe trait Align {
    /// The alignment in bytes
    const ALIGNMENT: usize;
}

/// Trait implemented by structs that can contain a bits union.
/// Used in combination with `Align` and `Flag` to ensure bits unions are inserted correctly.
pub unsafe trait BitsUnion {}

/// Trait implemented by structs that can contain the flag of a bits union.
/// Used in combination with `Align` and `BitsUnion` to ensure bits unions are inserted correctly.
pub unsafe trait Flag {}

unsafe impl Flag for u8 {}
