#![allow(unused)]

use std::mem::size_of;

use crate::{
    data::types::{
        abstract_type::{AbstractChar, AbstractFloat, Integer, Signed, Unsigned},
        construct_type::ConstructType,
    },
    inline_static_ref,
    prelude::{Target, Value},
};

pub trait PrimitiveType: Copy {
    const N_BITS: usize;
    type Super: ConstructType;
}

pub trait IntegerType: PrimitiveType {}

macro_rules! impl_primitive_type {
    ($ty:ty, $super:ty) => {
        impl PrimitiveType for $ty {
            const N_BITS: usize = size_of::<Self>();
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

impl IntegerType for u8 {}
impl IntegerType for u16 {}
impl IntegerType for u32 {}
impl IntegerType for u64 {}
impl IntegerType for usize {}
impl IntegerType for i8 {}
impl IntegerType for i16 {}
impl IntegerType for i32 {}
impl IntegerType for i64 {}
impl IntegerType for isize {}

impl_primitive_type!(bool, Integer);
impl_primitive_type!(super::bool::Bool, Integer);

impl IntegerType for bool {}
impl IntegerType for char {}

impl_primitive_type!(char, AbstractChar);
impl_primitive_type!(super::char::Char, AbstractChar);

impl IntegerType for super::bool::Bool {}
impl IntegerType for super::char::Char {}

// impl_primitive_type!(f16, AbstractFloat);
impl_primitive_type!(f32, AbstractFloat);
impl_primitive_type!(f64, AbstractFloat);

pub unsafe trait Enum {
    type Super: IntegerType;
    fn as_value<'target, Tgt: Target<'target>>(&self, _: &Tgt) -> Value<'target, 'static>;
    fn as_super(&self) -> Self::Super;
}
