//! Support for sharing primitive data with Julia.

use crate::context::AllocationContext;
use crate::error::JlrsResult;
use crate::traits::{Allocate, IntoPrimitive};
use jl_sys::{
    jl_box_bool, jl_box_char, jl_box_float32, jl_box_float64, jl_box_int16, jl_box_int32,
    jl_box_int64, jl_box_int8, jl_box_uint16, jl_box_uint32, jl_box_uint64, jl_box_uint8,
    jl_value_t,
};

/// A wrapper enum that contains a value that can be copied to Julia. You should never have to use
/// this directly in your code.
#[derive(Copy, Clone)]
pub enum Primitive {
    Bool(bool),
    Char(char),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    Usize(usize),
    Isize(isize),
}

impl Allocate for Primitive {
    unsafe fn allocate(&self, _: AllocationContext) -> JlrsResult<*mut jl_value_t> {
        let v = match self {
            Primitive::Bool(v) => jl_box_bool(*v as i8),
            Primitive::Char(v) => jl_box_char(*v as u32),
            Primitive::U8(v) => jl_box_uint8(*v),
            Primitive::U16(v) => jl_box_uint16(*v),
            Primitive::U32(v) => jl_box_uint32(*v),
            Primitive::U64(v) => jl_box_uint64(*v),
            Primitive::I8(v) => jl_box_int8(*v),
            Primitive::I16(v) => jl_box_int16(*v),
            Primitive::I32(v) => jl_box_int32(*v),
            Primitive::I64(v) => jl_box_int64(*v),
            Primitive::F32(v) => jl_box_float32(*v),
            Primitive::F64(v) => jl_box_float64(*v),
            Primitive::Usize(v) => {
                if std::mem::size_of::<usize>() == std::mem::size_of::<u32>() {
                    jl_box_uint32(*v as _)
                } else {
                    jl_box_uint64(*v as _)
                }
            }
            Primitive::Isize(v) => {
                if std::mem::size_of::<isize>() == std::mem::size_of::<i32>() {
                    jl_box_int32(*v as _)
                } else {
                    jl_box_int64(*v as _)
                }
            }
        };

        Ok(v)
    }
}

macro_rules! impl_into_primitive {
    ($type:ty, $var:path) => {
        impl IntoPrimitive for $type {
            fn into_primitive(&self) -> Primitive {
                $var(*self)
            }
        }
    };
}

impl_into_primitive!(bool, Primitive::Bool);
impl_into_primitive!(char, Primitive::Char);
impl_into_primitive!(u8, Primitive::U8);
impl_into_primitive!(u16, Primitive::U16);
impl_into_primitive!(u32, Primitive::U32);
impl_into_primitive!(u64, Primitive::U64);
impl_into_primitive!(i8, Primitive::I8);
impl_into_primitive!(i16, Primitive::I16);
impl_into_primitive!(i32, Primitive::I32);
impl_into_primitive!(i64, Primitive::I64);
impl_into_primitive!(f32, Primitive::F32);
impl_into_primitive!(f64, Primitive::F64);
impl_into_primitive!(usize, Primitive::Usize);
impl_into_primitive!(isize, Primitive::Isize);
