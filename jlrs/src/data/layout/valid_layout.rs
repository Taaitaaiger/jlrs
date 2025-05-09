//! Traits to check if a Rust type and a Julia type have matching layouts.
//!
//! When working with Julia values, it's always possible to access their [`DataType`]. This
//! `DataType` contains all information about the value's fields and their layout. The
//! [`ValidLayout`] and [`ValidField`] traits defined in this module are used to check if a type
//! has the same layout as a given Julia type. It is implemented automatically by JlrsReflect.jl,
//! you should not implement it manually.
//!
//! [`DataType`]: crate::data::managed::datatype::DataType

use std::ffi::c_void;

use jl_sys::{
    jl_bool_type, jl_char_type, jl_float32_type, jl_float64_type, jl_int16_type, jl_int32_type,
    jl_int64_type, jl_int8_type, jl_uint16_type, jl_uint32_type, jl_uint64_type, jl_uint8_type,
    jl_voidpointer_type,
};

use crate::{
    convert::into_julia::IntoJulia,
    data::managed::{datatype::DataType, union_all::UnionAll, value::Value},
    prelude::{Managed, Target},
};

/// Trait used to check if a Rust type and Julia type have matching layouts.
///
/// All layouts generated by JlrsReflect.jl derive this trait. In this case all fields are
/// checked recursively to determine if the value can be unboxed as that type.
///
/// Safety: implementations of [`ValidLayout::valid_layout`] must not trigger the GC. This means
/// no function can be called that allocates Julia data, calls Julia functions, or can trigger the
/// GC some other way.
#[diagnostic::on_unimplemented(
    message = "the trait bound `{Self}: ValidLayout` is not satisfied",
    label = "the trait `ValidLayout` is not implemented for `{Self}`",
    note = "Custom types that implement `ValidLayout` should be generated with JlrsCore.reflect",
    note = "Do not implement `ForeignType` or `OpaqueType` unless this type is exported to Julia with `julia_module!`"
)]
pub unsafe trait ValidLayout {
    /// Must be `true` if the Rust type is a managed type.
    const IS_REF: bool = false;

    /// Check if the layout of the implementor is compatible with the layout of `ty`.
    ///
    /// The argument is a `Value` to account for the fact that a field type can be a `Union`,
    /// `UnionAll` or `Union{}`. Calling this method will not trigger the GC.
    fn valid_layout(ty: Value) -> bool;

    /// Returns the type object this layout is associated with.
    fn type_object<'target, Tgt: Target<'target>>(target: &Tgt) -> Value<'target, 'static>;
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_valid_layout {
    ($t:ty, $type_obj:ident) => {
        unsafe impl $crate::data::layout::valid_layout::ValidLayout for $t {
            #[inline]
            fn valid_layout(v: $crate::data::managed::value::Value) -> bool {
                if v.is::<$crate::data::managed::datatype::DataType>() {
                    let dt = unsafe { v.cast_unchecked::<$crate::data::managed::datatype::DataType>() };
                    dt.is::<$t>()
                } else {
                    false
                }
            }

            #[inline]
            fn type_object<'target, Tgt>(
                _: &Tgt
            ) -> $crate::data::managed::value::Value<'target, 'static>
            where
                Tgt: $crate::memory::target::Target<'target>
            {
                unsafe {
                    <$crate::data::managed::value::Value as $crate::data::managed::private::ManagedPriv>::wrap_non_null(
                        std::ptr::NonNull::new_unchecked($type_obj.cast()),
                        $crate::private::Private
                    )
                }
            }

            const IS_REF: bool = false;
        }

        unsafe impl $crate::data::layout::valid_layout::ValidField for $t {
            #[inline]
            fn valid_field(v: $crate::data::managed::value::Value) -> bool {
                if v.is::<$crate::data::managed::datatype::DataType>() {
                    let dt = unsafe { v.cast_unchecked::<$crate::data::managed::datatype::DataType>() };
                    dt.is::<$t>()
                } else {
                    false
                }
            }
        }
    }
}

impl_valid_layout!(bool, jl_bool_type);
impl_valid_layout!(char, jl_char_type);
impl_valid_layout!(i8, jl_int8_type);
impl_valid_layout!(i16, jl_int16_type);
impl_valid_layout!(i32, jl_int32_type);
impl_valid_layout!(i64, jl_int64_type);
#[cfg(target_pointer_width = "64")]
impl_valid_layout!(isize, jl_int64_type);
#[cfg(target_pointer_width = "32")]
impl_valid_layout!(isize, jl_int32_type);
impl_valid_layout!(u8, jl_uint8_type);
impl_valid_layout!(u16, jl_uint16_type);
impl_valid_layout!(u32, jl_uint32_type);
impl_valid_layout!(u64, jl_uint64_type);
#[cfg(target_pointer_width = "64")]
impl_valid_layout!(usize, jl_uint64_type);
#[cfg(target_pointer_width = "32")]
impl_valid_layout!(usize, jl_uint32_type);
impl_valid_layout!(f32, jl_float32_type);
impl_valid_layout!(f64, jl_float64_type);
impl_valid_layout!(*mut c_void, jl_voidpointer_type);

unsafe impl<T: IntoJulia> ValidLayout for *mut T {
    #[inline]
    fn valid_layout(v: Value) -> bool {
        if v.is::<DataType>() {
            let dt = unsafe { v.cast_unchecked::<DataType>() };
            dt.is::<*mut T>()
        } else {
            false
        }
    }

    #[inline]
    fn type_object<'target, Tgt: Target<'target>>(target: &Tgt) -> Value<'target, 'static> {
        UnionAll::pointer_type(target).as_value()
    }

    const IS_REF: bool = false;
}

/// Trait used to check if a field of a Rust type and Julia type have matching layouts.
///
/// Layouts for immutable types generated by JlrsReflect.jl derive this trait. Mutable types
/// must use `Option<WeakValue>` because they're not stored inline when used as a field type.
pub unsafe trait ValidField {
    /// Returns `true` if `Self` is the correct representation for Julia data of type `ty`
    /// when it's used as a field type.
    fn valid_field(ty: Value) -> bool;
}
