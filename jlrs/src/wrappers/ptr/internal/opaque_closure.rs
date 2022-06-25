//! Wrapper for `OpaqueClosure`.

use crate::{
    call::Call,
    error::{JlrsResult, JuliaResult, JuliaResultRef},
    impl_debug,
    layout::typecheck::Typecheck,
    memory::{global::Global, output::Output, scope::PartialScope},
    private::Private,
    wrappers::ptr::{
        datatype::DataType, internal::method::MethodRef, private::WrapperPriv, type_name::TypeName,
        value::Value, value::ValueRef, Ref, Wrapper as _,
    },
};
use jl_sys::jl_opaque_closure_t;
use std::{
    ffi::c_void,
    marker::PhantomData,
    ptr::{null_mut, NonNull},
};

/// An opaque closure. Note that opaque closures are currently an experimental feature in Julia.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct OpaqueClosure<'scope>(NonNull<jl_opaque_closure_t>, PhantomData<&'scope ()>);

impl<'scope> OpaqueClosure<'scope> {
    /*
    using Base.Experimental
    oq = Base.Experimental.@opaque (x) -> 2x
    ty = typeof(oq)
    for (a, b) in zip(fieldnames(ty), fieldtypes(ty))
        println(a, ": ", b)
    end
    captures: Any
    world: Int64
    source: Any
    invoke: Ptr{Nothing}
    specptr: Ptr{Nothing}
    */

    /// The data captured by this `OpaqueClosure`.
    pub fn captures(self) -> ValueRef<'scope, 'static> {
        // Safety: the pointer points to valid data
        unsafe { ValueRef::wrap(self.unwrap_non_null(Private).as_ref().captures) }
    }

    /// Returns the world age of this `OpaqueClosure`.
    pub fn world(self) -> usize {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().world }
    }

    /// Returns the `source` field of this `OpaqueClosure`.
    pub fn source(self) -> MethodRef<'scope> {
        // Safety: the pointer points to valid data
        unsafe { MethodRef::wrap(self.unwrap_non_null(Private).as_ref().source) }
    }

    /// Returns a function pointer that can be used to call this `OpaqueClosure`. Using this is
    /// not necessary, you can use the methods of the Call trait instead.
    pub fn invoke(self) -> *mut c_void {
        // Safety: the pointer points to valid data
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
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().specptr }
    }

    /// Use the `Output` to extend the lifetime of this data.
    pub fn root<'target>(self, output: Output<'target>) -> OpaqueClosure<'target> {
        // Safety: the pointer points to valid data
        unsafe {
            let ptr = self.unwrap_non_null(Private);
            output.set_root::<OpaqueClosure>(ptr);
            OpaqueClosure::wrap_non_null(ptr, Private)
        }
    }
}

unsafe impl Typecheck for OpaqueClosure<'_> {
    fn typecheck(t: DataType) -> bool {
        unsafe { t.type_name().wrapper_unchecked() == TypeName::of_opaque_closure(Global::new()) }
    }
}

impl_debug!(OpaqueClosure<'_>);

impl<'scope> WrapperPriv<'scope, 'static> for OpaqueClosure<'scope> {
    type Wraps = jl_opaque_closure_t;
    const NAME: &'static str = "OpaqueClosure";

    // Safety: `inner` must not have been freed yet, the result must never be
    // used after the GC might have freed it.
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

impl<'data> Call<'data> for OpaqueClosure<'_> {
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

impl_root!(OpaqueClosure, 1);

/// A reference to an [`OpaqueClosure`] that has not been explicitly rooted.
pub type OpaqueClosureRef<'scope> = Ref<'scope, 'static, OpaqueClosure<'scope>>;
impl_valid_layout!(OpaqueClosureRef, OpaqueClosure);
impl_ref_root!(OpaqueClosure, OpaqueClosureRef, 1);
