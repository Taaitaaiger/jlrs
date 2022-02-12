//! Wrapper for `OpaqueClosure`.

use super::super::type_name::TypeName;
use super::super::union_all::UnionAll;
use super::super::MethodRef;
use super::super::{call::Call, datatype::DataType, private::Wrapper, value::Value, Wrapper as _};
use crate::error::{JlrsResult, JuliaResultRef};
use crate::impl_debug;
use crate::layout::typecheck::Typecheck;
use crate::layout::valid_layout::ValidLayout;
use crate::memory::{frame::Frame, global::Global, scope::Scope};
use crate::{private::Private, wrappers::ptr::ValueRef};
use jl_sys::jl_opaque_closure_t;
use std::ffi::c_void;
use std::ptr::null_mut;
use std::{marker::PhantomData, ptr::NonNull};

/// An opaque closure. Note that opaque closures are currently an experimental feature in Julia.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct OpaqueClosure<'scope>(NonNull<jl_opaque_closure_t>, PhantomData<&'scope ()>);

impl<'scope> OpaqueClosure<'scope> {
    /*
    oq = Base.Experiment.@opaque (x) -> 2x
    ty = typeof(oq)
    for (a, b) in zip(fieldnames(ty), fieldtypes(ty))
        println(a, ": ", b)
    end
    captures: Any
    isva: Bool
    world: Int64
    source: Any
    invoke: Ptr{Nothing}
    specptr: Ptr{Nothing}
    */

    /// The data captured by this `OpaqueClosure`.
    pub fn captures(self) -> ValueRef<'scope, 'static> {
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().captures) }
    }

    /// Returns `true` is this `OpaqueClosure` takes an arbitrary number of arguments.
    pub fn is_vararg(self) -> bool {
        unsafe { self.unwrap_non_null(Private).as_ref().isva != 0 }
    }

    /// Returns the world age of this `OpaqueClosure`.
    pub fn world(self) -> usize {
        unsafe { self.unwrap_non_null(Private).as_ref().world }
    }

    /// Returns the `source` field of this `OpaqueClosure`.
    pub fn source(self) -> MethodRef<'scope> {
        unsafe { MethodRef::wrap(self.unwrap_non_null(Private).as_ref().source) }
    }

    /// Returns a function pointer that can be used to call this `OpaqueClosure`. Using this is
    /// not necessary, you can use the methods of the Call trait instead.
    pub fn invoke(self) -> *mut c_void {
        unsafe {
            self.unwrap_non_null(Private)
                .as_ref()
                .invoke
                .map(|v| v as *mut c_void)
                .unwrap_or(null_mut())
        }
    }

    /// Returns the `specptr` field of this `OpaqueClosure`.
    pub fn specptr(self) -> *mut c_void {
        unsafe { self.unwrap_non_null(Private).as_ref().specptr }
    }
}

unsafe impl Typecheck for OpaqueClosure<'_> {
    fn typecheck(t: DataType) -> bool {
        unsafe { t.type_name().wrapper_unchecked() == TypeName::of_opaque_closure(Global::new()) }
    }
}

unsafe impl ValidLayout for OpaqueClosure<'_> {
    fn valid_layout(ty: Value) -> bool {
        unsafe {
            if let Ok(dt) = ty.cast::<DataType>() {
                dt.type_name().wrapper_unchecked() == TypeName::of_opaque_closure(Global::new())
            } else if let Ok(ua) = ty.cast::<UnionAll>() {
                ua.base_type()
                    .wrapper_unchecked()
                    .type_name()
                    .wrapper_unchecked()
                    == TypeName::of_opaque_closure(Global::new())
            } else {
                false
            }
        }
    }
}

impl_debug!(OpaqueClosure<'_>);

impl<'scope> Wrapper<'scope, 'static> for OpaqueClosure<'scope> {
    type Wraps = jl_opaque_closure_t;
    const NAME: &'static str = "OpaqueClosure";

    #[inline(always)]
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    #[inline(always)]
    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

impl<'data> Call<'data> for OpaqueClosure<'_> {
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
