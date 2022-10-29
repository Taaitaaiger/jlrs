//! Wrapper for `OpaqueClosure`.

use crate::{
    call::Call,
    layout::typecheck::Typecheck,
    memory::{target::global::Global, target::Target},
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

    /// Use the target to reroot this data.
    pub fn root<'target, T>(self, target: T) -> T::Data
    where
        T: Target<'target, 'static, OpaqueClosure<'target>>,
    {
        // Safety: the data is valid.
        unsafe { target.data_from_ptr(self.unwrap_non_null(Private), Private) }
    }
}

unsafe impl Typecheck for OpaqueClosure<'_> {
    fn typecheck(t: DataType) -> bool {
        unsafe { t.type_name().wrapper_unchecked() == TypeName::of_opaque_closure(&Global::new()) }
    }
}

impl_debug!(OpaqueClosure<'_>);

impl<'scope> WrapperPriv<'scope, 'static> for OpaqueClosure<'scope> {
    type Wraps = jl_opaque_closure_t;
    type StaticPriv = OpaqueClosure<'static>;
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
    unsafe fn call0<'target, T>(self, target: T) -> T::Result
    where
        T: Target<'target, 'data>,
    {
        self.as_value().call0(target)
    }

    unsafe fn call1<'target, T>(self, target: T, arg0: Value<'_, 'data>) -> T::Result
    where
        T: Target<'target, 'data>,
    {
        self.as_value().call1(target, arg0)
    }

    unsafe fn call2<'target, T>(
        self,
        target: T,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
    ) -> T::Result
    where
        T: Target<'target, 'data>,
    {
        self.as_value().call2(target, arg0, arg1)
    }

    unsafe fn call3<'target, T>(
        self,
        target: T,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
        arg2: Value<'_, 'data>,
    ) -> T::Result
    where
        T: Target<'target, 'data>,
    {
        self.as_value().call3(target, arg0, arg1, arg2)
    }

    unsafe fn call<'target, 'value, V, T>(self, target: T, args: V) -> T::Result
    where
        V: AsRef<[Value<'value, 'data>]>,
        T: Target<'target, 'data>,
    {
        self.as_value().call(target, args)
    }
}

impl_root!(OpaqueClosure, 1);

/// A reference to an [`OpaqueClosure`] that has not been explicitly rooted.
pub type OpaqueClosureRef<'scope> = Ref<'scope, 'static, OpaqueClosure<'scope>>;
impl_valid_layout!(OpaqueClosureRef, OpaqueClosure);
impl_ref_root!(OpaqueClosure, OpaqueClosureRef, 1);
