use jlrs::prelude::*;

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia)]
#[jlrs(julia_type = "Main.BitsWithCustom.BitsCharBitsIntChar")]
pub struct BitsCharBitsIntChar {
    pub a: ::jlrs::wrappers::inline::char::Char,
    pub b: BitsIntChar,
}

#[cfg(target_pointer_width = "64")]
#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia)]
#[jlrs(julia_type = "Main.BitsWithCustom.BitsIntChar")]
pub struct BitsIntChar {
    pub a: i64,
    pub b: ::jlrs::wrappers::inline::char::Char,
}

#[cfg(target_pointer_width = "32")]
#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia)]
#[jlrs(julia_type = "Main.BitsWithCustom.BitsIntChar")]
pub struct BitsIntChar {
    pub a: i32,
    pub b: ::jlrs::wrappers::inline::char::Char,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia)]
#[jlrs(julia_type = "Main.BitsWithTuples.BitsUInt8TupleInt32Int64")]
pub struct BitsUInt8TupleInt32Int64 {
    pub a: u8,
    pub b: ::jlrs::wrappers::inline::tuple::Tuple2<i32, i64>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia)]
#[jlrs(julia_type = "Main.BitsWithTuples.BitsUInt8TupleInt32TupleInt16UInt16")]
pub struct BitsUInt8TupleInt32TupleInt16UInt16 {
    pub a: u8,
    pub b: ::jlrs::wrappers::inline::tuple::Tuple2<
        i32,
        ::jlrs::wrappers::inline::tuple::Tuple2<i16, u16>,
    >,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia)]
#[jlrs(julia_type = "Main.MultiFieldBits.BitsCharFloat32Float64")]
pub struct BitsCharFloat32Float64 {
    pub a: ::jlrs::wrappers::inline::char::Char,
    pub b: f32,
    pub c: f64,
}

#[cfg(target_pointer_width = "64")]
#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia)]
#[jlrs(julia_type = "Main.MultiFieldBits.BitsIntBool")]
pub struct BitsIntBool {
    pub a: i64,
    pub b: ::jlrs::wrappers::inline::bool::Bool,
}

#[cfg(target_pointer_width = "32")]
#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia)]
#[jlrs(julia_type = "Main.MultiFieldBits.BitsIntBool")]
pub struct BitsIntBool {
    pub a: i32,
    pub b: ::jlrs::wrappers::inline::bool::Bool,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeBool")]
pub struct BitsTypeBool {
    pub a: ::jlrs::wrappers::inline::bool::Bool,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeChar")]
pub struct BitsTypeChar {
    pub a: ::jlrs::wrappers::inline::char::Char,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeFloat32")]
pub struct BitsTypeFloat32 {
    pub a: f32,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeFloat64")]
pub struct BitsTypeFloat64 {
    pub a: f64,
}

#[cfg(target_pointer_width = "64")]
#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeInt")]
pub struct BitsTypeInt {
    pub a: i64,
}

#[cfg(target_pointer_width = "32")]
#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeInt")]
pub struct BitsTypeInt {
    pub a: i32,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeInt16")]
pub struct BitsTypeInt16 {
    pub a: i16,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeInt32")]
pub struct BitsTypeInt32 {
    pub a: i32,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeInt64")]
pub struct BitsTypeInt64 {
    pub a: i64,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeInt8")]
pub struct BitsTypeInt8 {
    pub a: i8,
}

#[cfg(target_pointer_width = "64")]
#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeUInt")]
pub struct BitsTypeUInt {
    pub a: u64,
}

#[cfg(target_pointer_width = "32")]
#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeUInt")]
pub struct BitsTypeUInt {
    pub a: u32,
}
#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeUInt16")]
pub struct BitsTypeUInt16 {
    pub a: u16,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeUInt32")]
pub struct BitsTypeUInt32 {
    pub a: u32,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeUInt64")]
pub struct BitsTypeUInt64 {
    pub a: u64,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia)]
#[jlrs(julia_type = "Main.SingleFieldBits.BitsTypeUInt8")]
pub struct BitsTypeUInt8 {
    pub a: u8,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck)]
#[jlrs(julia_type = "Main.WithBitsUnion.DoubleVariant")]
pub struct DoubleVariant {
    pub a: i8,
    #[jlrs(bits_union_align)]
    _b_align: ::jlrs::wrappers::inline::union::Align4,
    #[jlrs(bits_union)]
    pub b: ::jlrs::wrappers::inline::union::BitsUnion<4>,
    #[jlrs(bits_union_flag)]
    pub b_flag: u8,
    pub c: i8,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia)]
#[jlrs(julia_type = "Main.WithBitsUnion.SingleVariant")]
pub struct SingleVariant {
    pub a: i8,
    pub b: i32,
    pub c: i8,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck)]
#[jlrs(julia_type = "Main.WithBitsUnion.SizeAlignMismatch")]
pub struct SizeAlignMismatch {
    pub a: i8,
    #[jlrs(bits_union_align)]
    _b_align: ::jlrs::wrappers::inline::union::Align4,
    #[jlrs(bits_union)]
    pub b: ::jlrs::wrappers::inline::union::BitsUnion<6>,
    #[jlrs(bits_union_flag)]
    pub b_flag: u8,
    pub c: i8,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck)]
#[jlrs(julia_type = "Main.WithBitsUnion.UnionInTuple")]
pub struct UnionInTuple<'frame, 'data> {
    pub a: i8,
    pub b: ::std::option::Option<::jlrs::wrappers::ptr::value::ValueRef<'frame, 'data>>,
    pub c: i8,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck)]
#[jlrs(julia_type = "Main.WithGeneric.WithGenericT")]
pub struct WithGenericT<T>
where
    T: ::jlrs::layout::valid_layout::ValidField + Clone,
{
    pub a: T,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck)]
#[jlrs(julia_type = "Main.WithGeneric.WithGenericUnionAll")]
pub struct WithGenericUnionAll<'frame, 'data> {
    pub a: ::std::option::Option<::jlrs::wrappers::ptr::value::ValueRef<'frame, 'data>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck)]
#[jlrs(julia_type = "Main.WithGeneric.WithNestedGenericT")]
pub struct WithNestedGenericT<T>
where
    T: ::jlrs::layout::valid_layout::ValidField + Clone,
{
    pub a: WithGenericT<T>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck)]
#[jlrs(julia_type = "Main.WithGeneric.WithPropagatedLifetime")]
pub struct WithPropagatedLifetime<'frame> {
    pub a: WithGenericT<::std::option::Option<::jlrs::wrappers::ptr::module::ModuleRef<'frame>>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck)]
#[jlrs(julia_type = "Main.WithGeneric.WithPropagatedLifetimes")]
pub struct WithPropagatedLifetimes<'frame, 'data> {
    pub a: WithGenericT<
        ::jlrs::wrappers::inline::tuple::Tuple2<
            i32,
            WithGenericT<
                ::std::option::Option<::jlrs::wrappers::ptr::array::ArrayRef<'frame, 'data>>,
            >,
        >,
    >,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia)]
#[jlrs(julia_type = "Main.WithGeneric.WithSetGeneric")]
pub struct WithSetGeneric {
    pub a: WithGenericT<i64>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia)]
#[jlrs(julia_type = "Main.WithGeneric.WithSetGenericTuple")]
pub struct WithSetGenericTuple {
    pub a: ::jlrs::wrappers::inline::tuple::Tuple1<WithGenericT<i64>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck)]
#[jlrs(julia_type = "Main.WithGeneric.WithValueType")]
pub struct WithValueType {
    pub a: i64,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck)]
#[jlrs(julia_type = "Main.WithNonBitsUnion.NonBitsUnion")]
pub struct NonBitsUnion<'frame, 'data> {
    pub a: ::std::option::Option<::jlrs::wrappers::ptr::value::ValueRef<'frame, 'data>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia)]
#[jlrs(julia_type = "Main.ZST.ZeroSized", zero_sized_type)]
pub struct ZeroSized {}
