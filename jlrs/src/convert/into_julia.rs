//! Convert simple data from Rust to Julia.
//!
//! In order to use data from Rust in Julia it must first be converted to a [`Value`]. The
//! easiest way to do this is by calling [`Value::new`], which is compatible with types that
//! implement the [`IntoJulia`] trait defined in this module. This trait only supports
//! isbits-types, and should not be implemented manually. Rather, you should use JlrsReflect.jl to
//! automatically derive it for compatible types.
//!
//! [`Value::new`]: crate::data::managed::value::Value::new
//! [`Value`]: crate::data::managed::value::Value

use std::{ffi::c_void, mem::MaybeUninit, ptr::NonNull};

use jl_sys::{
    jl_bool_type, jl_box_bool, jl_box_char, jl_box_float32, jl_box_float64, jl_box_int16,
    jl_box_int32, jl_box_int64, jl_box_int8, jl_box_uint16, jl_box_uint32, jl_box_uint64,
    jl_box_uint8, jl_box_voidpointer, jl_char_type, jl_float32_type, jl_float64_type,
    jl_int16_type, jl_int32_type, jl_int64_type, jl_int8_type, jl_new_struct_uninit,
    jl_uint16_type, jl_uint32_type, jl_uint64_type, jl_uint8_type, jl_voidpointer_type,
    jlrs_box_long, jlrs_box_ulong,
};

use crate::{
    data::{
        managed::{
            datatype::{DataType, DataTypeData},
            private::ManagedPriv,
            union_all::UnionAll,
            value::ValueData,
        },
        types::construct_type::ConstructType,
    },
    memory::target::Target,
    prelude::Managed,
    private::Private,
};

/// Trait implemented by types that can be converted to a Julia value in combination with
/// [`Value::new`]. This trait can be derived, it's recommended to use JlrsReflect.jl to
/// ensure it's implemented correctly.
///
/// If you do choose to implement it manually, you only need to implement the `julia_type` method
/// which must return the `DataType` of the type this data will have in Julia. The layout of this
/// type and the type in Rust must match exactly. Incompatible layouts will cause undefined
/// behavior. The type in Rust must always be `#[repr(C)]`. The `DataType` must be an isbits-type.
///
/// [`Value::new`]: crate::data::managed::value::Value::new
#[cfg_attr(
    feature = "diagnostics",
    diagnostic::on_unimplemented(
        message = "the trait bound `{Self}: IntoJulia` is not satisfied",
        label = "the trait `IntoJulia` is not implemented for `{Self}`",
        note = "Custom types that implement `IntoJulia` should be generated with JlrsCore.reflect",
        note = "Do not implement `ForeignType`, `OpaqueType`, or `ParametricVariant` unless this type is exported to Julia with `julia_module!`"
    )
)]
pub unsafe trait IntoJulia: Sized + 'static {
    /// Returns the associated Julia type of the implementor.
    ///
    /// The layout of that type and the Rust type must match exactly, and it must be an `isbits`
    /// type, otherwise this trait has been implemented incorrectly.
    fn julia_type<'scope, Tgt>(target: Tgt) -> DataTypeData<'scope, Tgt>
    where
        Tgt: Target<'scope>;

    #[doc(hidden)]
    #[inline]
    fn into_julia<'scope, Tgt>(self, target: Tgt) -> ValueData<'scope, 'static, Tgt>
    where
        Tgt: Target<'scope>,
    {
        // Safety: trait is implemented incorrectly if this is incorrect. A new instance of the
        // associated
        unsafe {
            let ty = Self::julia_type(&target).as_managed();
            debug_assert!(ty.is_bits());

            if let Some(instance) = ty.instance() {
                target.data_from_ptr(instance.unwrap_non_null(Private), Private)
            } else {
                let container = jl_new_struct_uninit(ty.unwrap(Private));
                debug_assert!(!container.is_null());
                let container = NonNull::new_unchecked(container);
                container.cast::<MaybeUninit<Self>>().as_mut().write(self);
                target.data_from_ptr(container, Private)
            }
        }
    }
}

macro_rules! impl_into_julia {
    ($type:ty, $boxer:ident, $julia_type:expr) => {
        // Safety: These implemetations use a boxing function provided by Julia
        unsafe impl IntoJulia for $type {
            #[inline]
            fn julia_type<'scope, Tgt>(
                target: Tgt,
            ) -> $crate::data::managed::datatype::DataTypeData<'scope, Tgt>
            where
                Tgt: $crate::memory::target::Target<'scope>,
            {
                unsafe {
                    target.data_from_ptr(
                        ::std::ptr::NonNull::new_unchecked($julia_type),
                        $crate::private::Private,
                    )
                }
            }

            #[inline]
            fn into_julia<'scope, Tgt>(
                self,
                target: Tgt,
            ) -> $crate::data::managed::value::ValueData<'scope, 'static, Tgt>
            where
                Tgt: $crate::memory::target::Target<'scope>,
            {
                unsafe {
                    target.data_from_ptr(
                        ::std::ptr::NonNull::new_unchecked($boxer(self as _)),
                        $crate::private::Private,
                    )
                }
            }
        }
    };
}

impl_into_julia!(bool, jl_box_bool, jl_bool_type);
impl_into_julia!(char, jl_box_char, jl_char_type);
impl_into_julia!(u8, jl_box_uint8, jl_uint8_type);
impl_into_julia!(u16, jl_box_uint16, jl_uint16_type);
impl_into_julia!(u32, jl_box_uint32, jl_uint32_type);
impl_into_julia!(u64, jl_box_uint64, jl_uint64_type);
impl_into_julia!(i8, jl_box_int8, jl_int8_type);
impl_into_julia!(i16, jl_box_int16, jl_int16_type);
impl_into_julia!(i32, jl_box_int32, jl_int32_type);
impl_into_julia!(i64, jl_box_int64, jl_int64_type);
impl_into_julia!(f32, jl_box_float32, jl_float32_type);
impl_into_julia!(f64, jl_box_float64, jl_float64_type);
impl_into_julia!(*mut c_void, jl_box_voidpointer, jl_voidpointer_type);

#[cfg(target_pointer_width = "32")]
impl_into_julia!(usize, jlrs_box_ulong, jl_uint32_type);

#[cfg(target_pointer_width = "64")]
impl_into_julia!(usize, jlrs_box_ulong, jl_uint64_type);

#[cfg(target_pointer_width = "32")]
impl_into_julia!(isize, jlrs_box_long, jl_int32_type);

#[cfg(target_pointer_width = "64")]
impl_into_julia!(isize, jlrs_box_long, jl_int64_type);

// Safety: *mut T and Ptr{T} have the same layout
unsafe impl<T: IntoJulia + ConstructType> IntoJulia for *mut T {
    #[inline]
    fn julia_type<'scope, Tgt>(target: Tgt) -> DataTypeData<'scope, Tgt>
    where
        Tgt: Target<'scope>,
    {
        target.with_local_scope::<_, _, 1>(|target, mut frame| {
            let ptr_ua = UnionAll::pointer_type(&frame);
            let inner_ty = T::construct_type(&mut frame);

            unsafe {
                let ty = ptr_ua.apply_types_unchecked(&frame, [inner_ty]).as_value();
                debug_assert!(ty.is::<DataType>());
                let ty = ty.cast_unchecked::<DataType>();
                debug_assert!(ty.is_concrete_type());
                ty.root(target)
            }
        })
    }
}
