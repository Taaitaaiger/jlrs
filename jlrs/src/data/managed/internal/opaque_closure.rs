//! Managed type for `OpaqueClosure`.

use std::{
    ffi::c_void,
    marker::PhantomData,
    ptr::{null_mut, NonNull},
};

use jl_sys::{jl_opaque_closure_t, jl_opaque_closure_type};

use crate::{
    args::Values,
    call::Call,
    data::{
        managed::{
            datatype::DataType,
            private::ManagedPriv,
            type_name::TypeName,
            value::{Value, ValueResult},
            Managed as _, Ref,
        },
        types::typecheck::Typecheck,
    },
    memory::target::{unrooted::Unrooted, Target, TargetResult},
    prelude::ValueData,
    private::Private,
};

/// An opaque closure. Note that opaque closures are currently an experimental feature in Julia.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct OpaqueClosure<'scope>(NonNull<jl_opaque_closure_t>, PhantomData<&'scope ()>);

impl<'scope> OpaqueClosure<'scope> {
    /*
    using Base.Experimental
    oc = Base.Experimental.@opaque (x) -> 2x
    ty = typeof(oc)
    inspect(ty):

    captures: Any (const)
    world: Int64 (const)
    source: Any (const)
    invoke: Ptr{Nothing} (const)
    specptr: Ptr{Nothing} (const)
    */

    /// The data captured by this `OpaqueClosure`.
    pub fn captures(self) -> Option<Value<'scope, 'static>> {
        // Safety: the pointer points to valid data
        unsafe {
            let data = self.unwrap_non_null(Private).as_ref().captures;
            let data = NonNull::new(data)?;
            Some(Value::wrap_non_null(data, Private))
        }
    }

    /// Returns the world age of this `OpaqueClosure`.
    pub fn world(self) -> usize {
        // Safety: the pointer points to valid data
        unsafe { self.unwrap_non_null(Private).as_ref().world }
    }

    /// Returns the `source` field of this `OpaqueClosure`.
    pub fn source(self) -> Option<Value<'scope, 'static>> {
        // Safety: the pointer points to valid data
        unsafe {
            let data = self.unwrap_non_null(Private).as_ref().source;
            let data = NonNull::new(data)?;
            Some(Value::wrap_non_null(data.cast(), Private))
        }
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
}

unsafe impl Typecheck for OpaqueClosure<'_> {
    fn typecheck(t: DataType) -> bool {
        unsafe { t.type_name() == TypeName::of_opaque_closure(&Unrooted::new()) }
    }
}

impl_debug!(OpaqueClosure<'_>);

impl<'scope, 'data> ManagedPriv<'scope, 'data> for OpaqueClosure<'scope> {
    type Wraps = jl_opaque_closure_t;
    type TypeConstructorPriv<'target, 'da> = OpaqueClosure<'target>;
    const NAME: &'static str = "OpaqueClosure";

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

impl<'data> Call<'data> for OpaqueClosure<'_> {
    unsafe fn call0<'target, T>(self, target: T) -> ValueResult<'target, 'data, T>
    where
        T: Target<'target>,
    {
        self.as_value().call0(target)
    }

    #[inline]
    unsafe fn call_unchecked<'target, 'value, V, T, const N: usize>(
        self,
        target: T,
        args: V,
    ) -> ValueData<'target, 'data, T>
    where
        V: Values<'value, 'data, N>,
        T: Target<'target>,
    {
        self.as_value().call_unchecked(target, args)
    }

    unsafe fn call1<'target, T>(
        self,
        target: T,
        arg0: Value<'_, 'data>,
    ) -> ValueResult<'target, 'data, T>
    where
        T: Target<'target>,
    {
        self.as_value().call1(target, arg0)
    }

    unsafe fn call2<'target, T>(
        self,
        target: T,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
    ) -> ValueResult<'target, 'data, T>
    where
        T: Target<'target>,
    {
        self.as_value().call2(target, arg0, arg1)
    }

    unsafe fn call3<'target, T>(
        self,
        target: T,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
        arg2: Value<'_, 'data>,
    ) -> ValueResult<'target, 'data, T>
    where
        T: Target<'target>,
    {
        self.as_value().call3(target, arg0, arg1, arg2)
    }

    unsafe fn call<'target, 'value, V, T, const N: usize>(
        self,
        target: T,
        args: V,
    ) -> ValueResult<'target, 'data, T>
    where
        V: Values<'value, 'data, N>,
        T: Target<'target>,
    {
        self.as_value().call(target, args)
    }
}

/// A reference to an [`OpaqueClosure`] that has not been explicitly rooted.
pub type OpaqueClosureRef<'scope> = Ref<'scope, 'static, OpaqueClosure<'scope>>;

/// An [`OpaqueClosureRef`] with static lifetimes. This is a useful shorthand for signatures of
/// `ccall`able functions that return a [`OpaqueClosure`].
pub type OpaqueClosureRet = Ref<'static, 'static, OpaqueClosure<'static>>;

impl_valid_layout!(OpaqueClosureRef, OpaqueClosure, jl_opaque_closure_type);

use crate::memory::target::TargetType;

/// `OpaqueClosure` or `OpaqueClosureRef`, depending on the target type `T`.
pub type OpaqueClosureData<'target, T> =
    <T as TargetType<'target>>::Data<'static, OpaqueClosure<'target>>;

/// `JuliaResult<OpaqueClosure>` or `JuliaResultRef<OpaqueClosureRef>`, depending on the target
/// type `T`.
pub type OpaqueClosureResult<'target, T> =
    TargetResult<'target, 'static, OpaqueClosure<'target>, T>;

unsafe impl<'scope> crate::convert::ccall_types::CCallArg for OpaqueClosure<'scope> {
    type CCallArgType = Value<'scope, 'static>;
    type FunctionArgType = Value<'scope, 'static>;
}

unsafe impl crate::convert::ccall_types::CCallReturn
    for crate::data::managed::Ref<'static, 'static, OpaqueClosure<'static>>
{
    type CCallReturnType = Value<'static, 'static>;
    type FunctionReturnType = Value<'static, 'static>;
    type ReturnAs = Self;

    #[inline]
    unsafe fn return_or_throw(self) -> Self::ReturnAs {
        self
    }
}
