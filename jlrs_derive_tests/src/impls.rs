use jlrs::prelude::*;

#[repr(C)]
#[jlrs(julia_type = "Main.BitsWithCustom.BitsCharBitsIntChar")]
#[derive(Copy, Clone, Debug, PartialEq, JuliaStruct, IntoJulia)]
pub struct BitsCharBitsIntChar {
    pub a: char,
    pub b: BitsIntChar,
}

#[repr(C)]
#[jlrs(julia_type = "Main.BitsWithCustom.BitsIntChar")]
#[derive(Copy, Clone, Debug, PartialEq, JuliaStruct, IntoJulia)]
pub struct BitsIntChar {
    pub a: i64,
    pub b: char,
}

#[repr(C)]
#[jlrs(julia_type = "Main.BitsWithTuples.BitsUInt8TupleInt32Int64")]
#[derive(Copy, Clone, Debug, PartialEq, JuliaStruct, IntoJulia)]
pub struct BitsUInt8TupleInt32Int64 {
    pub a: u8,
    pub b: ::jlrs::value::tuple::Tuple2<i32, i64>,
}

#[repr(C)]
#[jlrs(julia_type = "Main.BitsWithTuples.BitsUInt8TupleInt32TupleInt16UInt16")]
#[derive(Copy, Clone, Debug, PartialEq, JuliaStruct, IntoJulia)]
pub struct BitsUInt8TupleInt32TupleInt16UInt16 {
    pub a: u8,
    pub b: ::jlrs::value::tuple::Tuple2<i32, ::jlrs::value::tuple::Tuple2<i16, u16>>,
}

#[repr(C)]
#[jlrs(julia_type = "Main.MultiFieldBits.BitsCharFloat32Float64")]
#[derive(Copy, Clone, Debug, PartialEq, JuliaStruct, IntoJulia)]
pub struct BitsCharFloat32Float64 {
    pub a: char,
    pub b: f32,
    pub c: f64,
}

#[repr(C)]
#[jlrs(julia_type = "Main.MultiFieldBits.BitsIntBool")]
#[derive(Copy, Clone, Debug, PartialEq, JuliaStruct, IntoJulia)]
pub struct BitsIntBool {
    pub a: i64,
    pub b: bool,
}

#[repr(C)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeBool")]
#[derive(Copy, Clone, Debug, PartialEq, JuliaStruct, IntoJulia)]
pub struct BitsTypeBool {
    pub a: bool,
}

#[repr(C)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeChar")]
#[derive(Copy, Clone, Debug, PartialEq, JuliaStruct, IntoJulia)]
pub struct BitsTypeChar {
    pub a: char,
}

#[repr(C)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeFloat32")]
#[derive(Copy, Clone, Debug, PartialEq, JuliaStruct, IntoJulia)]
pub struct BitsTypeFloat32 {
    pub a: f32,
}

#[repr(C)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeFloat64")]
#[derive(Copy, Clone, Debug, PartialEq, JuliaStruct, IntoJulia)]
pub struct BitsTypeFloat64 {
    pub a: f64,
}

#[repr(C)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeInt")]
#[derive(Copy, Clone, Debug, PartialEq, JuliaStruct, IntoJulia)]
pub struct BitsTypeInt {
    pub a: i64,
}

#[repr(C)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeInt16")]
#[derive(Copy, Clone, Debug, PartialEq, JuliaStruct, IntoJulia)]
pub struct BitsTypeInt16 {
    pub a: i16,
}

#[repr(C)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeInt32")]
#[derive(Copy, Clone, Debug, PartialEq, JuliaStruct, IntoJulia)]
pub struct BitsTypeInt32 {
    pub a: i32,
}

#[repr(C)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeInt64")]
#[derive(Copy, Clone, Debug, PartialEq, JuliaStruct, IntoJulia)]
pub struct BitsTypeInt64 {
    pub a: i64,
}

#[repr(C)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeInt8")]
#[derive(Copy, Clone, Debug, PartialEq, JuliaStruct, IntoJulia)]
pub struct BitsTypeInt8 {
    pub a: i8,
}

#[repr(C)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeUInt")]
#[derive(Copy, Clone, Debug, PartialEq, JuliaStruct, IntoJulia)]
pub struct BitsTypeUInt {
    pub a: u64,
}

#[repr(C)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeUInt16")]
#[derive(Copy, Clone, Debug, PartialEq, JuliaStruct, IntoJulia)]
pub struct BitsTypeUInt16 {
    pub a: u16,
}

#[repr(C)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeUInt32")]
#[derive(Copy, Clone, Debug, PartialEq, JuliaStruct, IntoJulia)]
pub struct BitsTypeUInt32 {
    pub a: u32,
}

#[repr(C)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeUInt64")]
#[derive(Copy, Clone, Debug, PartialEq, JuliaStruct, IntoJulia)]
pub struct BitsTypeUInt64 {
    pub a: u64,
}

#[repr(C)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeUInt8")]
#[derive(Copy, Clone, Debug, PartialEq, JuliaStruct, IntoJulia)]
pub struct BitsTypeUInt8 {
    pub a: u8,
}
