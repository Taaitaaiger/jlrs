//! Managed type for `Union`.

use std::{marker::PhantomData, ptr::NonNull};

use jl_sys::{
    jl_islayout_inline, jl_type_union, jl_uniontype_t, jl_uniontype_type, jlrs_union_a,
    jlrs_union_b,
};

use super::{
    value::{ValueData, ValueResult},
    Ref,
};
use crate::{
    catch::{catch_exceptions, unwrap_exc},
    data::managed::{private::ManagedPriv, value::Value, Managed},
    impl_julia_typecheck,
    memory::target::{Target, TargetResult},
    private::Private,
};

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
    /// [`Union`]: crate::data::managed::union::Union
    /// [`DataType`]: crate::data::managed::datatype::DataType
    pub fn new<'target, V, Tgt>(target: Tgt, types: V) -> ValueResult<'target, 'static, Tgt>
    where
        V: AsRef<[Value<'scope, 'static>]>,
        Tgt: Target<'target>,
    {
        // Safety: if an exception is thrown it's caught, the result is immediately rooted
        unsafe {
            let types = types.as_ref();

            let callback = || jl_type_union(types.as_ptr() as *mut _, types.len());

            let res = match catch_exceptions(callback, unwrap_exc) {
                Ok(ptr) => Ok(NonNull::new_unchecked(ptr)),
                Err(e) => Err(e),
            };

            target.result_from_ptr(res, Private)
        }
    }

    /// Returns the union of all types in `types`. For each of these types, [`Value::is_kind`]
    /// must return `true`. Note that the result is not necessarily a [`Union`], for example the
    /// union of a single [`DataType`] is that type, not a `Union` with a single variant.
    ///
    /// If an exception is thrown it isn't caught.
    ///
    /// Safety: an exception must not be thrown if this method is called from a `ccall`ed
    /// function.
    ///
    /// [`Union`]: crate::data::managed::union::Union
    /// [`DataType`]: crate::data::managed::datatype::DataType
    #[inline]
    pub unsafe fn new_unchecked<'target, V, Tgt>(
        target: Tgt,
        types: V,
    ) -> ValueData<'target, 'static, Tgt>
    where
        V: AsRef<[Value<'scope, 'static>]>,
        Tgt: Target<'target>,
    {
        let types = types.as_ref();
        let un = jl_type_union(types.as_ptr() as *mut _, types.len());
        target.data_from_ptr(NonNull::new_unchecked(un), Private)
    }

    /// Returns true if the bits-union optimization applies to this union type.
    #[inline]
    pub fn is_bits_union(self) -> bool {
        let v: Value = self.as_value();
        // Safety: The C API function is called with valid arguments
        unsafe { jl_islayout_inline(v.unwrap(Private), &mut 0, &mut 0) != 0 }
    }

    /// Returns true if the bits-union optimization applies to this union type and calculates
    /// the size and aligment if it does. If this method returns false, the calculated size and
    /// alignment are invalid.
    #[inline]
    pub fn isbits_size_align(self, size: &mut usize, align: &mut usize) -> bool {
        let v: Value = self.as_value();
        // Safety: The C API function is called with valid arguments
        unsafe { jl_islayout_inline(v.unwrap(Private), size, align) != 0 }
    }

    /// Returns the size of a field that is of this `Union` type excluding the flag that is used
    /// in bits-unions.
    #[inline]
    pub fn size(self) -> usize {
        let mut sz = 0;
        if !self.isbits_size_align(&mut sz, &mut 0) {
            return std::mem::size_of::<usize>();
        }

        sz
    }

    /// Returns a vector of all type variants this union can have.
    #[inline]
    pub fn variants(self) -> Vec<Value<'scope, 'static>> {
        let mut comps = vec![];
        collect(self.as_value(), &mut comps);
        comps
    }

    /// Unions are stored as binary trees, the arguments are stored as its leaves. This method
    /// returns one of its branches.
    #[inline]
    pub fn a(self) -> Value<'scope, 'static> {
        // Safety: the pointer points to valid data
        unsafe {
            let a = jlrs_union_a(self.unwrap(Private));
            debug_assert!(!a.is_null());
            Value::wrap_non_null(NonNull::new_unchecked(a), Private)
        }
    }

    /// Unions are stored as binary trees, the arguments are stored as its leaves. This method
    /// returns one of its branches.
    #[inline]
    pub fn b(self) -> Value<'scope, 'static> {
        // Safety: the pointer points to valid data
        unsafe {
            let b = jlrs_union_b(self.unwrap(Private));
            debug_assert!(!b.is_null());
            Value::wrap_non_null(NonNull::new_unchecked(b), Private)
        }
    }
}

impl_julia_typecheck!(Union<'scope>, jl_uniontype_type, 'scope);
impl_debug!(Union<'_>);

impl<'scope> ManagedPriv<'scope, '_> for Union<'scope> {
    type Wraps = jl_uniontype_t;
    type WithLifetimes<'target, 'da> = Union<'target>;
    const NAME: &'static str = "Union";

    // Safety: `inner` must not have been freed yet, the result must never be
    // used after the GC might have freed it.
    #[inline]
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    #[inline]
    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

pub(crate) fn nth_union_component<'scope, 'data>(
    v: Value<'scope, 'data>,
    pi: &mut i32,
) -> Option<Value<'scope, 'data>> {
    // Safety: both a and b are never null
    if v.is::<Union>() {
        let un = unsafe { v.cast_unchecked::<Union>() };
        let a = nth_union_component(un.a(), pi);
        if a.is_some() {
            a
        } else {
            *pi -= 1;
            return nth_union_component(un.b(), pi);
        }
    } else {
        if *pi == 0 {
            Some(v)
        } else {
            None
        }
    }
}

fn collect<'scope>(value: Value<'scope, 'static>, comps: &mut Vec<Value<'scope, 'static>>) {
    // Safety: both a and b are never null
    if value.is::<Union>() {
        let u = unsafe { value.cast_unchecked::<Union>() };
        collect(u.a(), comps);
        collect(u.b(), comps);
    } else {
        comps.push(value);
    }
}

pub(crate) fn find_union_component(haystack: Value, needle: Value, nth: &mut u32) -> bool {
    // Safety: both a and b are never null
    if haystack.is::<Union>() {
        let hs = unsafe { haystack.cast_unchecked::<Union>() };
        if find_union_component(hs.a(), needle, nth) {
            true
        } else if find_union_component(hs.b(), needle, nth) {
            true
        } else {
            false
        }
    } else {
        if needle.unwrap_non_null(Private) == haystack.unwrap_non_null(Private) {
            return true;
        } else {
            *nth += 1;
            false
        }
    }
}

impl_construct_type_managed!(Union, 1, jl_uniontype_type);

/// A reference to a [`Union`] that has not been explicitly rooted.
pub type UnionRef<'scope> = Ref<'scope, 'static, Union<'scope>>;

/// A [`UnionRef`] with static lifetimes. This is a useful shorthand for signatures of
/// `ccall`able functions that return a [`Union`].
pub type UnionRet = Ref<'static, 'static, Union<'static>>;

impl_valid_layout!(UnionRef, Union, jl_uniontype_type);

use crate::memory::target::TargetType;

/// `Union` or `UnionRef`, depending on the target type `Tgt`.
pub type UnionData<'target, Tgt> = <Tgt as TargetType<'target>>::Data<'static, Union<'target>>;

/// `JuliaResult<Union>` or `JuliaResultRef<UnionRef>`, depending on the target type `Tgt`.
pub type UnionResult<'target, Tgt> = TargetResult<'target, 'static, Union<'target>, Tgt>;

impl_ccall_arg_managed!(Union, 1);
impl_into_typed!(Union);
