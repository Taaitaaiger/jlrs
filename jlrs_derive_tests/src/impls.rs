use jlrs::prelude::*;

#[repr(C)]
#[jlrs(julia_type = "Main.BitsWithCustom.BitsCharBitsIntChar")]
#[derive(Clone, Debug, Unbox, ValidLayout, IntoJulia)]
pub struct BitsCharBitsIntChar {
    pub a: ::jlrs::wrappers::inline::char::Char,
    pub b: BitsIntChar,
}

#[repr(C)]
#[jlrs(julia_type = "Main.BitsWithCustom.BitsIntChar")]
#[derive(Clone, Debug, Unbox, ValidLayout, IntoJulia)]
pub struct BitsIntChar {
    pub a: i64,
    pub b: ::jlrs::wrappers::inline::char::Char,
}

#[repr(C)]
#[jlrs(julia_type = "Main.BitsWithTuples.BitsUInt8TupleInt32Int64")]
#[derive(Clone, Debug, Unbox, ValidLayout, IntoJulia)]
pub struct BitsUInt8TupleInt32Int64 {
    pub a: u8,
    pub b: ::jlrs::wrappers::inline::tuple::Tuple2<i32, i64>,
}

#[repr(C)]
#[jlrs(julia_type = "Main.BitsWithTuples.BitsUInt8TupleInt32TupleInt16UInt16")]
#[derive(Clone, Debug, Unbox, ValidLayout, IntoJulia)]
pub struct BitsUInt8TupleInt32TupleInt16UInt16 {
    pub a: u8,
    pub b: ::jlrs::wrappers::inline::tuple::Tuple2<i32, ::jlrs::wrappers::inline::tuple::Tuple2<i16, u16>>,
}

#[repr(C)]
#[jlrs(julia_type = "Main.MultiFieldBits.BitsCharFloat32Float64")]
#[derive(Clone, Debug, Unbox, ValidLayout, IntoJulia)]
pub struct BitsCharFloat32Float64 {
    pub a: ::jlrs::wrappers::inline::char::Char,
    pub b: f32,
    pub c: f64,
}

#[repr(C)]
#[jlrs(julia_type = "Main.MultiFieldBits.BitsIntBool")]
#[derive(Clone, Debug, Unbox, ValidLayout, IntoJulia)]
pub struct BitsIntBool {
    pub a: i64,
    pub b: ::jlrs::wrappers::inline::bool::Bool,
}

#[repr(C)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeBool")]
#[derive(Clone, Debug, Unbox, ValidLayout, IntoJulia)]
pub struct BitsTypeBool {
    pub a: ::jlrs::wrappers::inline::bool::Bool,
}

#[repr(C)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeChar")]
#[derive(Clone, Debug, Unbox, ValidLayout, IntoJulia)]
pub struct BitsTypeChar {
    pub a: ::jlrs::wrappers::inline::char::Char,
}

#[repr(C)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeFloat32")]
#[derive(Clone, Debug, Unbox, ValidLayout, IntoJulia)]
pub struct BitsTypeFloat32 {
    pub a: f32,
}

#[repr(C)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeFloat64")]
#[derive(Clone, Debug, Unbox, ValidLayout, IntoJulia)]
pub struct BitsTypeFloat64 {
    pub a: f64,
}

#[repr(C)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeInt")]
#[derive(Clone, Debug, Unbox, ValidLayout, IntoJulia)]
pub struct BitsTypeInt {
    pub a: i64,
}

#[repr(C)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeInt16")]
#[derive(Clone, Debug, Unbox, ValidLayout, IntoJulia)]
pub struct BitsTypeInt16 {
    pub a: i16,
}

#[repr(C)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeInt32")]
#[derive(Clone, Debug, Unbox, ValidLayout, IntoJulia)]
pub struct BitsTypeInt32 {
    pub a: i32,
}

#[repr(C)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeInt64")]
#[derive(Clone, Debug, Unbox, ValidLayout, IntoJulia)]
pub struct BitsTypeInt64 {
    pub a: i64,
}

#[repr(C)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeInt8")]
#[derive(Clone, Debug, Unbox, ValidLayout, IntoJulia)]
pub struct BitsTypeInt8 {
    pub a: i8,
}

#[repr(C)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeUInt")]
#[derive(Clone, Debug, Unbox, ValidLayout, IntoJulia)]
pub struct BitsTypeUInt {
    pub a: u64,
}

#[repr(C)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeUInt16")]
#[derive(Clone, Debug, Unbox, ValidLayout, IntoJulia)]
pub struct BitsTypeUInt16 {
    pub a: u16,
}

#[repr(C)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeUInt32")]
#[derive(Clone, Debug, Unbox, ValidLayout, IntoJulia)]
pub struct BitsTypeUInt32 {
    pub a: u32,
}

#[repr(C)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeUInt64")]
#[derive(Clone, Debug, Unbox, ValidLayout, IntoJulia)]
pub struct BitsTypeUInt64 {
    pub a: u64,
}

#[repr(C)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeUInt8")]
#[derive(Clone, Debug, Unbox, ValidLayout, IntoJulia)]
pub struct BitsTypeUInt8 {
    pub a: u8,
}

#[repr(C)]
#[jlrs(julia_type = "Main.WithBitsUnion.DoubleVariant")]
#[derive(Clone, Debug, Unbox, ValidLayout)]
pub struct DoubleVariant {
    pub a: i8,
    #[jlrs(bits_union_align)]
    _b_align: ::jlrs::wrappers::inline::union::Align4,
    #[jlrs(bits_union)]
    pub b: ::jlrs::wrappers::inline::union::BitsUnionContainer<4>,
    #[jlrs(bits_union_flag)]
    pub b_flag: u8,
    pub c: i8,
}

#[repr(C)]
#[jlrs(julia_type = "Main.WithBitsUnion.SingleVariant")]
#[derive(Clone, Debug, Unbox, ValidLayout, IntoJulia)]
pub struct SingleVariant {
    pub a: i8,
    pub b: i32,
    pub c: i8,
}

#[repr(C)]
#[jlrs(julia_type = "Main.WithBitsUnion.SizeAlignMismatch")]
#[derive(Clone, Debug, Unbox, ValidLayout)]
pub struct SizeAlignMismatch {
    pub a: i8,
    #[jlrs(bits_union_align)]
    _b_align: ::jlrs::wrappers::inline::union::Align4,
    #[jlrs(bits_union)]
    pub b: ::jlrs::wrappers::inline::union::BitsUnionContainer<6>,
    #[jlrs(bits_union_flag)]
    pub b_flag: u8,
    pub c: i8,
}

#[repr(C)]
#[jlrs(julia_type = "Main.WithBitsUnion.UnionInTuple")]
#[derive(Clone, Debug, Unbox, ValidLayout)]
pub struct UnionInTuple<'frame, 'data> {
    pub a: i8,
    pub b: ::jlrs::wrappers::ptr::ValueRef<'frame, 'data>,
    pub c: i8,
}

#[repr(C)]
#[jlrs(julia_type = "Main.WithGeneric.WithGenericT")]
#[derive(Clone, Debug, Unbox, ValidLayout)]
pub struct WithGenericT<T>
where
    T: ::jlrs::layout::valid_layout::ValidLayout + Clone,
{
    pub a: T,
}

#[repr(C)]
#[jlrs(julia_type = "Main.WithGeneric.WithGenericUnionAll")]
#[derive(Clone, Debug, Unbox, ValidLayout)]
pub struct WithGenericUnionAll<'frame, 'data> {
    pub a: ::jlrs::wrappers::ptr::ValueRef<'frame, 'data>,
}

#[repr(C)]
#[jlrs(julia_type = "Main.WithGeneric.WithNestedGenericT")]
#[derive(Clone, Debug, Unbox, ValidLayout)]
pub struct WithNestedGenericT<T>
where
    T: ::jlrs::layout::valid_layout::ValidLayout + Clone,
{
    pub a: WithGenericT<T>,
}

#[repr(C)]
#[jlrs(julia_type = "Main.WithGeneric.WithPropagatedLifetime")]
#[derive(Clone, Debug, Unbox, ValidLayout)]
pub struct WithPropagatedLifetime<'frame> {
    pub a: WithGenericT<::jlrs::wrappers::ptr::ModuleRef<'frame>>,
}

#[repr(C)]
#[jlrs(julia_type = "Main.WithGeneric.WithPropagatedLifetimes")]
#[derive(Clone, Debug, Unbox, ValidLayout)]
pub struct WithPropagatedLifetimes<'frame, 'data> {
    pub a: WithGenericT<::jlrs::wrappers::inline::tuple::Tuple2<i32, WithGenericT<::jlrs::wrappers::ptr::ArrayRef<'frame, 'data>>>>,
}

#[repr(C)]
#[jlrs(julia_type = "Main.WithGeneric.WithSetGeneric")]
#[derive(Clone, Debug, Unbox, ValidLayout, IntoJulia)]
pub struct WithSetGeneric {
    pub a: WithGenericT<i64>,
}

#[repr(C)]
#[jlrs(julia_type = "Main.WithGeneric.WithSetGenericTuple")]
#[derive(Clone, Debug, Unbox, ValidLayout, IntoJulia)]
pub struct WithSetGenericTuple {
    pub a: ::jlrs::wrappers::inline::tuple::Tuple1<WithGenericT<i64>>,
}

#[repr(C)]
#[jlrs(julia_type = "Main.WithGeneric.WithValueType")]
#[derive(Clone, Debug, Unbox, ValidLayout)]
pub struct WithValueType {
    pub a: i64,
}

#[repr(C)]
#[jlrs(julia_type = "Main.WithNonBitsUnion.NonBitsUnion")]
#[derive(Clone, Debug, Unbox, ValidLayout)]
pub struct NonBitsUnion<'frame, 'data> {
    pub a: ::jlrs::wrappers::ptr::ValueRef<'frame, 'data>,
}

#[repr(C)]
#[jlrs(julia_type = "Main.ZST.ZeroSized", zero_sized_type)]
#[derive(Clone, Debug, Unbox, ValidLayout, IntoJulia)]
pub struct ZeroSized {
}
