use jlrs::prelude::*;

#[repr(C)]
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
#[jlrs(julia_type = "Main.BitsWithCustom.BitsCharBitsIntChar")]
pub struct BitsCharBitsIntChar {
    pub a: char,
    pub b: BitsIntChar,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
#[jlrs(julia_type = "Main.BitsWithCustom.BitsIntChar")]
pub struct BitsIntChar {
    pub a: i64,
    pub b: char,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
#[jlrs(julia_type = "Main.BitsWithTuples.BitsUInt8TupleInt32Int64")]
pub struct BitsUInt8TupleInt32Int64 {
    pub a: u8,
    pub b: ::jlrs::value::tuple::Tuple2<i32, i64>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
#[jlrs(julia_type = "Main.BitsWithTuples.BitsUInt8TupleInt32TupleInt16UInt16")]
pub struct BitsUInt8TupleInt32TupleInt16UInt16 {
    pub a: u8,
    pub b: ::jlrs::value::tuple::Tuple2<i32, ::jlrs::value::tuple::Tuple2<i16, u16>>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
#[jlrs(julia_type = "Main.MultiFieldBits.BitsCharFloat32Float64")]
pub struct BitsCharFloat32Float64 {
    pub a: char,
    pub b: f32,
    pub c: f64,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
#[jlrs(julia_type = "Main.MultiFieldBits.BitsIntBool")]
pub struct BitsIntBool {
    pub a: i64,
    pub b: bool,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeBool")]
pub struct BitsTypeBool {
    pub a: bool,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeChar")]
pub struct BitsTypeChar {
    pub a: char,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeFloat32")]
pub struct BitsTypeFloat32 {
    pub a: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeFloat64")]
pub struct BitsTypeFloat64 {
    pub a: f64,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeInt")]
pub struct BitsTypeInt {
    pub a: i64,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeInt16")]
pub struct BitsTypeInt16 {
    pub a: i16,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeInt32")]
pub struct BitsTypeInt32 {
    pub a: i32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeInt64")]
pub struct BitsTypeInt64 {
    pub a: i64,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeInt8")]
pub struct BitsTypeInt8 {
    pub a: i8,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeUInt")]
pub struct BitsTypeUInt {
    pub a: u64,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeUInt16")]
pub struct BitsTypeUInt16 {
    pub a: u16,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeUInt32")]
pub struct BitsTypeUInt32 {
    pub a: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeUInt64")]
pub struct BitsTypeUInt64 {
    pub a: u64,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeUInt8")]
pub struct BitsTypeUInt8 {
    pub a: u8,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, JuliaStruct)]
#[jlrs(julia_type = "Main.WithBitsUnion.DoubleVariant")]
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
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
#[jlrs(julia_type = "Main.WithBitsUnion.SingleVariant")]
pub struct SingleVariant {
    pub a: i8,
    pub b: i32,
    pub c: i8,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, JuliaStruct)]
#[jlrs(julia_type = "Main.WithBitsUnion.SizeAlignMismatch")]
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
#[derive(Copy, Clone, Debug, JuliaStruct)]
#[jlrs(julia_type = "Main.WithBitsUnion.UnionInTuple")]
pub struct UnionInTuple<'frame, 'data> {
    pub a: i8,
    pub b: ::jlrs::value::wrapper_ref::ValueRef<'frame, 'data>,
    pub c: i8,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, JuliaStruct)]
#[jlrs(julia_type = "Main.WithGeneric.WithGenericT")]
pub struct WithGenericT<T>
where
    T: ::jlrs::layout::valid_layout::ValidLayout + Copy,
{
    pub a: T,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, JuliaStruct)]
#[jlrs(julia_type = "Main.WithGeneric.WithGenericUnionAll")]
pub struct WithGenericUnionAll<'frame, 'data> {
    pub a: ::jlrs::value::wrapper_ref::ValueRef<'frame, 'data>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, JuliaStruct)]
#[jlrs(julia_type = "Main.WithGeneric.WithNestedGenericT")]
pub struct WithNestedGenericT<T>
where
    T: ::jlrs::layout::valid_layout::ValidLayout + Copy,
{
    pub a: WithGenericT<T>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, JuliaStruct)]
#[jlrs(julia_type = "Main.WithGeneric.WithPropagatedLifetime")]
pub struct WithPropagatedLifetime<'frame> {
    pub a: WithGenericT<::jlrs::value::module::Module<'frame>>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, JuliaStruct)]
#[jlrs(julia_type = "Main.WithGeneric.WithPropagatedLifetimes")]
pub struct WithPropagatedLifetimes<'frame, 'data> {
    pub a: WithGenericT<
        ::jlrs::value::tuple::Tuple2<i32, WithGenericT<::jlrs::value::array::Array<'frame, 'data>>>,
    >,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
#[jlrs(julia_type = "Main.WithGeneric.WithSetGeneric")]
pub struct WithSetGeneric {
    pub a: WithGenericT<i64>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, JuliaStruct, IntoJulia)]
#[jlrs(julia_type = "Main.WithGeneric.WithSetGenericTuple")]
pub struct WithSetGenericTuple {
    pub a: ::jlrs::value::tuple::Tuple1<WithGenericT<i64>>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, JuliaStruct)]
#[jlrs(julia_type = "Main.WithGeneric.WithValueType")]
pub struct WithValueType {
    pub a: i64,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, JuliaStruct)]
#[jlrs(julia_type = "Main.WithNonBitsUnion.NonBitsUnion")]
pub struct NonBitsUnion<'frame, 'data> {
    pub a: ::jlrs::value::wrapper_ref::ValueRef<'frame, 'data>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, JuliaStruct)]
#[jlrs(julia_type = "Main.WithStrings.WithString")]
pub struct WithString<'frame> {
    pub a: ::jlrs::value::string::JuliaString<'frame>,
}
