//! Primitive type trait

use super::{
    abstract_type::{AbstractChar, AbstractFloat, Integer, Signed, Unsigned},
    construct_type::ConstructType,
};
use crate::prelude::{Bool, Char};

/// A primitive type.
///
/// Safety: must only be implemented by types that are primitive types in Julia.
pub unsafe trait PrimitiveType: ConstructType {
    /// The size of an instance of this type in bits
    const N_BITS: usize;
    /// The super-type of this type
    type Super: ConstructType;
}

/// A primitive type.
///
/// Safety: must only be implemented by types that are primitive types in Julia and subtypes of
/// `Integer`.
pub unsafe trait IntegerType: PrimitiveType {}

macro_rules! impl_primitive_type {
    ($ty:ty, $super:ty) => {
        unsafe impl PrimitiveType for $ty {
            const N_BITS: usize = ::std::mem::size_of::<Self>();
            type Super = $super;
        }
    };
}

impl_primitive_type!(u8, Unsigned);
impl_primitive_type!(u16, Unsigned);
impl_primitive_type!(u32, Unsigned);
impl_primitive_type!(u64, Unsigned);
impl_primitive_type!(usize, Unsigned);
impl_primitive_type!(i8, Signed);
impl_primitive_type!(i16, Signed);
impl_primitive_type!(i32, Signed);
impl_primitive_type!(i64, Signed);
impl_primitive_type!(isize, Signed);

unsafe impl IntegerType for u8 {}
unsafe impl IntegerType for u16 {}
unsafe impl IntegerType for u32 {}
unsafe impl IntegerType for u64 {}
unsafe impl IntegerType for usize {}
unsafe impl IntegerType for i8 {}
unsafe impl IntegerType for i16 {}
unsafe impl IntegerType for i32 {}
unsafe impl IntegerType for i64 {}
unsafe impl IntegerType for isize {}

impl_primitive_type!(bool, Integer);
impl_primitive_type!(Bool, Integer);

unsafe impl IntegerType for bool {}
unsafe impl IntegerType for char {}

impl_primitive_type!(char, AbstractChar);
impl_primitive_type!(Char, AbstractChar);

unsafe impl IntegerType for Bool {}
unsafe impl IntegerType for Char {}

#[cfg(feature = "f16")]
impl_primitive_type!(half::f16, AbstractFloat);
impl_primitive_type!(f32, AbstractFloat);
impl_primitive_type!(f64, AbstractFloat);
