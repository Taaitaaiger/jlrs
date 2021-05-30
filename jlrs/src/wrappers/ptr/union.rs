//! Wrapper for `Core.Union`.

use super::{private::Wrapper, value::Value, ValueRef, Wrapper as _};
use crate::{impl_julia_typecheck, impl_valid_layout, private::Private};
use jl_sys::{jl_islayout_inline, jl_uniontype_t, jl_uniontype_type};
use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
    ptr::NonNull,
};

/// A struct field can have a type that's a union of several types. In this case, the type of this
/// field is an instance of `Union`.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Union<'frame>(NonNull<jl_uniontype_t>, PhantomData<&'frame ()>);

impl<'frame> Union<'frame> {
    /// Returns true if the bits-union optimization applies to this union type.
    pub fn is_bits_union(self) -> bool {
        unsafe {
            let v: Value = self.as_value();
            jl_islayout_inline(v.unwrap(Private), &mut 0, &mut 0) != 0
        }
    }

    /// Returns true if the bits-union optimization applies to this union type and calculates
    /// the size and aligment if it does. If this method returns false, the calculated size and
    /// alignment are invalid.
    pub fn isbits_size_align(self, size: &mut usize, align: &mut usize) -> bool {
        unsafe {
            let v: Value = self.as_value();
            jl_islayout_inline(v.unwrap(Private), size, align) != 0
        }
    }

    /// Returns the size of a field that is of this `Union` type excluding the flag that is used
    /// in bits-unions.
    pub fn size(self) -> usize {
        let mut sz = 0;
        if !self.isbits_size_align(&mut sz, &mut 0) {
            return std::mem::size_of::<usize>();
        }

        sz
    }

    /// Returns a vector of all type variants this union can have.
    pub fn variants(self) -> Vec<ValueRef<'frame, 'static>> {
        let mut comps = vec![];
        collect(self.as_value(), &mut comps);
        comps
    }

    /*
    for (a, b) in zip(fieldnames(Union), fieldtypes(Union))
        println(a, ": ", b)
    end
    a: Any
    b: Any
    */

    /// Unions are stored as binary trees, the arguments are stored as its leaves. This method
    /// returns one of its branches.
    pub fn a(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().a) }
    }

    /// Unions are stored as binary trees, the arguments are stored as its leaves. This method
    /// returns one of its branches.
    pub fn b(self) -> ValueRef<'frame, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().b) }
    }
}

impl<'scope> Debug for Union<'scope> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("Union").finish()
    }
}

impl_julia_typecheck!(Union<'frame>, jl_uniontype_type, 'frame);

impl_valid_layout!(Union<'frame>, 'frame);

impl<'scope> Wrapper<'scope, '_> for Union<'scope> {
    type Internal = jl_uniontype_t;

    unsafe fn wrap_non_null(inner: NonNull<Self::Internal>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    unsafe fn unwrap_non_null(self, _: Private) -> NonNull<Self::Internal> {
        self.0
    }
}

pub(crate) fn nth_union_component<'frame, 'data>(
    v: Value<'frame, 'data>,
    pi: &mut i32,
) -> Option<Value<'frame, 'data>> {
    unsafe {
        match v.cast::<Union>() {
            Ok(un) => {
                let a = nth_union_component(un.a().value_unchecked(), pi);
                if a.is_some() {
                    a
                } else {
                    *pi -= 1;
                    return nth_union_component(un.b().value_unchecked(), pi);
                }
            }
            Err(_) => {
                if *pi == 0 {
                    Some(v)
                } else {
                    None
                }
            }
        }
    }
}

fn collect<'scope>(value: Value<'scope, 'static>, comps: &mut Vec<ValueRef<'scope, 'static>>) {
    unsafe {
        match value.cast::<Union>() {
            Ok(u) => {
                collect(u.a().value_unchecked(), comps);
                collect(u.b().value_unchecked(), comps);
            }
            Err(_) => {
                comps.push(value.as_ref());
            }
        }
    }
}

pub(crate) fn find_union_component(haystack: Value, needle: Value, nth: &mut u32) -> bool {
    unsafe {
        match haystack.cast::<Union>() {
            Ok(hs) => {
                if find_union_component(hs.a().value_unchecked(), needle, nth) {
                    true
                } else if find_union_component(hs.b().value_unchecked(), needle, nth) {
                    true
                } else {
                    false
                }
            }
            Err(_) => {
                if needle.unwrap_non_null(Private) == haystack.unwrap_non_null(Private) {
                    return true;
                } else {
                    *nth += 1;
                    false
                }
            }
        }
    }
}
