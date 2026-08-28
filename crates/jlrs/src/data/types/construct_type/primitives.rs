//! Tpe constructor implementations for primitive types.

use std::{ffi::c_void, ptr::NonNull};

use jl_sys::{
    jl_bool_type, jl_char_type, jl_float32_type, jl_float64_type, jl_int8_type, jl_int16_type,
    jl_int32_type, jl_int64_type, jl_pointer_type, jl_uint8_type, jl_uint16_type, jl_uint32_type,
    jl_uint64_type, jl_value_t, jl_voidpointer_type,
};

use crate::{
    data::{
        managed::{
            Managed,
            union_all::UnionAll,
            value::{Value, ValueData},
        },
        types::construct_type::{ConstructType, TypeVarEnv},
    },
    memory::{scope::LocalScopeExt, target::Target},
};

macro_rules! impl_construct_julia_type_primitive {
    ($ty:ty, $jl_ty:ident) => {
        unsafe impl $crate::data::types::construct_type::ConstructType for $ty {
            type Static = $ty;

            const CACHEABLE: bool = false;

            #[inline]
            fn construct_type_uncached<'target, Tgt>(
                target: Tgt,
            ) -> $crate::data::managed::value::ValueData<'target, 'static, Tgt>
            where
                Tgt: $crate::memory::target::Target<'target>,
            {
                unsafe {
                    let ptr = ::std::ptr::NonNull::new_unchecked($jl_ty.cast::<::jl_sys::jl_value_t>());
                    target.data_from_ptr(ptr, $crate::private::Private)
                }
            }

            #[inline]
            fn construct_type_with_env_uncached<'target, Tgt>(
                target: Tgt,
                _env: &$crate::data::types::construct_type::type_var::TypeVarEnv,
            ) -> $crate::data::managed::value::ValueData<'target, 'static, Tgt>
            where
                Tgt: $crate::memory::target::Target<'target>,
            {
                unsafe {
                    let ptr = ::std::ptr::NonNull::new_unchecked($jl_ty.cast::<::jl_sys::jl_value_t>());
                    target.data_from_ptr(ptr, $crate::private::Private)
                }
            }

            #[inline]
            fn base_type<'target, Tgt>(_target: &Tgt) -> Option<$crate::data::managed::value::Value<'target, 'static>>
            where
                Tgt: $crate::memory::target::Target<'target>,
            {
                unsafe {
                    let ptr = ::std::ptr::NonNull::new_unchecked($jl_ty.cast::<::jl_sys::jl_value_t>());
                    Some(
                        <$crate::data::managed::value::Value as $crate::data::managed::private::ManagedPriv>::wrap_non_null(
                            ptr,
                            $crate::private::Private,
                        ),
                    )
                }
            }
        }
    };
}

impl_construct_julia_type_primitive!(u8, jl_uint8_type);
impl_construct_julia_type_primitive!(u16, jl_uint16_type);
impl_construct_julia_type_primitive!(u32, jl_uint32_type);
impl_construct_julia_type_primitive!(u64, jl_uint64_type);

#[cfg(target_pointer_width = "64")]
impl_construct_julia_type_primitive!(usize, jl_uint64_type);
#[cfg(target_pointer_width = "32")]
impl_construct_julia_type_primitive!(usize, jl_uint32_type);

impl_construct_julia_type_primitive!(i8, jl_int8_type);
impl_construct_julia_type_primitive!(i16, jl_int16_type);
impl_construct_julia_type_primitive!(i32, jl_int32_type);
impl_construct_julia_type_primitive!(i64, jl_int64_type);

#[cfg(target_pointer_width = "64")]
impl_construct_julia_type_primitive!(isize, jl_int64_type);
#[cfg(target_pointer_width = "32")]
impl_construct_julia_type_primitive!(isize, jl_int32_type);

impl_construct_julia_type_primitive!(f32, jl_float32_type);
impl_construct_julia_type_primitive!(f64, jl_float64_type);

impl_construct_julia_type_primitive!(bool, jl_bool_type);
impl_construct_julia_type_primitive!(char, jl_char_type);

impl_construct_julia_type_primitive!(*mut c_void, jl_voidpointer_type);

unsafe impl<U: ConstructType> ConstructType for *mut U {
    type Static = *mut U::Static;

    fn construct_type_uncached<'target, Tgt>(target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, 1>(|target, mut frame| {
            let ty = U::construct_type(&mut frame);
            unsafe {
                UnionAll::pointer_type(&frame)
                    .as_value()
                    .apply_type_unchecked(target, [ty])
            }
        })
    }

    #[inline]
    fn base_type<'target, Tgt>(_target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        unsafe {
            let ptr = NonNull::new_unchecked(jl_pointer_type.cast::<jl_value_t>());
            Some(
                <Value as crate::data::managed::private::ManagedPriv>::wrap_non_null(
                    ptr,
                    crate::private::Private,
                ),
            )
        }
    }

    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        env: &TypeVarEnv,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, 1>(|target, mut frame| {
            let ty = U::construct_type_with_env(&mut frame, env);
            unsafe {
                UnionAll::pointer_type(&frame)
                    .as_value()
                    .apply_type_unchecked(target, [ty])
            }
        })
    }
}
