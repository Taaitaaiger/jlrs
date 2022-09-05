//! Wrapper for `Function`, the supertype of all Julia functions.
//!
//! All Julia functions are subtypes of `Function`, a function can be called with the methods
//! of the [`Call`] trait. Note that you don't need to cast a [`Value`] to a [`Function`] in order
//! to call it because [`Value`] also implements [`Call`].
//!
//! [`Call`]: crate::call::Call

use crate::{
    call::{Call, ProvideKeywords, WithKeywords},
    error::{JlrsResult, JuliaResult, JuliaResultRef},
    layout::{typecheck::Typecheck, valid_layout::ValidLayout},
    memory::{global::Global, output::Output, scope::PartialScope},
    private::Private,
    wrappers::ptr::{datatype::DataType, private::WrapperPriv, value::Value, Wrapper},
};
use jl_sys::jl_value_t;
use std::{marker::PhantomData, ptr::NonNull};

use super::Ref;

/// A Julia function.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Function<'scope, 'data> {
    inner: NonNull<jl_value_t>,
    _scope: PhantomData<&'scope ()>,
    _data: PhantomData<&'data ()>,
}

impl<'scope, 'data> Function<'scope, 'data> {
    /// Returns the `DataType` of this function. In Julia, every function has its own `DataType`.
    pub fn datatype(self) -> DataType<'scope> {
        self.as_value().datatype()
    }

    /// Use the `Output` to extend the lifetime of this data.
    pub fn root<'target>(self, output: Output<'target>) -> Function<'target, 'data> {
        // The pointer points to valid data
        unsafe {
            let ptr = self.unwrap_non_null(Private);
            output.set_root::<Function>(ptr);
            Function::wrap_non_null(ptr, Private)
        }
    }
}

// Safety: The trait is implemented correctly by using the implementation
// of ValidLayout for FunctionRef
unsafe impl Typecheck for Function<'_, '_> {
    fn typecheck(ty: DataType) -> bool {
        <FunctionRef as ValidLayout>::valid_layout(ty.as_value())
    }
}

impl_debug!(Function<'_, '_>);

impl<'scope, 'data> WrapperPriv<'scope, 'data> for Function<'scope, 'data> {
    type Wraps = jl_value_t;
    const NAME: &'static str = "Function";

    // Safety: `inner` must not have been freed yet, the result must never be
    // used after the GC might have freed it.
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self {
            inner,
            _scope: PhantomData,
            _data: PhantomData,
        }
    }

    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.inner
    }
}

impl<'data> Call<'data> for Function<'_, 'data> {
    unsafe fn call0<'target, S>(self, scope: S) -> JlrsResult<JuliaResult<'target, 'data>>
    where
        S: PartialScope<'target>,
    {
        self.as_value().call0(scope)
    }

    unsafe fn call1<'target, S>(
        self,
        scope: S,
        arg0: Value<'_, 'data>,
    ) -> JlrsResult<JuliaResult<'target, 'data>>
    where
        S: PartialScope<'target>,
    {
        self.as_value().call1(scope, arg0)
    }

    unsafe fn call2<'target, S>(
        self,
        scope: S,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
    ) -> JlrsResult<JuliaResult<'target, 'data>>
    where
        S: PartialScope<'target>,
    {
        self.as_value().call2(scope, arg0, arg1)
    }

    unsafe fn call3<'target, S>(
        self,
        scope: S,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
        arg2: Value<'_, 'data>,
    ) -> JlrsResult<JuliaResult<'target, 'data>>
    where
        S: PartialScope<'target>,
    {
        self.as_value().call3(scope, arg0, arg1, arg2)
    }

    unsafe fn call<'target, 'value, V, S>(
        self,
        scope: S,
        args: V,
    ) -> JlrsResult<JuliaResult<'target, 'data>>
    where
        V: AsRef<[Value<'value, 'data>]>,
        S: PartialScope<'target>,
    {
        self.as_value().call(scope, args)
    }

    unsafe fn call0_unrooted<'target>(
        self,
        global: Global<'target>,
    ) -> JuliaResultRef<'target, 'data> {
        self.as_value().call0_unrooted(global)
    }

    unsafe fn call1_unrooted<'target>(
        self,
        global: Global<'target>,
        arg0: Value<'_, 'data>,
    ) -> JuliaResultRef<'target, 'data> {
        self.as_value().call1_unrooted(global, arg0)
    }

    unsafe fn call2_unrooted<'target>(
        self,
        global: Global<'target>,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
    ) -> JuliaResultRef<'target, 'data> {
        self.as_value().call2_unrooted(global, arg0, arg1)
    }

    unsafe fn call3_unrooted<'target>(
        self,
        global: Global<'target>,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
        arg2: Value<'_, 'data>,
    ) -> JuliaResultRef<'target, 'data> {
        self.as_value().call3_unrooted(global, arg0, arg1, arg2)
    }

    unsafe fn call_unrooted<'target, 'value, V>(
        self,
        global: Global<'target>,
        args: V,
    ) -> JuliaResultRef<'target, 'data>
    where
        V: AsRef<[Value<'value, 'data>]>,
    {
        self.as_value().call_unrooted(global, args)
    }
}

impl<'value, 'data> ProvideKeywords<'value, 'data> for Function<'value, 'data> {
    fn provide_keywords(
        self,
        kws: Value<'value, 'data>,
    ) -> JlrsResult<WithKeywords<'value, 'data>> {
        self.as_value().provide_keywords(kws)
    }
}

impl_root!(Function, 2);

/// A reference to an [`Function`] that has not been explicitly rooted.
pub type FunctionRef<'scope, 'data> = Ref<'scope, 'data, Function<'scope, 'data>>;

// Safety: FunctionRef is valid for ty if ty is a subtype of Function
unsafe impl ValidLayout for FunctionRef<'_, '_> {
    fn valid_layout(ty: Value) -> bool {
        let global = unsafe { Global::new() };
        let function_type = DataType::function_type(global);
        ty.subtype(function_type.as_value())
    }

    const IS_REF: bool = true;
}

impl_ref_root!(Function, FunctionRef, 2);
