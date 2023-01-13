//! Construct a Julia type from a Rust type.
//!
//! Many types in Julia can be constructed using nothing but the type in Rust. These types
//! implement the [`ConstructType`] trait. You shouldn't implement this trait manually, it's
//! automatically derived by JlrsReflect. This trait is mainly used by the `CCallArg` and
//! `CCallReturn` traits to to generate correctly-typed function signatures and `ccall`
//! invocations when the `julia_module` macro is used to make Rust functions callable from Julia.

use std::{ffi::c_void, ptr::NonNull};

use jl_sys::jl_apply_type;

use crate::{
    data::managed::{
        datatype::DataTypeData,
        private::ManagedPriv,
        union_all::UnionAll,
        value::{Value, ValueData},
    },
    memory::target::{ExtendedTarget, Target},
    prelude::{DataType, Managed},
    private::Private,
};

/// Construct the Julia type associated with this Rust type.
pub unsafe trait ConstructType {
    /// Returns the base type of this Rust type.
    ///
    /// If this type has no parameters the base type is the associated `DataType`. If this type
    /// does have type parameters, it's the underlying [`UnionAll`].
    fn base_type<'target, T>(target: &T) -> Value<'target, 'static>
    where
        T: Target<'target>;

    /// Returns the [`DataType`] associated with this type.
    ///
    /// [`DataType`]: crate::data::managed::datatype::DataType
    fn construct_type<'target, T>(
        target: ExtendedTarget<'target, '_, '_, T>,
    ) -> DataTypeData<'target, T>
    where
        T: Target<'target>;
}

/// Construct the "relaxed" Julia type associated with this Rust type.
///
/// Unlike `ConstructType`, the constructed type is allowed to be a `Union` or `UnionAll`. An
/// example of a type that implements this trait but doesn't implement `ConstructType` is `Array`.
pub unsafe trait ConstructTypeRelaxed {
    /// Returns the Julia type associated with this type. Unlike `ConstructType`, the
    /// constructed type is allowed to be a `Union` or `UnionAll`.
    fn construct_type_relaxed<'target, T>(
        target: ExtendedTarget<'target, '_, '_, T>,
    ) -> ValueData<'target, 'static, T>
    where
        T: Target<'target>;
}

unsafe impl<U: ConstructType> ConstructTypeRelaxed for U {
    fn construct_type_relaxed<'target, T>(
        target: ExtendedTarget<'target, '_, '_, T>,
    ) -> ValueData<'target, 'static, T>
    where
        T: Target<'target>,
    {
        let (target, frame) = target.split();

        frame
            .scope(|mut frame| unsafe {
                let unrooted = frame.unrooted();
                Ok(
                    Self::construct_type(unrooted.into_extended_target(&mut frame))
                        .as_value()
                        .root(target),
                )
            })
            .unwrap()
    }
}

macro_rules! impl_construct_type {
    ($ty:ty) => {
        unsafe impl ConstructType for $ty {
            fn base_type<'target, T>(
                target: &T,
            ) -> crate::data::managed::value::Value<'target, 'static>
            where
                T: Target<'target>,
            {
                unsafe {
                    <Self as crate::convert::into_julia::IntoJulia>::julia_type(target).as_value()
                }
            }

            fn construct_type<'target, 'current, 'borrow, T>(
                target: ExtendedTarget<'target, 'current, 'borrow, T>,
            ) -> DataTypeData<'target, T>
            where
                T: Target<'target>,
            {
                let (target, _) = target.split();
                <Self as crate::convert::into_julia::IntoJulia>::julia_type(target)
            }
        }
    };
}

impl_construct_type!(u8);
impl_construct_type!(u16);
impl_construct_type!(u32);
impl_construct_type!(u64);
impl_construct_type!(usize);
impl_construct_type!(i8);
impl_construct_type!(i16);
impl_construct_type!(i32);
impl_construct_type!(i64);
impl_construct_type!(isize);
impl_construct_type!(f32);
impl_construct_type!(f64);
impl_construct_type!(bool);
impl_construct_type!(char);
impl_construct_type!(*mut c_void);

unsafe impl<U: ConstructType> ConstructType for *mut U {
    fn base_type<'target, T>(target: &T) -> Value<'target, 'static>
    where
        T: Target<'target>,
    {
        UnionAll::pointer_type(target).as_value()
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
                let ptr_ua = UnionAll::pointer_type(&target);
                let inner_ty = U::construct_type(frame.as_extended_target());
                let params = &mut [inner_ty];
                let param_ptr = params.as_mut_ptr().cast();

                unsafe {
                    let applied = jl_apply_type(ptr_ua.unwrap(Private).cast(), param_ptr, 1);
                    debug_assert!(!applied.is_null());
                    let val = Value::wrap_non_null(NonNull::new_unchecked(applied), Private);
                    debug_assert!(val.is::<DataType>());
                    let ty = val.cast_unchecked::<DataType>();
                    Ok(target.data_from_ptr(ty.unwrap_non_null(Private), Private))
                }
            })
            .unwrap()
    }
}
