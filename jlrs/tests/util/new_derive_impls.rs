use jlrs::prelude::*;

#[repr(C)]
#[derive(
    Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia, ConstructType, CCallArg,
)]
#[jlrs(julia_type = "Main.BitsCharBitsIntChar")]
pub struct BitsCharBitsIntChar {
    pub a: ::jlrs::data::layout::char::Char,
    pub b: BitsIntChar,
}

#[repr(C)]
#[derive(
    Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia, ConstructType, CCallArg,
)]
#[jlrs(julia_type = "Main.BitsCharFloat32Float64")]
pub struct BitsCharFloat32Float64 {
    pub a: ::jlrs::data::layout::char::Char,
    pub b: f32,
    pub c: f64,
}

#[repr(C)]
#[derive(
    Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia, ConstructType, CCallArg,
)]
#[jlrs(julia_type = "Main.BitsIntBool")]
pub struct BitsIntBool {
    pub a: i64,
    pub b: ::jlrs::data::layout::bool::Bool,
}

#[repr(C)]
#[derive(
    Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia, ConstructType, CCallArg,
)]
#[jlrs(julia_type = "Main.BitsIntChar")]
pub struct BitsIntChar {
    pub a: i64,
    pub b: ::jlrs::data::layout::char::Char,
}

#[repr(C)]
#[derive(
    Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia, ConstructType, CCallArg,
)]
#[jlrs(julia_type = "Main.BitsTypeBool")]
pub struct BitsTypeBool {
    pub a: ::jlrs::data::layout::bool::Bool,
}

#[repr(C)]
#[derive(
    Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia, ConstructType, CCallArg,
)]
#[jlrs(julia_type = "Main.BitsTypeChar")]
pub struct BitsTypeChar {
    pub a: ::jlrs::data::layout::char::Char,
}

#[repr(C)]
#[derive(
    Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia, ConstructType, CCallArg,
)]
#[jlrs(julia_type = "Main.BitsTypeFloat32")]
pub struct BitsTypeFloat32 {
    pub a: f32,
}

#[repr(C)]
#[derive(
    Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia, ConstructType, CCallArg,
)]
#[jlrs(julia_type = "Main.BitsTypeFloat64")]
pub struct BitsTypeFloat64 {
    pub a: f64,
}

#[repr(C)]
#[derive(
    Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia, ConstructType, CCallArg,
)]
#[jlrs(julia_type = "Main.BitsTypeInt")]
pub struct BitsTypeInt {
    pub a: i64,
}

#[repr(C)]
#[derive(
    Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia, ConstructType, CCallArg,
)]
#[jlrs(julia_type = "Main.BitsTypeInt16")]
pub struct BitsTypeInt16 {
    pub a: i16,
}

#[repr(C)]
#[derive(
    Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia, ConstructType, CCallArg,
)]
#[jlrs(julia_type = "Main.BitsTypeInt32")]
pub struct BitsTypeInt32 {
    pub a: i32,
}

#[repr(C)]
#[derive(
    Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia, ConstructType, CCallArg,
)]
#[jlrs(julia_type = "Main.BitsTypeInt64")]
pub struct BitsTypeInt64 {
    pub a: i64,
}

#[repr(C)]
#[derive(
    Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia, ConstructType, CCallArg,
)]
#[jlrs(julia_type = "Main.BitsTypeInt8")]
pub struct BitsTypeInt8 {
    pub a: i8,
}

#[repr(C)]
#[derive(
    Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia, ConstructType, CCallArg,
)]
#[jlrs(julia_type = "Main.BitsTypeUInt")]
pub struct BitsTypeUInt {
    pub a: u64,
}

#[repr(C)]
#[derive(
    Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia, ConstructType, CCallArg,
)]
#[jlrs(julia_type = "Main.BitsTypeUInt16")]
pub struct BitsTypeUInt16 {
    pub a: u16,
}

#[repr(C)]
#[derive(
    Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia, ConstructType, CCallArg,
)]
#[jlrs(julia_type = "Main.BitsTypeUInt32")]
pub struct BitsTypeUInt32 {
    pub a: u32,
}

#[repr(C)]
#[derive(
    Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia, ConstructType, CCallArg,
)]
#[jlrs(julia_type = "Main.BitsTypeUInt64")]
pub struct BitsTypeUInt64 {
    pub a: u64,
}

#[repr(C)]
#[derive(
    Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia, ConstructType, CCallArg,
)]
#[jlrs(julia_type = "Main.BitsTypeUInt8")]
pub struct BitsTypeUInt8 {
    pub a: u8,
}

#[repr(C)]
#[derive(
    Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia, ConstructType, CCallArg,
)]
#[jlrs(julia_type = "Main.BitsUInt8TupleInt32Int64")]
pub struct BitsUInt8TupleInt32Int64 {
    pub a: u8,
    pub b: ::jlrs::data::layout::tuple::Tuple2<i32, i64>,
}

#[repr(C)]
#[derive(
    Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia, ConstructType, CCallArg,
)]
#[jlrs(julia_type = "Main.BitsUInt8TupleInt32TupleInt16UInt16")]
pub struct BitsUInt8TupleInt32TupleInt16UInt16 {
    pub a: u8,
    pub b: ::jlrs::data::layout::tuple::Tuple2<i32, ::jlrs::data::layout::tuple::Tuple2<i16, u16>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck)]
#[jlrs(julia_type = "Main.DoubleHasGeneric")]
pub struct DoubleHasGeneric<'frame, 'data> {
    pub a: ::std::option::Option<::jlrs::data::managed::value::ValueRef<'frame, 'data>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.DoubleImmut")]
pub struct DoubleImmut<'frame, 'data> {
    pub a: Immut<'frame, 'data>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.DoubleVariant")]
pub struct DoubleVariant {
    #[jlrs(bits_union_align)]
    _a_align: ::jlrs::data::layout::union::Align4,
    #[jlrs(bits_union)]
    pub a: ::jlrs::data::layout::union::BitsUnion<4>,
    #[jlrs(bits_union_flag)]
    pub a_flag: u8,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia, ConstructType)]
#[jlrs(julia_type = "Main.Empty", zero_sized_type)]
pub struct Empty {}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck)]
#[jlrs(julia_type = "Main.HasGenericImmut")]
pub struct HasGenericImmut<'frame, 'data> {
    pub a: ::std::option::Option<::jlrs::data::managed::value::ValueRef<'frame, 'data>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.HasGeneric")]
pub struct HasGeneric<T>
where
    T: ::jlrs::data::layout::valid_layout::ValidField + Clone,
{
    pub a: T,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.HasImmut")]
pub struct HasImmut<'frame, 'data> {
    pub a: Immut<'frame, 'data>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.Immut")]
pub struct Immut<'frame, 'data> {
    pub a: ::std::option::Option<::jlrs::data::managed::value::ValueRef<'frame, 'data>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.MutF32")]
pub struct MutF32 {
    pub a: f32,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.MutNested")]
pub struct MutNested<'frame, 'data> {
    pub a: ::std::option::Option<::jlrs::data::managed::value::ValueRef<'frame, 'data>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.NonBitsUnion")]
pub struct NonBitsUnion<'frame, 'data> {
    pub a: ::std::option::Option<::jlrs::data::managed::value::ValueRef<'frame, 'data>>,
}

#[repr(C)]
#[derive(
    Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia, ConstructType, CCallArg,
)]
#[jlrs(julia_type = "Main.SingleVariant")]
pub struct SingleVariant {
    pub a: i32,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.SizeAlignMismatch")]
pub struct SizeAlignMismatch {
    #[jlrs(bits_union_align)]
    _a_align: ::jlrs::data::layout::union::Align4,
    #[jlrs(bits_union)]
    pub a: ::jlrs::data::layout::union::BitsUnion<6>,
    #[jlrs(bits_union_flag)]
    pub a_flag: u8,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck)]
#[jlrs(julia_type = "Main.TypedEmpty")]
pub struct TypedEmpty {}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.UnionInTuple")]
pub struct UnionInTuple<'frame, 'data> {
    pub a: ::std::option::Option<::jlrs::data::managed::value::ValueRef<'frame, 'data>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.WithArray")]
pub struct WithArray<'frame, 'data> {
    pub a: ::std::option::Option<::jlrs::data::managed::array::ArrayRef<'frame, 'data>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.WithCodeInstance")]
pub struct WithCodeInstance<'frame, 'data> {
    pub a: ::std::option::Option<::jlrs::data::managed::value::ValueRef<'frame, 'data>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.WithDataType")]
pub struct WithDataType<'frame> {
    pub a: ::std::option::Option<::jlrs::data::managed::datatype::DataTypeRef<'frame>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.WithExpr")]
pub struct WithExpr<'frame, 'data> {
    pub a: ::std::option::Option<::jlrs::data::managed::value::ValueRef<'frame, 'data>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.WithGenericT")]
pub struct WithGenericT<T>
where
    T: ::jlrs::data::layout::valid_layout::ValidField + Clone,
{
    pub a: T,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.WithGenericTU")]
pub struct WithGenericTU<T, U>
where
    T: ::jlrs::data::layout::valid_layout::ValidField + Clone,
    U: ::jlrs::data::layout::valid_layout::ValidField + Clone,
{
    pub a: T,
    pub b: U,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.WithGenericUnionAll")]
pub struct WithGenericUnionAll<'frame, 'data> {
    pub a: ::std::option::Option<::jlrs::data::managed::value::ValueRef<'frame, 'data>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.WithMethod")]
pub struct WithMethod<'frame, 'data> {
    pub a: ::std::option::Option<::jlrs::data::managed::value::ValueRef<'frame, 'data>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.WithMethodInstance")]
pub struct WithMethodInstance<'frame, 'data> {
    pub a: ::std::option::Option<::jlrs::data::managed::value::ValueRef<'frame, 'data>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.WithMethodTable")]
pub struct WithMethodTable<'frame, 'data> {
    pub a: ::std::option::Option<::jlrs::data::managed::value::ValueRef<'frame, 'data>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.WithModule")]
pub struct WithModule<'frame> {
    pub a: ::std::option::Option<::jlrs::data::managed::module::ModuleRef<'frame>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.WithNestedGenericT")]
pub struct WithNestedGenericT<T>
where
    T: ::jlrs::data::layout::valid_layout::ValidField + Clone,
{
    pub a: WithGenericT<T>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.WithPropagatedLifetime")]
pub struct WithPropagatedLifetime<'frame> {
    pub a: WithGenericT<::std::option::Option<::jlrs::data::managed::module::ModuleRef<'frame>>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.WithPropagatedLifetimes")]
pub struct WithPropagatedLifetimes<'frame, 'data> {
    pub a: WithGenericT<
        ::jlrs::data::layout::tuple::Tuple2<
            i32,
            WithGenericT<
                ::std::option::Option<::jlrs::data::managed::array::ArrayRef<'frame, 'data>>,
            >,
        >,
    >,
}

#[repr(C)]
#[derive(
    Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia, ConstructType, CCallArg,
)]
#[jlrs(julia_type = "Main.WithSetGeneric")]
pub struct WithSetGeneric {
    pub a: WithGenericT<i64>,
}

#[repr(C)]
#[derive(
    Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, IntoJulia, ConstructType, CCallArg,
)]
#[jlrs(julia_type = "Main.WithSetGenericTuple")]
pub struct WithSetGenericTuple {
    pub a: ::jlrs::data::layout::tuple::Tuple1<WithGenericT<i64>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.WithSimpleVector")]
pub struct WithSimpleVector<'frame> {
    pub a: ::std::option::Option<::jlrs::data::managed::simple_vector::SimpleVectorRef<'frame>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.WithString")]
pub struct WithString<'frame> {
    pub a: ::std::option::Option<::jlrs::data::managed::string::StringRef<'frame>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.WithSymbol")]
pub struct WithSymbol<'frame> {
    pub a: ::std::option::Option<::jlrs::data::managed::symbol::SymbolRef<'frame>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.WithTask")]
pub struct WithTask<'frame> {
    pub a: ::std::option::Option<::jlrs::data::managed::task::TaskRef<'frame>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.WithTypeMapEntry")]
pub struct WithTypeMapEntry<'frame, 'data> {
    pub a: ::std::option::Option<::jlrs::data::managed::value::ValueRef<'frame, 'data>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.WithTypeMapLevel")]
pub struct WithTypeMapLevel<'frame, 'data> {
    pub a: ::std::option::Option<::jlrs::data::managed::value::ValueRef<'frame, 'data>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.WithTypeName")]
pub struct WithTypeName<'frame> {
    pub a: ::std::option::Option<::jlrs::data::managed::type_name::TypeNameRef<'frame>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.WithTypeVar")]
pub struct WithTypeVar<'frame> {
    pub a: ::std::option::Option<::jlrs::data::managed::type_var::TypeVarRef<'frame>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.WithUnion")]
pub struct WithUnion<'frame> {
    pub a: ::std::option::Option<::jlrs::data::managed::union::UnionRef<'frame>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.WithUnionAll")]
pub struct WithUnionAll<'frame> {
    pub a: ::std::option::Option<::jlrs::data::managed::union_all::UnionAllRef<'frame>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, ValidField, Typecheck)]
#[jlrs(julia_type = "Main.WithValueType")]
pub struct WithValueType {
    pub a: i64,
}
