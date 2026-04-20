// TODO
#![allow(unused)]

use jlrs::prelude::*;

#[derive(ConstructType)]
#[jlrs(julia_type = "AnAbstractType")]
pub struct AnAbstractType {}

#[derive(ConstructType)]
#[jlrs(julia_type = "AnAbstractUnionAll")]
pub struct AnAbstractUnionAll<T> {
    _t: ::std::marker::PhantomData<T>,
}

#[repr(C)]
#[derive(
    Clone,
    Debug,
    Unbox,
    ValidLayout,
    Typecheck,
    IntoJulia,
    ValidField,
    IsBits,
    ConstructType,
    CCallArg,
    CCallReturn,
)]
#[jlrs(julia_type = "BitsCharBitsIntChar")]
pub struct BitsCharBitsIntChar {
    pub a: ::jlrs::data::layout::char::Char,
    pub b: BitsIntChar,
}

#[repr(C)]
#[derive(
    Clone,
    Debug,
    Unbox,
    ValidLayout,
    Typecheck,
    IntoJulia,
    ValidField,
    IsBits,
    ConstructType,
    CCallArg,
    CCallReturn,
)]
#[jlrs(julia_type = "BitsCharFloat32Float64")]
pub struct BitsCharFloat32Float64 {
    pub a: ::jlrs::data::layout::char::Char,
    pub b: f32,
    pub c: f64,
}

#[repr(C)]
#[derive(
    Clone,
    Debug,
    Unbox,
    ValidLayout,
    Typecheck,
    IntoJulia,
    ValidField,
    IsBits,
    ConstructType,
    CCallArg,
    CCallReturn,
)]
#[jlrs(julia_type = "BitsIntBool")]
pub struct BitsIntBool {
    pub a: i64,
    pub b: ::jlrs::data::layout::bool::Bool,
}

#[repr(C)]
#[derive(
    Clone,
    Debug,
    Unbox,
    ValidLayout,
    Typecheck,
    IntoJulia,
    ValidField,
    IsBits,
    ConstructType,
    CCallArg,
    CCallReturn,
)]
#[jlrs(julia_type = "BitsIntChar")]
pub struct BitsIntChar {
    pub a: i64,
    pub b: ::jlrs::data::layout::char::Char,
}

#[repr(C)]
#[derive(
    Clone,
    Debug,
    Unbox,
    ValidLayout,
    Typecheck,
    IntoJulia,
    ValidField,
    IsBits,
    ConstructType,
    CCallArg,
    CCallReturn,
)]
#[jlrs(julia_type = "BitsTypeBool")]
pub struct BitsTypeBool {
    pub a: ::jlrs::data::layout::bool::Bool,
}

#[repr(C)]
#[derive(
    Clone,
    Debug,
    Unbox,
    ValidLayout,
    Typecheck,
    IntoJulia,
    ValidField,
    IsBits,
    ConstructType,
    CCallArg,
    CCallReturn,
)]
#[jlrs(julia_type = "BitsTypeChar")]
pub struct BitsTypeChar {
    pub a: ::jlrs::data::layout::char::Char,
}

#[repr(C)]
#[derive(
    Clone,
    Debug,
    Unbox,
    ValidLayout,
    Typecheck,
    IntoJulia,
    ValidField,
    IsBits,
    ConstructType,
    CCallArg,
    CCallReturn,
)]
#[jlrs(julia_type = "BitsTypeFloat32")]
pub struct BitsTypeFloat32 {
    pub a: f32,
}

#[repr(C)]
#[derive(
    Clone,
    Debug,
    Unbox,
    ValidLayout,
    Typecheck,
    IntoJulia,
    ValidField,
    IsBits,
    ConstructType,
    CCallArg,
    CCallReturn,
)]
#[jlrs(julia_type = "BitsTypeFloat64")]
pub struct BitsTypeFloat64 {
    pub a: f64,
}

#[repr(C)]
#[derive(
    Clone,
    Debug,
    Unbox,
    ValidLayout,
    Typecheck,
    IntoJulia,
    ValidField,
    IsBits,
    ConstructType,
    CCallArg,
    CCallReturn,
)]
#[jlrs(julia_type = "BitsTypeInt")]
pub struct BitsTypeInt {
    pub a: i64,
}

#[repr(C)]
#[derive(
    Clone,
    Debug,
    Unbox,
    ValidLayout,
    Typecheck,
    IntoJulia,
    ValidField,
    IsBits,
    ConstructType,
    CCallArg,
    CCallReturn,
)]
#[jlrs(julia_type = "BitsTypeInt16")]
pub struct BitsTypeInt16 {
    pub a: i16,
}

#[repr(C)]
#[derive(
    Clone,
    Debug,
    Unbox,
    ValidLayout,
    Typecheck,
    IntoJulia,
    ValidField,
    IsBits,
    ConstructType,
    CCallArg,
    CCallReturn,
)]
#[jlrs(julia_type = "BitsTypeInt32")]
pub struct BitsTypeInt32 {
    pub a: i32,
}

#[repr(C)]
#[derive(
    Clone,
    Debug,
    Unbox,
    ValidLayout,
    Typecheck,
    IntoJulia,
    ValidField,
    IsBits,
    ConstructType,
    CCallArg,
    CCallReturn,
)]
#[jlrs(julia_type = "BitsTypeInt64")]
pub struct BitsTypeInt64 {
    pub a: i64,
}

#[repr(C)]
#[derive(
    Clone,
    Debug,
    Unbox,
    ValidLayout,
    Typecheck,
    IntoJulia,
    ValidField,
    IsBits,
    ConstructType,
    CCallArg,
    CCallReturn,
)]
#[jlrs(julia_type = "BitsTypeInt8")]
pub struct BitsTypeInt8 {
    pub a: i8,
}

#[repr(C)]
#[derive(
    Clone,
    Debug,
    Unbox,
    ValidLayout,
    Typecheck,
    IntoJulia,
    ValidField,
    IsBits,
    ConstructType,
    CCallArg,
    CCallReturn,
)]
#[jlrs(julia_type = "BitsTypeUInt")]
pub struct BitsTypeUInt {
    pub a: u64,
}

#[repr(C)]
#[derive(
    Clone,
    Debug,
    Unbox,
    ValidLayout,
    Typecheck,
    IntoJulia,
    ValidField,
    IsBits,
    ConstructType,
    CCallArg,
    CCallReturn,
)]
#[jlrs(julia_type = "BitsTypeUInt16")]
pub struct BitsTypeUInt16 {
    pub a: u16,
}

#[repr(C)]
#[derive(
    Clone,
    Debug,
    Unbox,
    ValidLayout,
    Typecheck,
    IntoJulia,
    ValidField,
    IsBits,
    ConstructType,
    CCallArg,
    CCallReturn,
)]
#[jlrs(julia_type = "BitsTypeUInt32")]
pub struct BitsTypeUInt32 {
    pub a: u32,
}

#[repr(C)]
#[derive(
    Clone,
    Debug,
    Unbox,
    ValidLayout,
    Typecheck,
    IntoJulia,
    ValidField,
    IsBits,
    ConstructType,
    CCallArg,
    CCallReturn,
)]
#[jlrs(julia_type = "BitsTypeUInt64")]
pub struct BitsTypeUInt64 {
    pub a: u64,
}

#[repr(C)]
#[derive(
    Clone,
    Debug,
    Unbox,
    ValidLayout,
    Typecheck,
    IntoJulia,
    ValidField,
    IsBits,
    ConstructType,
    CCallArg,
    CCallReturn,
)]
#[jlrs(julia_type = "BitsTypeUInt8")]
pub struct BitsTypeUInt8 {
    pub a: u8,
}

#[repr(C)]
#[derive(
    Clone,
    Debug,
    Unbox,
    ValidLayout,
    Typecheck,
    IntoJulia,
    ValidField,
    IsBits,
    ConstructType,
    CCallArg,
    CCallReturn,
)]
#[jlrs(julia_type = "BitsUInt8TupleInt32Int64")]
pub struct BitsUInt8TupleInt32Int64 {
    pub a: u8,
    pub b: ::jlrs::data::layout::tuple::Tuple2<i32, i64>,
}

#[repr(C)]
#[derive(
    Clone,
    Debug,
    Unbox,
    ValidLayout,
    Typecheck,
    IntoJulia,
    ValidField,
    IsBits,
    ConstructType,
    CCallArg,
    CCallReturn,
)]
#[jlrs(julia_type = "BitsUInt8TupleInt32TupleInt16UInt16")]
pub struct BitsUInt8TupleInt32TupleInt16UInt16 {
    pub a: u8,
    pub b: ::jlrs::data::layout::tuple::Tuple2<i32, ::jlrs::data::layout::tuple::Tuple2<i16, u16>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck)]
#[jlrs(julia_type = "DoubleHasGeneric")]
pub struct DoubleHasGeneric<'scope, 'data> {
    pub a: ::std::option::Option<::jlrs::data::managed::value::WeakValue<'scope, 'data>>,
}

#[derive(ConstructType, HasLayout)]
#[jlrs(julia_type = "DoubleHasGeneric", constructor_for = "DoubleHasGeneric", scope_lifetime = true, data_lifetime = true, layout_params = [], elided_params = ["T"], all_params = ["T"])]
pub struct DoubleHasGenericTypeConstructor<T> {
    _t: ::std::marker::PhantomData<T>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "DoubleImmut")]
pub struct DoubleImmut<'scope, 'data> {
    pub a: Immut<'scope, 'data>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "DoubleVariant")]
pub struct DoubleVariant {
    #[jlrs(bits_union_align)]
    _a_align: ::jlrs::data::layout::union::Align4,
    #[jlrs(bits_union)]
    pub a: ::jlrs::data::layout::union::BitsUnion<4>,
    #[jlrs(bits_union_flag)]
    pub a_flag: u8,
}

#[repr(C)]
#[derive(
    Clone, Debug, Unbox, ValidLayout, Typecheck, IntoJulia, ValidField, IsBits, ConstructType,
)]
#[jlrs(julia_type = "Empty", zero_sized_type)]
pub struct Empty {}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "HasAbstractField")]
pub struct HasAbstractField<'scope, 'data> {
    pub a: ::std::option::Option<::jlrs::data::managed::value::WeakValue<'scope, 'data>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "HasAbstractUnionAllField")]
pub struct HasAbstractUnionAllField<'scope, 'data> {
    pub a: ::std::option::Option<::jlrs::data::managed::value::WeakValue<'scope, 'data>>,
}

#[derive(ConstructType)]
#[jlrs(julia_type = "HasAtomicField")]
pub struct HasAtomicFieldTypeConstructor {}

#[derive(ConstructType)]
#[jlrs(julia_type = "HasCustomAtomicField")]
pub struct HasCustomAtomicFieldTypeConstructor {}

#[repr(C)]
#[derive(
    Clone,
    Debug,
    Unbox,
    ValidLayout,
    Typecheck,
    ValidField,
    IsBits,
    ConstructType,
    CCallArg,
    CCallReturn,
)]
#[jlrs(julia_type = "HasGenericAbstractField")]
pub struct HasGenericAbstractField<T> {
    pub a: T,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, IsBits)]
#[jlrs(julia_type = "HasGenericAbstractUnionAllField")]
pub struct HasGenericAbstractUnionAllField<U> {
    pub a: U,
}

#[derive(ConstructType, HasLayout)]
#[jlrs(julia_type = "HasGenericAbstractUnionAllField", constructor_for = "HasGenericAbstractUnionAllField", scope_lifetime = false, data_lifetime = false, layout_params = ["U"], elided_params = ["T"], all_params = ["T", "U"])]
pub struct HasGenericAbstractUnionAllFieldTypeConstructor<T, U> {
    _t: ::std::marker::PhantomData<T>,
    _u: ::std::marker::PhantomData<U>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField)]
#[jlrs(julia_type = "HasGenericImmut")]
pub struct HasGenericImmut<'scope, 'data> {
    pub a: ::std::option::Option<::jlrs::data::managed::value::WeakValue<'scope, 'data>>,
}

#[derive(ConstructType, HasLayout)]
#[jlrs(julia_type = "HasGenericImmut", constructor_for = "HasGenericImmut", scope_lifetime = true, data_lifetime = true, layout_params = [], elided_params = ["T"], all_params = ["T"])]
pub struct HasGenericImmutTypeConstructor<T> {
    _t: ::std::marker::PhantomData<T>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ConstructType)]
#[jlrs(julia_type = "HasGeneric")]
pub struct HasGeneric<T> {
    pub a: T,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ConstructType)]
#[jlrs(julia_type = "HasImmut")]
pub struct HasImmut<'scope, 'data> {
    pub a: Immut<'scope, 'data>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "Immut")]
pub struct Immut<'scope, 'data> {
    pub a: ::std::option::Option<::jlrs::data::managed::value::WeakValue<'scope, 'data>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ConstructType)]
#[jlrs(julia_type = "MutF32")]
pub struct MutF32 {
    pub a: f32,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ConstructType)]
#[jlrs(julia_type = "MutNested")]
pub struct MutNested<'scope, 'data> {
    pub a: ::std::option::Option<::jlrs::data::managed::value::WeakValue<'scope, 'data>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "NonBitsUnion")]
pub struct NonBitsUnion<'scope, 'data> {
    pub a: ::std::option::Option<::jlrs::data::managed::value::WeakValue<'scope, 'data>>,
}

#[repr(C)]
#[derive(
    Clone,
    Debug,
    Unbox,
    ValidLayout,
    Typecheck,
    IntoJulia,
    ValidField,
    IsBits,
    ConstructType,
    CCallArg,
    CCallReturn,
)]
#[jlrs(julia_type = "SingleVariant")]
pub struct SingleVariant {
    pub a: i32,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "SizeAlignMismatch")]
pub struct SizeAlignMismatch {
    #[jlrs(bits_union_align)]
    _a_align: ::jlrs::data::layout::union::Align4,
    #[jlrs(bits_union)]
    pub a: ::jlrs::data::layout::union::BitsUnion<6>,
    #[jlrs(bits_union_flag)]
    pub a_flag: u8,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, IsBits)]
#[jlrs(julia_type = "TypedEmpty")]
pub struct TypedEmpty {}

#[derive(ConstructType, HasLayout)]
#[jlrs(julia_type = "TypedEmpty", constructor_for = "TypedEmpty", scope_lifetime = false, data_lifetime = false, layout_params = [], elided_params = ["T"], all_params = ["T"])]
pub struct TypedEmptyTypeConstructor<T> {
    _t: ::std::marker::PhantomData<T>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "UnionInTuple")]
pub struct UnionInTuple<'scope, 'data> {
    pub a: ::std::option::Option<::jlrs::data::managed::value::WeakValue<'scope, 'data>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "WithArray")]
pub struct WithArray<'scope, 'data> {
    pub a: ::std::option::Option<::jlrs::data::managed::array::WeakArray<'scope, 'data>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "WithCodeInstance")]
pub struct WithCodeInstance<'scope, 'data> {
    pub a: ::std::option::Option<::jlrs::data::managed::value::WeakValue<'scope, 'data>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "WithDataType")]
pub struct WithDataType<'scope> {
    pub a: ::std::option::Option<::jlrs::data::managed::datatype::WeakDataType<'scope>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "WithExpr")]
pub struct WithExpr<'scope> {
    pub a: ::std::option::Option<::jlrs::data::managed::expr::WeakExpr<'scope>>,
}

#[repr(C)]
#[derive(
    Clone,
    Debug,
    Unbox,
    ValidLayout,
    Typecheck,
    ValidField,
    IsBits,
    ConstructType,
    CCallArg,
    CCallReturn,
)]
#[jlrs(julia_type = "WithGenericT")]
pub struct WithGenericT<T> {
    pub a: T,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "WithGenericUnionAll")]
pub struct WithGenericUnionAll<'scope, 'data> {
    pub a: ::std::option::Option<::jlrs::data::managed::value::WeakValue<'scope, 'data>>,
}

#[repr(C)]
#[derive(
    Clone,
    Debug,
    Unbox,
    ValidLayout,
    Typecheck,
    IntoJulia,
    ValidField,
    IsBits,
    ConstructType,
    CCallArg,
    CCallReturn,
)]
#[jlrs(julia_type = "WithInt32")]
pub struct WithInt32 {
    pub int32: i32,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "WithMethod")]
pub struct WithMethod<'scope, 'data> {
    pub a: ::std::option::Option<::jlrs::data::managed::value::WeakValue<'scope, 'data>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "WithMethodInstance")]
pub struct WithMethodInstance<'scope, 'data> {
    pub a: ::std::option::Option<::jlrs::data::managed::value::WeakValue<'scope, 'data>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "WithMethodTable")]
pub struct WithMethodTable<'scope, 'data> {
    pub a: ::std::option::Option<::jlrs::data::managed::value::WeakValue<'scope, 'data>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "WithModule")]
pub struct WithModule<'scope> {
    pub a: ::std::option::Option<::jlrs::data::managed::module::WeakModule<'scope>>,
}

#[repr(C)]
#[derive(
    Clone,
    Debug,
    Unbox,
    ValidLayout,
    Typecheck,
    ValidField,
    IsBits,
    ConstructType,
    CCallArg,
    CCallReturn,
)]
#[jlrs(julia_type = "WithNestedGenericT")]
pub struct WithNestedGenericT<T> {
    pub a: WithGenericT<T>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "WithPropagatedLifetime")]
pub struct WithPropagatedLifetime<'scope> {
    pub a: WithGenericT<::std::option::Option<::jlrs::data::managed::module::WeakModule<'scope>>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "WithPropagatedLifetimes")]
pub struct WithPropagatedLifetimes<'scope, 'data> {
    pub a: WithGenericT<
        ::jlrs::data::layout::tuple::Tuple2<
            i32,
            WithGenericT<
                ::std::option::Option<::jlrs::data::managed::array::WeakArray<'scope, 'data>>,
            >,
        >,
    >,
}

#[repr(C)]
#[derive(
    Clone,
    Debug,
    Unbox,
    ValidLayout,
    Typecheck,
    IntoJulia,
    ValidField,
    IsBits,
    ConstructType,
    CCallArg,
    CCallReturn,
)]
#[jlrs(julia_type = "WithSetGeneric")]
pub struct WithSetGeneric {
    pub a: WithGenericT<i64>,
}

#[repr(C)]
#[derive(
    Clone,
    Debug,
    Unbox,
    ValidLayout,
    Typecheck,
    IntoJulia,
    ValidField,
    IsBits,
    ConstructType,
    CCallArg,
    CCallReturn,
)]
#[jlrs(julia_type = "WithSetGenericTuple")]
pub struct WithSetGenericTuple {
    pub a: ::jlrs::data::layout::tuple::Tuple1<WithGenericT<i64>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "WithSimpleVector")]
pub struct WithSimpleVector<'scope> {
    pub a: ::std::option::Option<::jlrs::data::managed::simple_vector::WeakSimpleVector<'scope>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "WithString")]
pub struct WithString<'scope> {
    pub a: ::std::option::Option<::jlrs::data::managed::string::WeakString<'scope>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "WithSymbol")]
pub struct WithSymbol<'scope> {
    pub a: ::std::option::Option<::jlrs::data::managed::symbol::WeakSymbol<'scope>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "WithTypeMapEntry")]
pub struct WithTypeMapEntry<'scope, 'data> {
    pub a: ::std::option::Option<::jlrs::data::managed::value::WeakValue<'scope, 'data>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "WithTypeMapLevel")]
pub struct WithTypeMapLevel<'scope, 'data> {
    pub a: ::std::option::Option<::jlrs::data::managed::value::WeakValue<'scope, 'data>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "WithTypeName")]
pub struct WithTypeName<'scope> {
    pub a: ::std::option::Option<::jlrs::data::managed::type_name::WeakTypeName<'scope>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "WithTypeVar")]
pub struct WithTypeVar<'scope> {
    pub a: ::std::option::Option<::jlrs::data::managed::type_var::WeakTypeVar<'scope>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "WithUnion")]
pub struct WithUnion<'scope> {
    pub a: ::std::option::Option<::jlrs::data::managed::union::WeakUnion<'scope>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "WithUnionAll")]
pub struct WithUnionAll<'scope> {
    pub a: ::std::option::Option<::jlrs::data::managed::union_all::WeakUnionAll<'scope>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, IsBits)]
#[jlrs(julia_type = "WithValueType")]
pub struct WithValueType {
    pub a: i64,
}

#[derive(ConstructType, HasLayout)]
#[jlrs(julia_type = "WithValueType", constructor_for = "WithValueType", scope_lifetime = false, data_lifetime = false, layout_params = [], elided_params = ["N"], all_params = ["N"])]
pub struct WithValueTypeTypeConstructor<N> {
    _n: ::std::marker::PhantomData<N>,
}

#[repr(C)]
#[derive(
    Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg, CCallReturn,
)]
#[jlrs(julia_type = "DoubleUVariant")]
pub struct DoubleUVariant {
    #[jlrs(bits_union_align)]
    _a_align: ::jlrs::data::layout::union::Align4,
    #[jlrs(bits_union)]
    pub a: ::jlrs::data::layout::union::TypedBitsUnion<::jlrs::UnionOf![u16, u32], 4>,
    #[jlrs(bits_union_flag)]
    pub a_flag: u8,
}

#[repr(C)]
#[derive(
    Clone,
    Debug,
    Unbox,
    ValidLayout,
    Typecheck,
    ValidField,
    IsBits,
    ConstructType,
    CCallArg,
    CCallReturn,
    PartialEq,
)]
#[jlrs(julia_type = "WithGenericTU")]
pub struct WithGenericTU<T, U> {
    pub a: T,
    pub b: U,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, IsBits)]
#[jlrs(julia_type = "HasElidedParam")]
pub struct HasElidedParam<T> {
    pub a: T,
}

#[derive(ConstructType)]
#[jlrs(julia_type = "HasElidedParam")]
pub struct HasElidedParamTypeConstructor<T, U> {
    _t: ::std::marker::PhantomData<T>,
    _u: ::std::marker::PhantomData<U>,
}

#[derive(
    Copy,
    Clone,
    Debug,
    PartialEq,
    Enum,
    Unbox,
    IntoJulia,
    ConstructType,
    IsBits,
    Typecheck,
    ValidField,
    ValidLayout,
    CCallArg,
    CCallReturn,
)]
#[jlrs(julia_type = "StandardEnum")]
#[repr(i32)]
pub enum StandardEnum {
    #[allow(non_camel_case_types)]
    #[jlrs(julia_enum_variant = "se_a")]
    SeA = 1,
    #[allow(non_camel_case_types)]
    #[jlrs(julia_enum_variant = "se_b")]
    SeB = 2,
    #[allow(non_camel_case_types)]
    #[jlrs(julia_enum_variant = "se_c")]
    SeC = 3,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, IsBits)]
#[jlrs(julia_type = "Elided")]
pub struct Elided<B> {
    pub a: B,
}

#[derive(ConstructType, HasLayout)]
#[jlrs(julia_type = "Elided", constructor_for = "Elided", scope_lifetime = false, data_lifetime = false, layout_params = ["B"], elided_params = ["A"], all_params = ["A", "B"])]
pub struct ElidedTypeConstructor<A, B> {
    _a: ::std::marker::PhantomData<A>,
    _b: ::std::marker::PhantomData<B>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "WithElidedInUnion")]
pub struct WithElidedInUnion {
    #[jlrs(bits_union_align)]
    _a_align: ::jlrs::data::layout::union::Align8,
    #[jlrs(bits_union)]
    pub a: ::jlrs::data::layout::union::TypedBitsUnion<
        ::jlrs::UnionOf![
            f64,
            i16,
            ElidedTypeConstructor<
                ::jlrs::data::types::construct_type::ConstantBool<true>,
                ElidedTypeConstructor<::jlrs::data::types::construct_type::ConstantI64<1>, i64>,
            >
        ],
        8,
    >,
    #[jlrs(bits_union_flag)]
    pub a_flag: u8,
}
