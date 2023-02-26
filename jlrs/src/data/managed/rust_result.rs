//! Managed type for `Jlrs.RustResult`.
//!
//! Functions written in Rust that are called from Julia can't arbitrarily throw an exception.
//! `RustResult{T}` is a type provided by the Jlrs package that contains either data or an
//! exception. It can be converted to `T`, if it contains an exception that exception is thrown.
//!
//! This is useful when writing functions that are exported to Julia with the [`julia_module`]
//! macro. By returning a [`RustResult`], this conversion is automatically invoked by the
//! generated Julia function.
//!
//! [`julia_module`]: jlrs_macros::julia_module

use std::{marker::PhantomData, ptr::NonNull};

use super::{
    union_all::UnionAll,
    value::{typed::TypedValue, ValueData},
    Ref,
};
#[cfg(feature = "ccall")]
use crate::{ccall::CCall, memory::stack_frame::StackFrame};
use crate::{
    convert::ccall_types::{CCallArg, CCallReturn},
    data::{
        layout::bool::Bool,
        managed::{
            datatype::DataType,
            module::Module,
            private::ManagedPriv,
            string::JuliaString,
            value::{Value, ValueRef},
            Managed,
        },
        types::construct_type::ConstructType,
    },
    error::JlrsError,
    memory::target::{target_type::TargetType, ExtendedTarget, Target},
    private::Private,
};
/// A `RustResult` can contain either typed data or an exception.
#[derive(PartialEq)]
#[repr(transparent)]
pub struct RustResult<'scope, 'data, U: ConstructType>(NonNull<RustResultLayout<'scope, 'data, U>>);

impl<'target, 'data, U: ConstructType> RustResult<'target, 'data, U> {
    /// Constructs a `RustResult` that contains data.
    pub fn ok<T: Target<'target>>(
        target: ExtendedTarget<'target, '_, '_, T>,
        data: TypedValue<'_, 'data, U>,
    ) -> RustResultData<'target, 'data, U, T> {
        let (target, frame) = target.split();

        frame
            .scope(|mut frame| {
                let unrooted = frame.unrooted();
                unsafe {
                    let ty = Self::construct_type(frame.as_extended_target())
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

    /// Constructs a `RustResult` that contains an exception.
    pub fn error<T: Target<'target>>(
        target: ExtendedTarget<'target, '_, '_, T>,
        error: Value<'_, 'data>,
    ) -> RustResultData<'target, 'data, U, T> {
        let (target, frame) = target.split();

        frame
            .scope(|mut frame| {
                let unrooted = frame.unrooted();
                unsafe {
                    let ty = Self::construct_type(frame.as_extended_target())
                        .cast_unchecked::<DataType>();
                    Ok(ty
                        .instantiate_unchecked(&frame, [error, Value::true_v(&unrooted)])
                        .as_value()
                        .cast_unchecked::<RustResult<U>>()
                        .root(target))
                }
            })
            .unwrap()
    }

    /// Constructs a `RustResult` that contains a `Jlrs.BorrowException`.
    pub fn borrow_error<T: Target<'target>>(
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

                    let ty = Self::construct_type(frame.as_extended_target())
                        .cast_unchecked::<DataType>();
                    Ok(ty
                        .instantiate_unchecked(&frame, [error, Value::true_v(&unrooted)])
                        .as_value()
                        .cast_unchecked::<RustResult<U>>()
                        .root(target))
                }
            })
            .unwrap()
    }

    /// Constructs a `RustResult` that contains `error`, which is converted to a `Jlrs.JlrsError`.
    pub fn jlrs_error<T: Target<'target>>(
        target: ExtendedTarget<'target, '_, '_, T>,
        error: JlrsError,
    ) -> RustResultData<'target, 'data, U, T> {
        let (target, frame) = target.split();

        frame
            .scope(|mut frame| {
                let unrooted = frame.unrooted();
                unsafe {
                    let msg = JuliaString::new(&mut frame, format!("{}", error));
                    let error = Module::main(&unrooted)
                        .submodule(unrooted, "Jlrs")
                        .unwrap()
                        .as_managed()
                        .global(unrooted, "JlrsError")
                        .unwrap()
                        .as_value()
                        .cast_unchecked::<DataType>()
                        .instantiate_unchecked(&mut frame, [msg.as_value()]);

                    let ty = Self::construct_type(frame.as_extended_target())
                        .cast_unchecked::<DataType>();
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
    pub unsafe fn borrow_error_internal() -> RustResultRef<'static, 'static, U> {
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

                    let ty = Self::construct_type(frame.as_extended_target())
                        .cast_unchecked::<DataType>();
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

impl<'scope, 'data, U: ConstructType> Clone for RustResult<'scope, 'data, U> {
    fn clone(&self) -> Self {
        RustResult(self.0)
    }
}

impl<'scope, 'data, U: ConstructType> Copy for RustResult<'scope, 'data, U> {}

// TODO: not sure if this should be implemented or not. RustResult is only intended to be returned
// from ccalled functions, so it's not necessary to support Typecheck, ValidLayout, and
// ValidField.  The main issue is that unlike other implementations of these traits, an
// ExtendedTarget is needed to construct the type object associated with U.
/*unsafe impl<U: ConstructType> Typecheck for RustResult<'_, '_, U> {
    fn typecheck(t: DataType) -> bool {
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

            todo!()
        }
    }
}*/

impl<U: ConstructType> ::std::fmt::Debug for RustResult<'_, '_, U> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self.display_string() {
            Ok(s) => f.write_str(&s),
            Err(e) => f.write_fmt(format_args!("<Cannot display value: {}>", e)),
        }
    }
}

impl<'scope, 'data, U: ConstructType> ManagedPriv<'scope, 'data> for RustResult<'scope, 'data, U> {
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

unsafe impl<'scope, 'data, U: ConstructType> ConstructType for RustResult<'scope, 'data, U> {
    fn construct_type<'target, 'current, 'borrow, T>(
        target: ExtendedTarget<'target, '_, '_, T>,
    ) -> ValueData<'target, 'static, T>
    where
        T: Target<'target>,
    {
        let (target, frame) = target.split();
        frame
            .scope(|mut frame| {
                let param_ty = U::construct_type(frame.as_extended_target());
                unsafe {
                    let base_type = Module::main(&frame)
                        .submodule(&frame, "Jlrs")
                        .unwrap()
                        .as_managed()
                        .global(&frame, "RustResult")
                        .unwrap()
                        .as_value();

                    Ok(base_type
                        .cast::<UnionAll>()
                        .unwrap()
                        .apply_types_unchecked(&frame, [param_ty.as_value()])
                        .as_value()
                        .root(target))
                }
            })
            .unwrap()
    }
}

/// A reference to a [`RustResultRef`] that has not been explicitly rooted.
pub type RustResultRef<'scope, 'data, U> = Ref<'scope, 'data, RustResult<'scope, 'data, U>>;

/// A [`RustResultRef`] with static lifetimes. This is a useful shorthand for signatures of
/// `ccall`able functions that return a [`RustResult`].
pub type RustResultRet<U> = Ref<'static, 'static, RustResult<'static, 'static, U>>;

/*unsafe impl<'scope, 'data, U: ConstructType> ValidLayout for RustResultRef<'scope, 'data, U> {
    fn valid_layout(ty: Value) -> bool {
        if let Ok(dt) = ty.cast::<DataType>() {
            dt.is::<RustResult<U>>()
        } else {
            false
        }
    }

    const IS_REF: bool = true;
}

unsafe impl<'scope, 'data, U: ConstructType> ValidField for Option<RustResultRef<'scope, 'data, U>> {
    fn valid_field(ty: Value) -> bool {
        if let Ok(dt) = ty.cast::<DataType>() {
            dt.is::<RustResult<U>>()
        } else {
            false
        }
    }
}*/

/// `RustResult<U>` or `RustResultRef<U>`, depending on the target type `T`.
pub type RustResultData<'target, 'data, U, T> =
    <T as TargetType<'target>>::Data<'data, RustResult<'target, 'data, U>>;

/// `JuliaResult<RustResult<U>>` or `JuliaResultRef<RustResultRef<U>>`, depending on the target type `T`.
pub type RustResultResult<'target, 'data, U, T> =
    <T as TargetType<'target>>::Result<'data, RustResult<'target, 'data, U>>;

unsafe impl<'scope, 'data, U: ConstructType> CCallArg for RustResult<'scope, 'data, U> {
    type CCallArgType = Value<'scope, 'data>;
    type FunctionArgType = Value<'scope, 'static>;
}

unsafe impl<U: ConstructType> CCallReturn
    for Ref<'static, 'static, RustResult<'static, 'static, U>>
{
    type CCallReturnType = Value<'static, 'static>;
    type FunctionReturnType = U;
}

/// The layout of a [`RustResult`].
#[repr(C)]
pub struct RustResultLayout<'scope, 'data, U: ConstructType> {
    data: Option<ValueRef<'scope, 'data>>,
    is_exc: Bool,
    _marker: PhantomData<U>,
}

impl<'scope, 'data, U: ConstructType> Clone for RustResultLayout<'scope, 'data, U> {
    fn clone(&self) -> Self {
        RustResultLayout {
            data: self.data,
            is_exc: self.is_exc,
            _marker: PhantomData,
        }
    }
}

impl<'scope, 'data, U: ConstructType> Copy for RustResultLayout<'scope, 'data, U> {}
