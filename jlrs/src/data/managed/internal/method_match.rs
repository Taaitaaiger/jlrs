//! Managed for `MethodMatch`.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/f9720dc2ebd6cd9e3086365f281e62506444ef37/src/julia.h#L585
use std::{marker::PhantomData, ptr::NonNull};

use jl_sys::{jl_method_match_t, jl_method_match_type};

use crate::{
    data::managed::{private::ManagedPriv, simple_vector::SimpleVector, value::Value, Ref},
    impl_julia_typecheck,
    private::Private,
};

/// Managed for `MethodMatch`.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct MethodMatch<'scope>(NonNull<jl_method_match_t>, PhantomData<&'scope ()>);

impl<'scope> MethodMatch<'scope> {
    /*
    inspect(Core.MethodMatch):

    spec_types: Type (const)
    sparams: Core.SimpleVector (const)
    method: Method (const)
    fully_covers: Bool (const)
    */

    /// The `spec_types` field.
    pub fn spec_types(self) -> Option<Value<'scope, 'static>> {
        // Safety: the pointer points to valid data
        unsafe {
            let data = self.unwrap_non_null(Private).as_ref().spec_types.cast();
            let data = NonNull::new(data)?;
            Some(Value::wrap_non_null(data, Private))
        }
    }

    /// The `sparams` field.
    pub fn sparams(self) -> Option<SimpleVector<'scope>> {
        //> Safety: the pointer points to valid data
        unsafe {
            let data = self.unwrap_non_null(Private).as_ref().sparams;
            let data = NonNull::new(data)?;
            Some(SimpleVector::wrap_non_null(data, Private))
        }
    }

    /// The `method` field.
    pub fn method(self) -> Option<Method<'scope>> {
        //> Safety: the pointer points to valid data
        unsafe {
            let data = self.unwrap_non_null(Private).as_ref().method;
            let data = NonNull::new(data)?;
            Some(Method::wrap_non_null(data, Private))
        }
    }

    /// A bool on the julia side, but can be temporarily 0x2 as a sentinel
    /// during construction.
    pub fn fully_covers(self) -> u8 {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().fully_covers }
    }
}

impl_julia_typecheck!(MethodMatch<'scope>, jl_method_match_type, 'scope);
impl_debug!(MethodMatch<'_>);

impl<'scope> ManagedPriv<'scope, '_> for MethodMatch<'scope> {
    type Wraps = jl_method_match_t;
    type TypeConstructorPriv<'target, 'da> = MethodMatch<'target>;
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

/// A reference to a [`MethodMatch`] that has not been explicitly rooted.
pub type MethodMatchRef<'scope> = Ref<'scope, 'static, MethodMatch<'scope>>;
impl_valid_layout!(MethodMatchRef, MethodMatch);

use super::method::Method;
use crate::memory::target::target_type::TargetType;

/// `MethodMetch` or `MethodMetchRef`, depending on the target type `T`.
pub type MethodMatchData<'target, T> =
    <T as TargetType<'target>>::Data<'static, MethodMatch<'target>>;

/// `JuliaResult<MethodMetch>` or `JuliaResultRef<MethodMetchRef>`, depending on the target type
/// `T`.
pub type MethodMatchResult<'target, T> =
    <T as TargetType<'target>>::Result<'static, MethodMatch<'target>>;
