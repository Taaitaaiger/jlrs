//! Wrapper for `Function`, the super type of all Julia functions.
//!
//! All Julia functions are subtypes of `Function`, a function can be called with the methods
//! of the [`Call`] trait. Note that you don't need to cast a [`Value`] to a [`Function`] in order
//! to call it because [`Value`] also implements [`Call`].
//!
//! [`Call`]: crate::wrappers::ptr::call::Call

use jl_sys::jl_value_t;
use std::{marker::PhantomData, ptr::NonNull};

use super::{
    call::{Call, CallExt, WithKeywords},
    datatype::DataType,
    private::Wrapper as WrapperPriv,
    value::Value,
    Wrapper,
};
use crate::{
    error::{JlrsError, JlrsResult, JuliaResult, JuliaResultRef, CANNOT_DISPLAY_TYPE},
    impl_debug,
    layout::{
        typecheck::{NamedTuple, Typecheck},
        valid_layout::ValidLayout,
    },
    memory::{frame::Frame, global::Global, scope::Scope},
    private::Private,
};

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
}

unsafe impl ValidLayout for Function<'_, '_> {
    fn valid_layout(ty: Value) -> bool {
        let global = unsafe { Global::new() };
        let function_type = DataType::function_type(global);
        ty.subtype(function_type.as_value())
    }
}

unsafe impl Typecheck for Function<'_, '_> {
    fn typecheck(t: DataType) -> bool {
        <Self as ValidLayout>::valid_layout(t.as_value())
    }
}

impl_debug!(Function<'_, '_>);

impl<'scope, 'data> WrapperPriv<'scope, 'data> for Function<'scope, 'data> {
    type Internal = jl_value_t;
    const NAME: &'static str = "Function";

    unsafe fn wrap_non_null(inner: NonNull<Self::Internal>, _: Private) -> Self {
        Self {
            inner,
            _scope: PhantomData,
            _data: PhantomData,
        }
    }

    unsafe fn unwrap_non_null(self, _: Private) -> NonNull<Self::Internal> {
        self.inner
    }
}

impl<'data> Call<'data> for Function<'_, 'data> {
    unsafe fn call0<'target, 'current, S, F>(self, scope: S) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'data, F>,
        F: Frame<'current>,
    {
        self.as_value().call0(scope)
    }

    unsafe fn call1<'target, 'current, S, F>(
        self,
        scope: S,
        arg0: Value<'_, 'data>,
    ) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'data, F>,
        F: Frame<'current>,
    {
        self.as_value().call1(scope, arg0)
    }

    unsafe fn call2<'target, 'current, S, F>(
        self,
        scope: S,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
    ) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'data, F>,
        F: Frame<'current>,
    {
        self.as_value().call2(scope, arg0, arg1)
    }

    unsafe fn call3<'target, 'current, S, F>(
        self,
        scope: S,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
        arg2: Value<'_, 'data>,
    ) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'data, F>,
        F: Frame<'current>,
    {
        self.as_value().call3(scope, arg0, arg1, arg2)
    }

    unsafe fn call<'target, 'current, 'value, V, S, F>(
        self,
        scope: S,
        args: V,
    ) -> JlrsResult<S::JuliaResult>
    where
        V: AsMut<[Value<'value, 'data>]>,
        S: Scope<'target, 'current, 'data, F>,
        F: Frame<'current>,
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
        V: AsMut<[Value<'value, 'data>]>,
    {
        self.as_value().call_unrooted(global, args)
    }
}

impl<'target, 'current, 'value, 'data> CallExt<'target, 'current, 'value, 'data>
    for Function<'value, 'data>
{
    fn attach_stacktrace<F>(self, frame: &mut F) -> JlrsResult<JuliaResult<'current, 'data>>
    where
        F: Frame<'current>,
    {
        self.as_value().attach_stacktrace(frame)
    }

    fn tracing_call<F>(self, frame: &mut F) -> JlrsResult<JuliaResult<'current, 'data>>
    where
        F: Frame<'current>,
    {
        self.as_value().tracing_call(frame)
    }

    fn tracing_call_unrooted(
        self,
        global: Global<'target>,
    ) -> JlrsResult<JuliaResultRef<'target, 'data>> {
        self.as_value().tracing_call_unrooted(global)
    }

    fn attach_stacktrace_unrooted(
        self,
        global: Global<'target>,
    ) -> JlrsResult<JuliaResultRef<'target, 'data>> {
        self.as_value().attach_stacktrace_unrooted(global)
    }

    fn with_keywords(self, kws: Value<'value, 'data>) -> JlrsResult<WithKeywords<'value, 'data>> {
        if !kws.is::<NamedTuple>() {
            let type_str = kws.datatype().display_string_or(CANNOT_DISPLAY_TYPE);
            Err(JlrsError::NotANamedTuple { type_str })?
        }
        Ok(WithKeywords::new(self.as_value(), kws))
    }
}
