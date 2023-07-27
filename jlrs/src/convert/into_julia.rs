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

use std::{ffi::c_void, ptr::NonNull};

use jl_sys::{
    jl_apply_type, jl_bool_type, jl_box_bool, jl_box_char, jl_box_float32, jl_box_float64,
    jl_box_int16, jl_box_int32, jl_box_int64, jl_box_int8, jl_box_uint16, jl_box_uint32,
    jl_box_uint64, jl_box_uint8, jl_box_voidpointer, jl_char_type, jl_float32_type,
    jl_float64_type, jl_int16_type, jl_int32_type, jl_int64_type, jl_int8_type,
    jl_new_struct_uninit, jl_uint16_type, jl_uint32_type, jl_uint64_type, jl_uint8_type,
    jl_voidpointer_type,
};

use crate::{
    data::managed::{
        datatype::{DataType, DataTypeData},
        private::ManagedPriv,
        union_all::UnionAll,
        value::{Value, ValueData},
    },
    memory::target::Target,
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
pub unsafe trait IntoJulia: Sized + 'static {
    /// Returns the associated Julia type of the implementor.
    ///
    /// The layout of that type and the Rust type must match exactly, and it must be an `isbits`
    /// type, otherwise this trait has been implemented incorrectly.
    fn julia_type<'scope, T>(target: T) -> DataTypeData<'scope, T>
    where
        T: Target<'scope>;

    #[doc(hidden)]
    #[inline]
    fn into_julia<'scope, T>(self, target: T) -> ValueData<'scope, 'static, T>
    where
        T: Target<'scope>,
    {
        // Safety: trait is implemented incorrectly if this is incorrect. A new instance of the
        // associated
        unsafe {
            let ty = Self::julia_type(&target).as_managed();
            debug_assert!(ty.is_bits());

            let instance = ty.instance();
            if instance.is_none() {
                let container = jl_new_struct_uninit(ty.unwrap(Private));
                container.cast::<Self>().write(self);
                target.data_from_ptr(NonNull::new_unchecked(container), Private)
            } else {
                target.data_from_ptr(instance.unwrap().unwrap_non_null(Private), Private)
            }
        }
    }
}

macro_rules! impl_into_julia {
    ($type:ty, $boxer:ident, $julia_type:expr) => {
        // Safety: These implemetations use a boxing function provided by Julia
        unsafe impl IntoJulia for $type {
            #[inline]
            fn julia_type<'scope, T>(
                target: T,
            ) -> $crate::data::managed::datatype::DataTypeData<'scope, T>
            where
                T: $crate::memory::target::Target<'scope>,
            {
                unsafe {
                    target.data_from_ptr(
                        ::std::ptr::NonNull::new_unchecked($julia_type),
                        $crate::private::Private,
                    )
                }
            }

            #[inline]
            fn into_julia<'scope, T>(
                self,
                target: T,
            ) -> $crate::data::managed::value::ValueData<'scope, 'static, T>
            where
                T: $crate::memory::target::Target<'scope>,
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
impl_into_julia!(usize, jl_box_uint32, jl_uint32_type);

#[cfg(target_pointer_width = "64")]
impl_into_julia!(usize, jl_box_uint64, jl_uint64_type);

#[cfg(target_pointer_width = "32")]
impl_into_julia!(isize, jl_box_int32, jl_int32_type);

#[cfg(target_pointer_width = "64")]
impl_into_julia!(isize, jl_box_int64, jl_int64_type);

// Safety: *mut T and Ptr{T} have the same layout
unsafe impl<U: IntoJulia> IntoJulia for *mut U {
    #[inline]
    fn julia_type<'scope, T>(target: T) -> DataTypeData<'scope, T>
    where
        T: Target<'scope>,
    {
        let ptr_ua = UnionAll::pointer_type(&target);
        let inner_ty = U::julia_type(&target);
        let params = &mut [inner_ty];
        let param_ptr = params.as_mut_ptr().cast();

        // Safety: Not rooting the result is fine. The result must be a concrete type which is
        // globally rooted.
        unsafe {
            let applied = jl_apply_type(ptr_ua.unwrap(Private).cast(), param_ptr, 1);
            debug_assert!(!applied.is_null());
            let val = Value::wrap_non_null(NonNull::new_unchecked(applied), Private);
            debug_assert!(val.is::<DataType>());
            let ty = val.cast_unchecked::<DataType>();
            debug_assert!(ty.is_concrete_type());
            target.data_from_ptr(ty.unwrap_non_null(Private), Private)
        }
    }
}
