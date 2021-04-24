//! Trait to check if a Rust type and a Julia type have matching layouts.

use crate::value::Value;

/// Trait implemented as part of `JuliaStruct` that is used to verify this type has the same
/// layout as the Julia value.
pub unsafe trait ValidLayout {
    #[doc(hidden)]
    // NB: the type is passed as a value to account for DataTypes, UnionAlls and Unions.
    unsafe fn valid_layout(ty: Value) -> bool;
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_valid_layout {
    ($type:ty, $($lt:lifetime),+) => {
        unsafe impl<$($lt),+> $crate::layout::valid_layout::ValidLayout for $type {
            unsafe fn valid_layout(v: $crate::value::Value) -> bool {
                if let Ok(dt) =  v.cast::<$crate::value::datatype::DataType>() {
                    dt.is::<$type>()
                } else {
                    false
                }
            }
        }
    };
    ($t:ty) => {
        unsafe impl $crate::layout::valid_layout::ValidLayout for $t {
            unsafe fn valid_layout(v: $crate::value::Value) -> bool {
                if let Ok(dt) =  v.cast::<$crate::value::datatype::DataType>() {
                    dt.is::<$t>()
                } else {
                    false
                }
            }
        }
    }
}

impl_valid_layout!(bool);
impl_valid_layout!(char);
impl_valid_layout!(i8);
impl_valid_layout!(i16);
impl_valid_layout!(i32);
impl_valid_layout!(i64);
impl_valid_layout!(isize);
impl_valid_layout!(u8);
impl_valid_layout!(u16);
impl_valid_layout!(u32);
impl_valid_layout!(u64);
impl_valid_layout!(usize);
impl_valid_layout!(f32);
impl_valid_layout!(f64);
