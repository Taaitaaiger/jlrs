//! Wrapper for `Union`.

use super::{private::Wrapper, value::Value, ValueRef, Wrapper as _};
use crate::{
    error::{JlrsResult, JuliaResultRef},
    impl_debug, impl_julia_typecheck, impl_valid_layout,
    memory::{frame::Frame, global::Global, scope::Scope},
    private::Private,
};
use jl_sys::{
    jl_islayout_inline, jl_type_union, jl_uniontype_t, jl_uniontype_type,
    jlrs_result_tag_t_JLRS_RESULT_ERR, jlrs_type_union,
};
use std::{marker::PhantomData, ptr::NonNull};

/// A struct field can have a type that's a union of several types. In this case, the type of this
/// field is an instance of `Union`.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Union<'scope>(NonNull<jl_uniontype_t>, PhantomData<&'scope ()>);

impl<'scope> Union<'scope> {
    /// Returns the union of all types in `types`. For each of these types, [`Value::is_kind`]
    /// must return `true`. Note that the result is not necessarily a [`Union`], for example the
    /// union of a single [`DataType`] is that type, not a `Union` with a single variant.
    ///
    /// If an exception is thrown, it's caught and returned.
    ///
    /// [`Union`]: crate::wrappers::ptr::union::Union
    /// [`DataType`]: crate::wrappers::ptr::datatype::DataType
    pub fn new<'target, 'current, V, S, F>(scope: S, mut types: V) -> JlrsResult<S::JuliaResult>
    where
        V: AsMut<[Value<'scope, 'static>]>,
        S: Scope<'target, 'current, 'static, F>,
        F: Frame<'current>,
    {
        unsafe {
            let types = types.as_mut();
            let un = jlrs_type_union(types.as_mut_ptr().cast(), types.len());
            if un.flag == jlrs_result_tag_t_JLRS_RESULT_ERR {
                scope.call_result(Err(NonNull::new_unchecked(un.data)), Private)
            } else {
                scope.call_result(Ok(NonNull::new_unchecked(un.data)), Private)
            }
        }
    }

    /// Returns the union of all types in `types`. For each of these types, [`Value::is_kind`]
    /// must return `true`. Note that the result is not necessarily a [`Union`], for example the
    /// union of a single [`DataType`] is that type, not a `Union` with a single variant.
    ///
    /// If an exception is thrown the process aborts.
    ///
    /// [`Union`]: crate::wrappers::ptr::union::Union
    /// [`DataType`]: crate::wrappers::ptr::datatype::DataType
    pub fn new_unchecked<'target, 'current, V, S, F>(scope: S, mut types: V) -> JlrsResult<S::Value>
    where
        V: AsMut<[Value<'scope, 'static>]>,
        S: Scope<'target, 'current, 'static, F>,
        F: Frame<'current>,
    {
        unsafe {
            let types = types.as_mut();
            let un = jl_type_union(types.as_mut_ptr().cast(), types.len());
            scope.value(NonNull::new_unchecked(un), Private)
        }
    }

    /// Returns the union of all types in `types`. For each of these types, [`Value::is_kind`]
    /// must return `true`. Note that the result is not necessarily a [`Union`], for example the
    /// union of a single [`DataType`] is that type, not a `Union` with a single variant. Unlike
    /// [`Union::new`] this method doesn't root the allocated value or exception.
    ///
    /// [`Union`]: crate::wrappers::ptr::union::Union
    /// [`DataType`]: crate::wrappers::ptr::datatype::DataType
    pub fn new_unrooted<'global, V>(
        _: Global<'global>,
        mut types: V,
    ) -> JuliaResultRef<'global, 'static>
    where
        V: AsMut<[Value<'scope, 'static>]>,
    {
        unsafe {
            let types = types.as_mut();
            let un = jlrs_type_union(types.as_mut_ptr().cast(), types.len());
            if un.flag == jlrs_result_tag_t_JLRS_RESULT_ERR {
                Err(ValueRef::wrap(un.data))
            } else {
                Ok(ValueRef::wrap(un.data))
            }
        }
    }

    /// Returns the union of all types in `types`. For each of these types, [`Value::is_kind`]
    /// must return `true`. Note that the result is not necessarily a [`Union`], for example the
    /// union of a single [`DataType`] is that type, not a `Union` with a single variant. Unlike
    /// [`Union::new`] this method doesn't root the allocated value.
    ///
    /// If an exception is thrown the process aborts.
    ///
    /// [`Union`]: crate::wrappers::ptr::union::Union
    /// [`DataType`]: crate::wrappers::ptr::datatype::DataType
    pub fn new_unrooted_unchecked<'global, V>(
        _: Global<'global>,
        mut types: V,
    ) -> ValueRef<'global, 'static>
    where
        V: AsMut<[Value<'scope, 'static>]>,
    {
        unsafe {
            let types = types.as_mut();
            let un = jl_type_union(types.as_mut_ptr().cast(), types.len());
            ValueRef::wrap(un)
        }
    }

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
    pub fn variants(self) -> Vec<ValueRef<'scope, 'static>> {
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
    pub fn a(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().a) }
    }

    /// Unions are stored as binary trees, the arguments are stored as its leaves. This method
    /// returns one of its branches.
    pub fn b(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().b) }
    }
}

impl_julia_typecheck!(Union<'scope>, jl_uniontype_type, 'scope);
impl_debug!(Union<'_>);
impl_valid_layout!(Union<'scope>, 'scope);

impl<'scope> Wrapper<'scope, '_> for Union<'scope> {
    type Internal = jl_uniontype_t;
    const NAME: &'static str = "Union";

    unsafe fn wrap_non_null(inner: NonNull<Self::Internal>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    unsafe fn unwrap_non_null(self, _: Private) -> NonNull<Self::Internal> {
        self.0
    }
}

pub(crate) fn nth_union_component<'scope, 'data>(
    v: Value<'scope, 'data>,
    pi: &mut i32,
) -> Option<Value<'scope, 'data>> {
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
