//! Enforce layout requirements of union fields.
//!
//! If a field of a Julia type is a union of several bits types, this field is stored in an
//! interesting way: the field will have the same alignment as the type in the union with the
//! largest alignment, and the same size as the type with the largest size, after the data a
//! single-byte flag is stored which indicates what the active variant is. Unlike normal struct
//! fields, the size doesn't  have to be a multiple of the alignment. This is unlike structs in
//! Rust, whose size is a multiple of their alignment.
//!
//! In order represent such a union in Rust, JlrsReflect.jl generates three separate fields: a
//! zero-sized alignment field which enforces the alignment, a container to store the raw bytes,
//! and a flag to indicate the active variant.
//!
//! This module provides three traits, one for each of the fields. [`Align`] ensures the next
//! field, which contains the data of the bits-union, is aligned correctly. [`BitsUnionContainer`]
//! and [`Flag`] are marker traits that are used by jlrs-derive to implement `ValidLayout`
//! correctly.

use std::{
    ffi::c_void,
    fmt::{Debug, Formatter, Result as FmtResult},
    mem::MaybeUninit,
    ptr::NonNull,
};

use jl_sys::jl_bottom_type;

use super::valid_layout::{ValidField, ValidLayout};
use crate::{
    data::{
        managed::{
            datatype::DataType, private::ManagedPriv as _, union::Union, value::Value, Managed,
        },
        types::typecheck::Typecheck,
    },
    private::Private,
};

/// Ensures the next field is aligned to 1 byte.
#[repr(C, align(1))]
#[derive(Copy, Clone, PartialEq)]
pub struct Align1;

unsafe impl Align for Align1 {
    const ALIGNMENT: usize = 1;
}

impl Debug for Align1 {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str("Align<1 byte>")
    }
}

/// Ensures the next field is aligned to 2 bytes.
#[repr(C, align(2))]
#[derive(Copy, Clone, PartialEq)]
pub struct Align2;

unsafe impl Align for Align2 {
    const ALIGNMENT: usize = 2;
}

impl Debug for Align2 {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str("Align<2 bytes>")
    }
}

/// Ensures the next field is aligned to 4 bytes.
#[repr(C, align(4))]
#[derive(Copy, Clone, PartialEq)]
pub struct Align4;

unsafe impl Align for Align4 {
    const ALIGNMENT: usize = 4;
}

impl Debug for Align4 {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str("Align<4 bytes>")
    }
}

/// Ensures the next field is aligned to 8 bytes.
#[repr(C, align(8))]
#[derive(Copy, Clone, PartialEq)]
pub struct Align8;

unsafe impl Align for Align8 {
    const ALIGNMENT: usize = 8;
}

impl Debug for Align8 {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str("Align<8 bytes>")
    }
}

/// Ensures the next field is aligned to 16 bytes.
#[repr(C, align(16))]
#[derive(Copy, Clone, PartialEq)]
pub struct Align16;

unsafe impl Align for Align16 {
    const ALIGNMENT: usize = 16;
}

impl Debug for Align16 {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str("Align<16 bytes>")
    }
}

/// When a `Union` is used as a field type in a struct, there are two possible representations.
/// Which representation is chosen depends on its arguments.
///
/// In the general case the `Union` is simply represented as a `Value`. If all of the are isbits*
/// types an inline representation is used. In this case, the value is essentially stored in an
/// array of bytes that is large enough to contain the largest-sized value, followed by a single,
/// byte-sized flag. This array has the same alignment as the value with the largest required
/// alignment.
///
/// In order to take all of this into account, when mapping a Julia struct that has one of these
/// optimized unions as a field, they are translated to three distinct fields. The first is a
/// zero-sized type with a set alignment, the second a `BitsUnion`, and finally a `u8`. The
/// generic parameter of `BitsUnion` must always be `[MaybeUninit<u8>; N]` with N explicitly equal
/// to the size of the largest possible value. The previous, zero-sized, field ensures the
/// `BitsUnion` is properly aligned, the flag indicates the type of the stored value.
///
/// Currently, even though a struct that contains an optimized union is supported by the
/// `JuliaStruct` macro, these fields can't be used from Rust. If you want to access the value,
/// you can use `Value::get_field` which will essentially convert it to the general representation.
///
/// *The types that are eligible for the optimization is actually not limited to just isbits
/// types. In particular, a struct which contains an optimized union as a field is no longer an
/// isbits type but the optimization still applies.
#[derive(Copy, Clone)]
pub struct BitsUnion<const N: usize>([MaybeUninit<u8>; N]);

impl<const N: usize> Debug for BitsUnion<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if N == 1 {
            f.write_str("BitsUnion<1 byte>")
        } else {
            f.write_fmt(format_args!("BitsUnion<{} bytes>", N))
        }
    }
}

unsafe impl<const N: usize> BitsUnionContainer for BitsUnion<N> {}

#[doc(hidden)]
pub unsafe fn correct_layout_for<A: Align, B: BitsUnionContainer, F: Flag>(u: Union) -> bool {
    let mut bu_sz = 0;
    let mut bu_align = 0;
    if !u.isbits_size_align(&mut bu_sz, &mut bu_align) {
        return false;
    }

    A::ALIGNMENT == bu_align && std::mem::size_of::<B>() == bu_sz
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct EmptyUnion(MaybeUninit<*mut c_void>);

impl Debug for EmptyUnion {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str("Union{}")
    }
}

unsafe impl ValidLayout for EmptyUnion {
    #[inline]
    fn valid_layout(ty: Value) -> bool {
        unsafe { ty.unwrap(Private) == jl_bottom_type }
    }

    const IS_REF: bool = true;

    fn type_object<'target, Tgt: crate::prelude::Target<'target>>(
        _target: &Tgt,
    ) -> Value<'target, 'static> {
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_bottom_type), Private) }
    }
}

unsafe impl ValidField for EmptyUnion {
    #[inline]
    fn valid_field(ty: Value) -> bool {
        unsafe { ty.unwrap(Private) == jl_bottom_type }
    }
}

unsafe impl Typecheck for EmptyUnion {
    #[inline]
    fn typecheck(t: DataType) -> bool {
        <Self as ValidLayout>::valid_layout(t.as_value())
    }
}

/// Trait implemented by the aligning structs, which ensure bits unions are properly aligned.
/// Used in combination with `BitsUnionContainer` and `Flag` to ensure bits unions are inserted
/// correctly.
pub unsafe trait Align: private::AlignPriv {
    /// The alignment in bytes
    const ALIGNMENT: usize;
}

/// Trait implemented by structs that can contain the data of a bits union. Used in combination
/// with `Align` and `Flag` to ensure bits unions are inserted correctly.
pub unsafe trait BitsUnionContainer: private::BitsUnionContainerPriv {}

/// Trait implemented by structs that can contain the flag of a bits union. Used in combination
/// with `Align` and `BitsUnionContainer` to ensure bits unions are inserted correctly.
pub unsafe trait Flag: private::FlagPriv {}
unsafe impl Flag for u8 {}

mod private {
    use std::fmt::Debug;

    use super::{Align1, Align16, Align2, Align4, Align8, BitsUnion};

    pub trait AlignPriv: Copy + Debug {}
    impl AlignPriv for Align1 {}
    impl AlignPriv for Align2 {}
    impl AlignPriv for Align4 {}
    impl AlignPriv for Align8 {}
    impl AlignPriv for Align16 {}

    pub trait BitsUnionContainerPriv: Copy + Debug {}
    impl<const N: usize> BitsUnionContainerPriv for BitsUnion<N> {}

    pub trait FlagPriv: Copy + Debug {}
    impl FlagPriv for u8 {}
}
