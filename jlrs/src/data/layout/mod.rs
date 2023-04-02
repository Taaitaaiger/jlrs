//! Type and field layouts of Julia data.
//!
//! The layout of an instance of a Julia type depends on the types of its fields. There are
//! essentially three ways a field is represented in the layout of the containing type: inline,
//! as a reference to Julia data, or as a bits union.
//!
//! As a rule of thumb you can assume a field whose type is a concrete, immutable and not a union
//! type is stored inline; a field that is a union of immutable types, none of which contain
//! references to Julia data, is stored as a bits union; all other types are stored as references
//! to Julia data. Due to these different storage modes, a valid layout for a Julia type isn't
//! necessarily a valid layout for a field of that type. The [`ValidLayout`] and [`ValidField`]
//! are available to handle the distinction.
//!
//! You shouldn't implement layouts for Julia types manually, but rather use the functionality
//! from the `Jlrs.Reflect` module to generate them and derive all applicable traits.
//!
//! [`ValidLayout`]: crate::data::layout::valid_layout::ValidLayout
//! [`ValidField`]: crate::data::layout::valid_layout::ValidField

macro_rules! impl_ccall_arg {
    ($ty:ident) => {
        unsafe impl $crate::convert::ccall_types::CCallArg for $ty {
            type CCallArgType = Self;
            type FunctionArgType = Self;
        }
    };
}

macro_rules! impl_construct_julia_type {
    ($ty:ty, $jl_ty:ident) => {
        unsafe impl $crate::data::types::construct_type::ConstructType for $ty {
            fn construct_type<'target, T>(
                target: $crate::memory::target::ExtendedTarget<'target, '_, '_, T>,
            ) -> $crate::data::managed::value::ValueData<'target, 'static, T>
            where
                T: $crate::memory::target::Target<'target>,
            {
                let (target, _) = target.split();
                unsafe {
                    let ptr =
                        ::std::ptr::NonNull::new_unchecked($jl_ty.cast::<::jl_sys::jl_value_t>());
                    target.data_from_ptr(ptr, $crate::private::Private)
                }
            }

            fn base_type<'target, Tgt>(_target: &Tgt) -> Option<$crate::data::managed::value::Value<'target, 'static>>
            where
                Tgt: $crate::memory::target::Target<'target>,
            {
                unsafe {
                    let ptr =
                        ::std::ptr::NonNull::new_unchecked($jl_ty.cast::<::jl_sys::jl_value_t>());
                    Some(<$crate::data::managed::value::Value as $crate::data::managed::private::ManagedPriv>::wrap_non_null(ptr, $crate::private::Private))
                }
            }
        }
    };
}

pub mod bool;
pub mod char;
#[cfg(feature = "f16")]
pub mod f16;
pub mod nothing;
#[cfg(feature = "internal-types")]
pub mod ssa_value;
pub mod tuple;
pub mod union;
pub mod valid_layout;
