//! Managed type for `Jlrs.RustResult`.
//!
//! Functions written in Rust that are called from Julia can't arbitrarily throw an exception.
//! `RustResult{T}` is a type provided by the Jlrs package that contains either data or an
//! exception. It can be converted to `T`, if it contains an exception that exception is thrown.
//!
//! This is useful when writing functions that are exposed to Julia with the [`julia_module`]
//! macro. By returning a [`RustResult`], this conversion is automatically invoked by the
//! generated Julia function.
//!
//! [`julia_module`]: jlrs_macros::julia_module

use std::{marker::PhantomData, ptr::NonNull};

use jl_sys::{jl_gc_enable, jl_gc_is_enabled};
use stack_frame::StackFrame;

use super::{
    datatype::DataTypeData, typecheck::Typecheck, union_all::UnionAll, value::typed::TypedValue,
    Ref,
};
use crate::{
    ccall::CCall,
    convert::{
        ccall_types::{CCallArg, CCallReturn},
        construct_type::ConstructType,
    },
    data::{
        layout::valid_layout::{ValidField, ValidLayout},
        managed::private::ManagedPriv,
    },
    memory::{
        stack_frame,
        target::{unrooted::Unrooted, ExtendedTarget},
    },
    prelude::{Bool, DataType, Managed, Module, Target, TargetType, Value, ValueRef},
    private::Private,
};

// TODO: other traits
/// The layout of a [`RustResult`].
#[repr(C)]
pub struct RustResultLayout<'scope, 'data, U: CCallReturn> {
    data: Option<ValueRef<'scope, 'data>>,
    is_exc: Bool,
    _marker: PhantomData<*mut U>,
}

impl<'scope, 'data, U: CCallReturn> Clone for RustResultLayout<'scope, 'data, U> {
    fn clone(&self) -> Self {
        RustResultLayout {
            data: self.data,
            is_exc: self.is_exc,
            _marker: PhantomData,
        }
    }
}

impl<'scope, 'data, U: CCallReturn> Copy for RustResultLayout<'scope, 'data, U> {}

/// A `RustResult` can contain either typed data or an exception.
#[derive(PartialEq)]
#[repr(transparent)]
pub struct RustResult<'scope, 'data, U: CCallReturn>(NonNull<RustResultLayout<'scope, 'data, U>>);

impl<'target, 'data, U: CCallReturn + ValidLayout + ConstructType> RustResult<'target, 'data, U> {
    /// Construct a `RustResult` that contains data.
    pub fn ok<T: Target<'target>>(
        target: ExtendedTarget<'target, '_, '_, T>,
        data: TypedValue<'_, 'data, U>,
    ) -> RustResultData<'target, 'data, U, T> {
        let (target, frame) = target.split();

        frame
            .scope(|mut frame| {
                let unrooted = frame.unrooted();
                unsafe {
                    let ty = Self::base_type(&frame)
                        .apply_type_unchecked(&mut frame, [data.datatype().as_value()])
                        .cast_unchecked::<DataType>();

                    Ok(ty
                        .instantiate_unchecked(&frame, [data.as_value(), Value::false_v(&unrooted)])
                        .as_value()
                        .cast_unchecked::<RustResult<U>>()
                        .root(target))
                }
            })
            .unwrap()
    }

    /// Construct a `RustResult` that contains an exception.
    pub fn err<T: Target<'target>>(
        target: ExtendedTarget<'target, '_, '_, T>,
        error: Value<'_, 'data>,
    ) -> RustResultData<'target, 'data, U, T> {
        let (target, frame) = target.split();

        frame
            .scope(|mut frame| {
                let unrooted = frame.unrooted();
                unsafe {
                    let ty = Self::construct_type(frame.as_extended_target());
                    Ok(ty
                        .instantiate_unchecked(&frame, [error, Value::true_v(&unrooted)])
                        .as_value()
                        .cast_unchecked::<RustResult<U>>()
                        .root(target))
                }
            })
            .unwrap()
    }

    pub fn borrow_err<T: Target<'target>>(
        target: ExtendedTarget<'target, '_, '_, T>,
    ) -> RustResultData<'target, 'data, U, T> {
        let (target, frame) = target.split();

        frame
            .scope(|mut frame| {
                let unrooted = frame.unrooted();
                unsafe {
                    let error = Module::main(&unrooted)
                        .submodule(unrooted, "Jlrs")
                        .unwrap()
                        .as_managed()
                        .global(unrooted, "BorrowError")
                        .unwrap()
                        .as_value()
                        .cast_unchecked::<DataType>()
                        .instance()
                        .unwrap();

                    let ty = Self::construct_type(frame.as_extended_target());
                    Ok(ty
                        .instantiate_unchecked(&frame, [error, Value::true_v(&unrooted)])
                        .as_value()
                        .cast_unchecked::<RustResult<U>>()
                        .root(target))
                }
            })
            .unwrap()
    }

    #[doc(hidden)]
    #[cfg(feature = "ccall")]
    pub unsafe fn borrow_err_internal() -> RustResultRef<'static, 'static, U> {
        let mut frame = StackFrame::new();
        let mut ccall = CCall::new(&mut frame);

        ccall
            .scope(|mut frame| {
                let unrooted = frame.unrooted();
                unsafe {
                    let error = Module::main(&unrooted)
                        .submodule(unrooted, "Jlrs")
                        .unwrap()
                        .as_managed()
                        .global(unrooted, "BorrowError")
                        .unwrap()
                        .as_value()
                        .cast_unchecked::<DataType>()
                        .instance()
                        .unwrap();

                    let ty = Self::construct_type(frame.as_extended_target());
                    Ok(ty
                        .instantiate_unchecked(&frame, [error, Value::true_v(&unrooted)])
                        .as_value()
                        .cast_unchecked::<RustResult<U>>()
                        .as_ref()
                        .leak())
                }
            })
            .unwrap()
    }
}

impl<'scope, 'data, U: CCallReturn> Clone for RustResult<'scope, 'data, U> {
    fn clone(&self) -> Self {
        RustResult(self.0)
    }
}

impl<'scope, 'data, U: CCallReturn> Copy for RustResult<'scope, 'data, U> {}

unsafe impl<U: CCallReturn> Typecheck for RustResult<'_, '_, U> {
    fn typecheck(t: crate::prelude::DataType) -> bool {
        unsafe {
            let unrooted = Unrooted::new();
            let rust_result_typename = Module::main(&unrooted)
                .submodule(unrooted, "Jlrs")
                .unwrap()
                .as_managed()
                .global(unrooted, "RustResult")
                .unwrap()
                .as_value()
                .cast_unchecked::<UnionAll>()
                .base_type()
                .type_name();

            if t.type_name() != rust_result_typename {
                return false;
            }

            let enabled = jl_gc_is_enabled();
            jl_gc_enable(0);
            let mut stack_frame = StackFrame::new();
            let mut ccall = CCall::new(&mut stack_frame);

            let res = ccall.scope(|mut frame| {
                let ty = <U::CCallReturnType as ConstructType>::construct_type(
                    frame.as_extended_target(),
                );
                let param_ty = t.parameters().data().as_slice()[0]
                    .unwrap()
                    .as_value()
                    .cast::<DataType>()?;

                Ok(param_ty == ty)
            });

            jl_gc_enable(enabled);
            res.unwrap()
        }
    }
}

impl<U: CCallReturn> ::std::fmt::Debug for RustResult<'_, '_, U> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self.display_string() {
            Ok(s) => f.write_str(&s),
            Err(e) => f.write_fmt(format_args!("<Cannot display value: {}>", e)),
        }
    }
}

impl<'scope, 'data, U: CCallReturn> ManagedPriv<'scope, 'data> for RustResult<'scope, 'data, U> {
    type Wraps = RustResultLayout<'scope, 'data, U>;
    type TypeConstructorPriv<'target, 'da> = RustResult<'target, 'da, U>;
    const NAME: &'static str = "RustResult";

    // Safety: `inner` must not have been freed yet, the result must never be
    // used after the GC might have freed it.
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner)
    }

    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

unsafe impl<'scope, 'data, U: ConstructType + CCallReturn> ConstructType
    for RustResult<'scope, 'data, U>
{
    fn base_type<'target, T>(target: &T) -> Value<'target, 'static>
    where
        T: Target<'target>,
    {
        unsafe {
            Module::main(target)
                .submodule(target, "Jlrs")
                .unwrap()
                .as_managed()
                .global(target, "RustResult")
                .unwrap()
                .as_value()
        }
    }

    fn construct_type<'target, 'current, 'borrow, T>(
        target: ExtendedTarget<'target, 'current, 'borrow, T>,
    ) -> DataTypeData<'target, T>
    where
        T: Target<'target>,
    {
        let (target, frame) = target.split();
        frame
            .scope(|mut frame| {
                let param_ty = U::construct_type(frame.as_extended_target());
                unsafe {
                    Ok(Self::base_type(&frame)
                        .cast::<UnionAll>()
                        .unwrap()
                        .apply_types_unchecked(&frame, [param_ty.as_value()])
                        .as_value()
                        .cast::<DataType>()
                        .unwrap()
                        .root(target))
                }
            })
            .unwrap()
        //
    }
}

/// A reference to a [`RustResultRef`] that has not been explicitly rooted.
pub type RustResultRef<'scope, 'data, U> = Ref<'scope, 'data, RustResult<'scope, 'data, U>>;

/// A [`RustResultRef`] with static lifetimes. This is a useful shorthand for signatures of
/// `ccall`able functions that return a [`RustResult`].
pub type RustResultRet<U> = Ref<'static, 'static, RustResult<'static, 'static, U>>;

unsafe impl<'scope, 'data, U: CCallReturn> ValidLayout for RustResultRef<'scope, 'data, U> {
    fn valid_layout(ty: Value) -> bool {
        if let Ok(dt) = ty.cast::<DataType>() {
            dt.is::<RustResult<U>>()
        } else {
            false
        }
    }

    const IS_REF: bool = true;
}

unsafe impl<'scope, 'data, U: CCallReturn> ValidField for Option<RustResultRef<'scope, 'data, U>> {
    fn valid_field(ty: Value) -> bool {
        if let Ok(dt) = ty.cast::<DataType>() {
            dt.is::<RustResult<U>>()
        } else {
            false
        }
    }
}

/// `RustResult<U>` or `RustResultRef<U>`, depending on the target type `T`.
pub type RustResultData<'target, 'data, U, T> =
    <T as TargetType<'target>>::Data<'data, RustResult<'target, 'data, U>>;

/// `JuliaResult<RustResult<U>>` or `JuliaResultRef<RustResultRef<U>>`, depending on the target type `T`.
pub type RustResultResult<'target, 'data, U, T> =
    <T as TargetType<'target>>::Result<'data, RustResult<'target, 'data, U>>;

unsafe impl<'scope, 'data, U: CCallReturn> CCallArg for RustResult<'scope, 'data, U> {
    type CCallArgType = Option<ValueRef<'scope, 'data>>;
    type FunctionArgType = Option<ValueRef<'scope, 'data>>;
}

unsafe impl<U: CCallReturn> CCallReturn for Ref<'static, 'static, RustResult<'static, 'static, U>> {
    type CCallReturnType = Option<ValueRef<'static, 'static>>;
    type FunctionReturnType = U::FunctionReturnType;
}
