//! Convert simple data from Rust to Julia.
//!
//! In order to use data from Rust in Julia, it must first be converted to a [`Value`]. The
//! easiest way to do this is by using [`Value::new`], which is compatible with types that
//! implement the [`IntoJulia`] trait defined in this module. This trait only supports bits-types
//! with no type parameters, and should not be implemented manually. Rather, you should use
//! JlrsReflect.jl to automatically derive it for compatible types.
//!
//! [`Value::new`]: crate::wrappers::ptr::value::Value::new
//! [`Value`]: crate::wrappers::ptr::value::Value

use crate::{
    memory::global::Global,
    private::Private,
    wrappers::ptr::{
        datatype::DataType, private::Wrapper as WrapperPriv, union_all::UnionAll, value::Value,
        DataTypeRef, ValueRef, Wrapper,
    },
};
use jl_sys::{
    jl_apply_type, jl_bool_type, jl_box_bool, jl_box_char, jl_box_float32, jl_box_float64,
    jl_box_int16, jl_box_int32, jl_box_int64, jl_box_int8, jl_box_uint16, jl_box_uint32,
    jl_box_uint64, jl_box_uint8, jl_box_voidpointer, jl_char_type, jl_float32_type,
    jl_float64_type, jl_int16_type, jl_int32_type, jl_int64_type, jl_int8_type,
    jl_new_struct_uninit, jl_uint16_type, jl_uint32_type, jl_uint64_type, jl_uint8_type,
    jl_voidpointer_type,
};
use std::{ffi::c_void, ptr::NonNull};

/// Trait implemented by types that can be converted to a Julia value in combination with
/// [`Value::new`]. This trait can be derived, it's recommended to use JlrsReflect.jl to
/// ensure it's implemented correctly.
///
/// If you do choose to implement it manually, you only need to implement the `julia_type` method
/// which must return the `DataType` of the type this data will have in Julia. The layout of this
/// type and the type in Rust must match exactly. Incompatible layouts will lead to UB. Note that
/// the type in Rust must always be `#[repr(C)]`.
///
/// [`Value::new`]: crate::wrappers::ptr::value::Value::new
pub unsafe trait IntoJulia: Sized + 'static {
    /// Returns the associated Julia type of the implementor. The layout of that type and the
    /// Rust type must match exactly, otherwise the trait is implemented incorrectly.
    fn julia_type<'scope>(_: Global<'scope>) -> DataTypeRef<'scope>;

    #[doc(hidden)]
    #[inline(always)]
    fn into_julia<'scope>(self, global: Global<'scope>) -> ValueRef<'scope, 'static> {
        unsafe {
            let ty = Self::julia_type(global)
                .wrapper()
                .expect("DataTypeRef::wrapper returned None");

            debug_assert!(ty.is_bits());

            let container = jl_new_struct_uninit(ty.unwrap(Private));
            container.cast::<Self>().write(self);

            ValueRef::wrap(container)
        }
    }
}

macro_rules! impl_into_julia {
    ($type:ty, $boxer:ident, $julia_type:expr) => {
        unsafe impl IntoJulia for $type {
            #[inline(always)]
            fn julia_type<'scope>(_: Global<'scope>) -> $crate::wrappers::ptr::DataTypeRef<'scope> {
                unsafe { $crate::wrappers::ptr::DataTypeRef::wrap($julia_type) }
            }

            #[inline(always)]
            fn into_julia<'scope>(
                self,
                _: Global<'scope>,
            ) -> $crate::wrappers::ptr::ValueRef<'scope, 'static> {
                unsafe { $crate::wrappers::ptr::ValueRef::wrap($boxer(self as _)) }
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

#[cfg(not(target_pointer_width = "64"))]
impl_into_julia!(usize, jl_box_uint32, jl_uint32_type);

#[cfg(target_pointer_width = "64")]
impl_into_julia!(usize, jl_box_uint64, jl_uint64_type);

#[cfg(not(target_pointer_width = "64"))]
impl_into_julia!(isize, jl_box_int32, jl_int32_type);

#[cfg(target_pointer_width = "64")]
impl_into_julia!(isize, jl_box_int64, jl_int64_type);

unsafe impl<T: IntoJulia> IntoJulia for *mut T {
    fn julia_type<'scope>(global: Global<'scope>) -> DataTypeRef<'scope> {
        let ptr_ua = UnionAll::pointer_type(global);
        let inner_ty = T::julia_type(global);
        let params = &mut [inner_ty];
        let param_ptr = params.as_mut_ptr().cast();

        unsafe {
            // Not rooting the result should be fine. The result must be a concrete type, which
            // means `applied` can't have any free type parameters, so it should be cached.
            let applied = jl_apply_type(ptr_ua.unwrap(Private).cast(), param_ptr, 1);
            debug_assert!(!applied.is_null());

            let val = Value::wrap_non_null(NonNull::new_unchecked(applied), Private);
            debug_assert!(val.is::<DataType>());

            val.cast_unchecked::<DataType>().as_ref()
        }
    }
}
