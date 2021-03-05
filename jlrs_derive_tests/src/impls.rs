use jlrs::prelude::*;

#[repr(C)]
#[jlrs(julia_type = "Main.BitsWithCustom.BitsCharBitsIntChar")]
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
pub struct BitsCharBitsIntChar {
    pub a: char,
    pub b: BitsIntChar,
}

#[repr(C)]
#[jlrs(julia_type = "Main.BitsWithCustom.BitsIntChar")]
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
pub struct BitsIntChar {
    pub a: i64,
    pub b: char,
}

#[repr(C)]
#[jlrs(julia_type = "Main.BitsWithTuples.BitsUInt8TupleInt32Int64")]
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
pub struct BitsUInt8TupleInt32Int64 {
    pub a: u8,
    pub b: ::jlrs::value::tuple::Tuple2<i32, i64>,
}

#[repr(C)]
#[jlrs(julia_type = "Main.BitsWithTuples.BitsUInt8TupleInt32TupleInt16UInt16")]
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
pub struct BitsUInt8TupleInt32TupleInt16UInt16 {
    pub a: u8,
    pub b: ::jlrs::value::tuple::Tuple2<i32, ::jlrs::value::tuple::Tuple2<i16, u16>>,
}

#[repr(C)]
#[jlrs(julia_type = "Main.MultiFieldBits.BitsCharFloat32Float64")]
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
pub struct BitsCharFloat32Float64 {
    pub a: char,
    pub b: f32,
    pub c: f64,
}

#[repr(C)]
#[jlrs(julia_type = "Main.MultiFieldBits.BitsIntBool")]
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
pub struct BitsIntBool {
    pub a: i64,
    pub b: bool,
}

#[repr(C)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeBool")]
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
pub struct BitsTypeBool {
    pub a: bool,
}

#[repr(C)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeChar")]
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
pub struct BitsTypeChar {
    pub a: char,
}

#[repr(C)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeFloat32")]
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
pub struct BitsTypeFloat32 {
    pub a: f32,
}

#[repr(C)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeFloat64")]
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
pub struct BitsTypeFloat64 {
    pub a: f64,
}

#[repr(C)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeInt")]
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
pub struct BitsTypeInt {
    pub a: i64,
}

#[repr(C)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeInt16")]
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
pub struct BitsTypeInt16 {
    pub a: i16,
}

#[repr(C)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeInt32")]
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
pub struct BitsTypeInt32 {
    pub a: i32,
}

#[repr(C)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeInt64")]
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
pub struct BitsTypeInt64 {
    pub a: i64,
}

#[repr(C)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeInt8")]
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
pub struct BitsTypeInt8 {
    pub a: i8,
}

#[repr(C)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeUInt")]
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
pub struct BitsTypeUInt {
    pub a: u64,
}

#[repr(C)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeUInt16")]
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
pub struct BitsTypeUInt16 {
    pub a: u16,
}

#[repr(C)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeUInt32")]
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
pub struct BitsTypeUInt32 {
    pub a: u32,
}

#[repr(C)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeUInt64")]
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
pub struct BitsTypeUInt64 {
    pub a: u64,
}

#[repr(C)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeUInt8")]
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
pub struct BitsTypeUInt8 {
    pub a: u8,
}

#[repr(C)]
#[jlrs(julia_type = "Main.WithBitsUnion.DoubleVariant")]
#[derive(Copy, Clone, Debug, JuliaStruct)]
pub struct DoubleVariant {
    pub a: i8,
    #[jlrs(bits_union_align)]
    _b_align: ::jlrs::value::union::Align4,
    #[jlrs(bits_union)]
    pub b: ::jlrs::value::union::BitsUnion<[::std::mem::MaybeUninit<u8>; 4]>,
    #[jlrs(bits_union_flag)]
    pub b_flag: u8,
    pub c: i8,
}

#[repr(C)]
#[jlrs(julia_type = "Main.WithBitsUnion.SingleVariant")]
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
pub struct SingleVariant {
    pub a: i8,
    pub b: i32,
    pub c: i8,
}

#[repr(C)]
#[jlrs(julia_type = "Main.WithBitsUnion.SizeAlignMismatch")]
#[derive(Copy, Clone, Debug, JuliaStruct)]
pub struct SizeAlignMismatch {
    pub a: i8,
    #[jlrs(bits_union_align)]
    _b_align: ::jlrs::value::union::Align4,
    #[jlrs(bits_union)]
    pub b: ::jlrs::value::union::BitsUnion<[::std::mem::MaybeUninit<u8>; 6]>,
    #[jlrs(bits_union_flag)]
    pub b_flag: u8,
    pub c: i8,
}

#[repr(C)]
#[jlrs(julia_type = "Main.WithBitsUnion.UnionInTuple")]
#[derive(Copy, Clone, Debug, JuliaStruct)]
pub struct UnionInTuple<'frame, 'data> {
    pub a: i8,
    pub b: ::jlrs::value::Value<'frame, 'data>,
    pub c: i8,
}

#[repr(C)]
#[jlrs(julia_type = "Main.WithGeneric.WithGenericT")]
#[derive(Copy, Clone, Debug, JuliaStruct)]
pub struct WithGenericT<T>
where
    T: ::jlrs::traits::ValidLayout + Copy,
{
    pub a: T,
}

#[repr(C)]
#[jlrs(julia_type = "Main.WithGeneric.WithGenericUnionAll")]
#[derive(Copy, Clone, Debug, JuliaStruct)]
pub struct WithGenericUnionAll<'frame, 'data> {
    pub a: ::jlrs::value::Value<'frame, 'data>,
}

#[repr(C)]
#[jlrs(julia_type = "Main.WithGeneric.WithNestedGenericT")]
#[derive(Copy, Clone, Debug, JuliaStruct)]
pub struct WithNestedGenericT<T>
where
    T: ::jlrs::traits::ValidLayout + Copy,
{
    pub a: WithGenericT<T>,
}

#[repr(C)]
#[jlrs(julia_type = "Main.WithGeneric.WithPropagatedLifetime")]
#[derive(Copy, Clone, Debug, JuliaStruct)]
pub struct WithPropagatedLifetime<'frame> {
    pub a: WithGenericT<::jlrs::value::module::Module<'frame>>,
}

#[repr(C)]
#[jlrs(julia_type = "Main.WithGeneric.WithPropagatedLifetimes")]
#[derive(Copy, Clone, Debug, JuliaStruct)]
pub struct WithPropagatedLifetimes<'frame, 'data> {
    pub a: WithGenericT<
        ::jlrs::value::tuple::Tuple2<i32, WithGenericT<::jlrs::value::array::Array<'frame, 'data>>>,
    >,
}

#[repr(C)]
#[jlrs(julia_type = "Main.WithGeneric.WithSetGeneric")]
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
pub struct WithSetGeneric {
    pub a: WithGenericT<i64>,
}

#[repr(C)]
#[jlrs(julia_type = "Main.WithGeneric.WithSetGenericTuple")]
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
pub struct WithSetGenericTuple {
    pub a: ::jlrs::value::tuple::Tuple1<WithGenericT<i64>>,
}

#[repr(C)]
#[jlrs(julia_type = "Main.WithGeneric.WithValueType")]
#[derive(Copy, Clone, Debug, JuliaStruct)]
pub struct WithValueType {
    pub a: i64,
}

#[repr(C)]
#[jlrs(julia_type = "Main.WithNonBitsUnion.NonBitsUnion")]
#[derive(Copy, Clone, Debug, JuliaStruct)]
pub struct NonBitsUnion<'frame, 'data> {
    pub a: ::jlrs::value::Value<'frame, 'data>,
}

#[repr(C)]
#[jlrs(julia_type = "Main.WithStrings.WithString")]
#[derive(Copy, Clone, Debug, JuliaStruct)]
pub struct WithString<'frame> {
    pub a: ::jlrs::value::string::JuliaString<'frame>,
}
