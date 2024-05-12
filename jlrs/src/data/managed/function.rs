//! Managed type for `Function`, the supertype of all Julia functions.
//!
//! All Julia functions are subtypes of `Function`, a function can be called with the methods
//! of the [`Call`] trait. You don't need to cast a [`Value`] to a [`Function`] in order to call
//! it because [`Value`] also implements [`Call`].
//!
//! [`Call`]: crate::call::Call

use std::{marker::PhantomData, ptr::NonNull};

use jl_sys::jl_value_t;

use super::{value::ValueResult, Ref};
use crate::{
    args::Values,
    call::{Call, ProvideKeywords, WithKeywords},
    convert::ccall_types::{CCallArg, CCallReturn},
    data::{
        layout::valid_layout::{ValidField, ValidLayout},
        managed::{datatype::DataType, private::ManagedPriv, value::Value, Managed},
        types::{abstract_type::AbstractType, construct_type::ConstructType, typecheck::Typecheck},
    },
    error::JlrsResult,
    memory::target::{unrooted::Unrooted, Target, TargetResult},
    prelude::ValueData,
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
    #[inline]
    pub fn datatype(self) -> DataType<'scope> {
        self.as_value().datatype()
    }
}

// Safety: The trait is implemented correctly by using the implementation
// of ValidLayout for FunctionRef
unsafe impl Typecheck for Function<'_, '_> {
    #[inline]
    fn typecheck(ty: DataType) -> bool {
        <FunctionRef as ValidLayout>::valid_layout(ty.as_value())
    }
}

impl_debug!(Function<'_, '_>);

impl<'scope, 'data> ManagedPriv<'scope, 'data> for Function<'scope, 'data> {
    type Wraps = jl_value_t;
    type WithLifetimes<'target, 'da> = Function<'target, 'da>;
    const NAME: &'static str = "Function";

    // Safety: `inner` must not have been freed yet, the result must never be
    // used after the GC might have freed it.
    #[inline]
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self {
            inner,
            _scope: PhantomData,
            _data: PhantomData,
        }
    }

    #[inline]
    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.inner
    }
}

impl<'data> Call<'data> for Function<'_, 'data> {
    #[inline]
    unsafe fn call0<'target, Tgt>(self, target: Tgt) -> ValueResult<'target, 'data, Tgt>
    where
        Tgt: Target<'target>,
    {
        self.as_value().call0(target)
    }

    #[inline]
    unsafe fn call_unchecked<'target, 'value, V, Tgt, const N: usize>(
        self,
        target: Tgt,
        args: V,
    ) -> ValueData<'target, 'data, Tgt>
    where
        V: Values<'value, 'data, N>,
        Tgt: Target<'target>,
    {
        self.as_value().call_unchecked(target, args)
    }

    #[inline]
    unsafe fn call1<'target, Tgt>(
        self,
        target: Tgt,
        arg0: Value<'_, 'data>,
    ) -> ValueResult<'target, 'data, Tgt>
    where
        Tgt: Target<'target>,
    {
        self.as_value().call1(target, arg0)
    }

    #[inline]
    unsafe fn call2<'target, Tgt>(
        self,
        target: Tgt,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
    ) -> ValueResult<'target, 'data, Tgt>
    where
        Tgt: Target<'target>,
    {
        self.as_value().call2(target, arg0, arg1)
    }

    #[inline]
    unsafe fn call3<'target, Tgt>(
        self,
        target: Tgt,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
        arg2: Value<'_, 'data>,
    ) -> ValueResult<'target, 'data, Tgt>
    where
        Tgt: Target<'target>,
    {
        self.as_value().call3(target, arg0, arg1, arg2)
    }

    #[inline]
    unsafe fn call<'target, 'value, V, Tgt, const N: usize>(
        self,
        target: Tgt,
        args: V,
    ) -> ValueResult<'target, 'data, Tgt>
    where
        V: Values<'value, 'data, N>,
        Tgt: Target<'target>,
    {
        self.as_value().call(target, args)
    }
}

impl<'value, 'data> ProvideKeywords<'value, 'data> for Function<'value, 'data> {
    #[inline]
    fn provide_keywords(
        self,
        kws: Value<'value, 'data>,
    ) -> JlrsResult<WithKeywords<'value, 'data>> {
        self.as_value().provide_keywords(kws)
    }
}

/// A reference to an [`Function`] that has not been explicitly rooted.
pub type FunctionRef<'scope, 'data> = Ref<'scope, 'data, Function<'scope, 'data>>;

/// A [`FunctionRef`] with static lifetimes. This is a useful shorthand for signatures of
/// `ccall`able functions that return a [`Function`].
pub type FunctionRet = Ref<'static, 'static, Function<'static, 'static>>;

// Safety: FunctionRef is valid for ty if ty is a subtype of Function
unsafe impl ValidLayout for FunctionRef<'_, '_> {
    #[inline]
    fn valid_layout(ty: Value) -> bool {
        let global = unsafe { Unrooted::new() };
        let function_type = DataType::function_type(&global);
        ty.subtype(function_type.as_value())
    }

    #[inline]
    fn type_object<'target, Tgt: Target<'target>>(target: &Tgt) -> Value<'target, 'static> {
        DataType::function_type(&target).as_value()
    }

    const IS_REF: bool = true;
}

unsafe impl ValidField for Option<FunctionRef<'_, '_>> {
    #[inline]
    fn valid_field(ty: Value) -> bool {
        let global = unsafe { Unrooted::new() };
        let function_type = DataType::function_type(&global);
        ty.subtype(function_type.as_value())
    }
}

use crate::memory::target::TargetType;

/// `Function` or `FunctionRef`, depending on the target type `Tgt`.
pub type FunctionData<'target, 'data, Tgt> =
    <Tgt as TargetType<'target>>::Data<'data, Function<'target, 'data>>;

/// `JuliaResult<Function>` or `JuliaResultRef<FunctionRef>`, depending on the target type `Tgt`.
pub type FunctionResult<'target, 'data, Tgt> =
    TargetResult<'target, 'data, Function<'target, 'data>, Tgt>;

unsafe impl<'scope, 'data> CCallArg for Function<'scope, 'data> {
    type CCallArgType = Value<'scope, 'data>;
    type FunctionArgType = Value<'scope, 'data>;
}

unsafe impl CCallReturn for FunctionRet {
    type CCallReturnType = Value<'static, 'static>;
    type FunctionReturnType = Value<'static, 'static>;
    type ReturnAs = Self;

    #[inline]
    unsafe fn return_or_throw(self) -> Self::ReturnAs {
        self
    }
}

unsafe impl ConstructType for Function<'_, '_> {
    type Static = Function<'static, 'static>;

    const CACHEABLE: bool = false;

    #[inline]
    fn construct_type_uncached<'target, Tgt>(
        target: Tgt,
    ) -> super::value::ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        DataType::function_type(&target).as_value().root(target)
    }

    #[inline]
    fn base_type<'target, Tgt>(target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        Some(DataType::function_type(target).as_value())
    }

    #[inline]
    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        _env: &crate::data::types::construct_type::TypeVarEnv,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        DataType::function_type(&target).as_value().root(target)
    }
}

unsafe impl<'scope, 'data> AbstractType for Function<'scope, 'data> {}
