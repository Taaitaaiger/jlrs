//! Wrapper for `MethodMatch`.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/f9720dc2ebd6cd9e3086365f281e62506444ef37/src/julia.h#L585
use crate::{
    impl_julia_typecheck,
    memory::output::Output,
    private::Private,
    wrappers::ptr::{
        internal::method::MethodRef, private::WrapperPriv, simple_vector::SimpleVectorRef,
        value::ValueRef, Ref,
    },
};
use jl_sys::{jl_method_match_t, jl_method_match_type};
use std::{marker::PhantomData, ptr::NonNull};

/// Wrapper for `MethodMatch`.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct MethodMatch<'scope>(NonNull<jl_method_match_t>, PhantomData<&'scope ()>);

impl<'scope> MethodMatch<'scope> {
    /*
    for (a, b) in zip(fieldnames(Core.MethodMatch), fieldtypes(Core.MethodMatch))
        println(a, ": ", b)
    end
    spec_types: Type
    sparams: Core.SimpleVector
    method: Method
    fully_covers: Bool
    */

    /// The `spec_types` field.
    pub fn spec_types(self) -> ValueRef<'scope, 'static> {
        // Safety: the pointer points to valid data
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().spec_types.cast()) }
    }

    /// The `sparams` field.
    pub fn sparams(self) -> SimpleVectorRef<'scope> {
        // Safety: the pointer points to valid data
        unsafe { SimpleVectorRef::wrap(self.unwrap_non_null(Private).as_ref().sparams) }
    }

    /// The `method` field.
    pub fn method(self) -> MethodRef<'scope> {
        // Safety: the pointer points to valid data
        unsafe { MethodRef::wrap(self.unwrap_non_null(Private).as_ref().method) }
    }

    /// A bool on the julia side, but can be temporarily 0x2 as a sentinel
    /// during construction.
    pub fn fully_covers(self) -> u8 {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().fully_covers }
    }

    /// Use the `Output` to extend the lifetime of this data.
    pub fn root<'target>(self, output: Output<'target>) -> MethodMatch<'target> {
        // Safety: the pointer points to valid data
        unsafe {
            let ptr = self.unwrap_non_null(Private);
            output.set_root::<MethodMatch>(ptr);
            MethodMatch::wrap_non_null(ptr, Private)
        }
    }
}

impl_julia_typecheck!(MethodMatch<'scope>, jl_method_match_type, 'scope);
impl_debug!(MethodMatch<'_>);

impl<'scope> WrapperPriv<'scope, '_> for MethodMatch<'scope> {
    type Wraps = jl_method_match_t;
    const NAME: &'static str = "MethodMatch";

    // Safety: `inner` must not have been freed yet, the result must never be
    // used after the GC might have freed it.
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

impl_root!(MethodMatch, 1);

/// A reference to a [`MethodMatch`] that has not been explicitly rooted.
pub type MethodMatchRef<'scope> = Ref<'scope, 'static, MethodMatch<'scope>>;
impl_valid_layout!(MethodMatchRef, MethodMatch);
impl_ref_root!(MethodMatch, MethodMatchRef, 1);
