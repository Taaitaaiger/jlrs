// TODO
#![allow(unused)]

use jlrs::prelude::*;

#[derive(ConstructType)]
#[jlrs(julia_type = "Main.AnAbstractType")]
pub struct AnAbstractType {}

#[derive(ConstructType)]
#[jlrs(julia_type = "Main.AnAbstractUnionAll")]
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
#[jlrs(julia_type = "Main.BitsCharBitsIntChar")]
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
#[jlrs(julia_type = "Main.BitsCharFloat32Float64")]
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
#[jlrs(julia_type = "Main.BitsIntBool")]
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
#[jlrs(julia_type = "Main.BitsIntChar")]
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
#[jlrs(julia_type = "Main.BitsTypeBool")]
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
#[jlrs(julia_type = "Main.BitsTypeChar")]
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
#[jlrs(julia_type = "Main.BitsTypeFloat32")]
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
#[jlrs(julia_type = "Main.BitsTypeFloat64")]
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
#[jlrs(julia_type = "Main.BitsTypeInt")]
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
#[jlrs(julia_type = "Main.BitsTypeInt16")]
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
#[jlrs(julia_type = "Main.BitsTypeInt32")]
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
#[jlrs(julia_type = "Main.BitsTypeInt64")]
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
#[jlrs(julia_type = "Main.BitsTypeInt8")]
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
#[jlrs(julia_type = "Main.BitsTypeUInt")]
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
#[jlrs(julia_type = "Main.BitsTypeUInt16")]
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
#[jlrs(julia_type = "Main.BitsTypeUInt32")]
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
#[jlrs(julia_type = "Main.BitsTypeUInt64")]
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
#[jlrs(julia_type = "Main.BitsTypeUInt8")]
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
#[jlrs(julia_type = "Main.BitsUInt8TupleInt32Int64")]
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
#[jlrs(julia_type = "Main.BitsUInt8TupleInt32TupleInt16UInt16")]
pub struct BitsUInt8TupleInt32TupleInt16UInt16 {
    pub a: u8,
    pub b: ::jlrs::data::layout::tuple::Tuple2<i32, ::jlrs::data::layout::tuple::Tuple2<i16, u16>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck)]
#[jlrs(julia_type = "Main.DoubleHasGeneric")]
pub struct DoubleHasGeneric<'scope, 'data> {
    pub a: ::std::option::Option<::jlrs::data::managed::value::WeakValue<'scope, 'data>>,
}

#[derive(ConstructType, HasLayout)]
#[jlrs(julia_type = "Main.DoubleHasGeneric", constructor_for = "DoubleHasGeneric", scope_lifetime = true, data_lifetime = true, layout_params = [], elided_params = ["T"], all_params = ["T"])]
pub struct DoubleHasGenericTypeConstructor<T> {
    _t: ::std::marker::PhantomData<T>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.DoubleImmut")]
pub struct DoubleImmut<'scope, 'data> {
    pub a: Immut<'scope, 'data>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
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
#[derive(
    Clone, Debug, Unbox, ValidLayout, Typecheck, IntoJulia, ValidField, IsBits, ConstructType,
)]
#[jlrs(julia_type = "Main.Empty", zero_sized_type)]
pub struct Empty {}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.HasAbstractField")]
pub struct HasAbstractField<'scope, 'data> {
    pub a: ::std::option::Option<::jlrs::data::managed::value::WeakValue<'scope, 'data>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.HasAbstractUnionAllField")]
pub struct HasAbstractUnionAllField<'scope, 'data> {
    pub a: ::std::option::Option<::jlrs::data::managed::value::WeakValue<'scope, 'data>>,
}

#[derive(ConstructType)]
#[jlrs(julia_type = "Main.HasAtomicField")]
pub struct HasAtomicFieldTypeConstructor {}

#[derive(ConstructType)]
#[jlrs(julia_type = "Main.HasCustomAtomicField")]
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
#[jlrs(julia_type = "Main.HasGenericAbstractField")]
pub struct HasGenericAbstractField<T> {
    pub a: T,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, IsBits)]
#[jlrs(julia_type = "Main.HasGenericAbstractUnionAllField")]
pub struct HasGenericAbstractUnionAllField<U> {
    pub a: U,
}

#[derive(ConstructType, HasLayout)]
#[jlrs(julia_type = "Main.HasGenericAbstractUnionAllField", constructor_for = "HasGenericAbstractUnionAllField", scope_lifetime = false, data_lifetime = false, layout_params = ["U"], elided_params = ["T"], all_params = ["T", "U"])]
pub struct HasGenericAbstractUnionAllFieldTypeConstructor<T, U> {
    _t: ::std::marker::PhantomData<T>,
    _u: ::std::marker::PhantomData<U>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField)]
#[jlrs(julia_type = "Main.HasGenericImmut")]
pub struct HasGenericImmut<'scope, 'data> {
    pub a: ::std::option::Option<::jlrs::data::managed::value::WeakValue<'scope, 'data>>,
}

#[derive(ConstructType, HasLayout)]
#[jlrs(julia_type = "Main.HasGenericImmut", constructor_for = "HasGenericImmut", scope_lifetime = true, data_lifetime = true, layout_params = [], elided_params = ["T"], all_params = ["T"])]
pub struct HasGenericImmutTypeConstructor<T> {
    _t: ::std::marker::PhantomData<T>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ConstructType)]
#[jlrs(julia_type = "Main.HasGeneric")]
pub struct HasGeneric<T> {
    pub a: T,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ConstructType)]
#[jlrs(julia_type = "Main.HasImmut")]
pub struct HasImmut<'scope, 'data> {
    pub a: Immut<'scope, 'data>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.Immut")]
pub struct Immut<'scope, 'data> {
    pub a: ::std::option::Option<::jlrs::data::managed::value::WeakValue<'scope, 'data>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ConstructType)]
#[jlrs(julia_type = "Main.MutF32")]
pub struct MutF32 {
    pub a: f32,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ConstructType)]
#[jlrs(julia_type = "Main.MutNested")]
pub struct MutNested<'scope, 'data> {
    pub a: ::std::option::Option<::jlrs::data::managed::value::WeakValue<'scope, 'data>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.NonBitsUnion")]
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
#[jlrs(julia_type = "Main.SingleVariant")]
pub struct SingleVariant {
    pub a: i32,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
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
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, IsBits)]
#[jlrs(julia_type = "Main.TypedEmpty")]
pub struct TypedEmpty {}

#[derive(ConstructType, HasLayout)]
#[jlrs(julia_type = "Main.TypedEmpty", constructor_for = "TypedEmpty", scope_lifetime = false, data_lifetime = false, layout_params = [], elided_params = ["T"], all_params = ["T"])]
pub struct TypedEmptyTypeConstructor<T> {
    _t: ::std::marker::PhantomData<T>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.UnionInTuple")]
pub struct UnionInTuple<'scope, 'data> {
    pub a: ::std::option::Option<::jlrs::data::managed::value::WeakValue<'scope, 'data>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.WithArray")]
pub struct WithArray<'scope, 'data> {
    pub a: ::std::option::Option<::jlrs::data::managed::array::WeakArray<'scope, 'data>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.WithCodeInstance")]
pub struct WithCodeInstance<'scope, 'data> {
    pub a: ::std::option::Option<::jlrs::data::managed::value::WeakValue<'scope, 'data>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.WithDataType")]
pub struct WithDataType<'scope> {
    pub a: ::std::option::Option<::jlrs::data::managed::datatype::WeakDataType<'scope>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.WithExpr")]
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
#[jlrs(julia_type = "Main.WithGenericT")]
pub struct WithGenericT<T> {
    pub a: T,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.WithGenericUnionAll")]
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
#[jlrs(julia_type = "Main.WithInt32")]
pub struct WithInt32 {
    pub int32: i32,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.WithMethod")]
pub struct WithMethod<'scope, 'data> {
    pub a: ::std::option::Option<::jlrs::data::managed::value::WeakValue<'scope, 'data>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.WithMethodInstance")]
pub struct WithMethodInstance<'scope, 'data> {
    pub a: ::std::option::Option<::jlrs::data::managed::value::WeakValue<'scope, 'data>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.WithMethodTable")]
pub struct WithMethodTable<'scope, 'data> {
    pub a: ::std::option::Option<::jlrs::data::managed::value::WeakValue<'scope, 'data>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.WithModule")]
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
#[jlrs(julia_type = "Main.WithNestedGenericT")]
pub struct WithNestedGenericT<T> {
    pub a: WithGenericT<T>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.WithPropagatedLifetime")]
pub struct WithPropagatedLifetime<'scope> {
    pub a: WithGenericT<::std::option::Option<::jlrs::data::managed::module::WeakModule<'scope>>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.WithPropagatedLifetimes")]
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
#[jlrs(julia_type = "Main.WithSetGeneric")]
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
#[jlrs(julia_type = "Main.WithSetGenericTuple")]
pub struct WithSetGenericTuple {
    pub a: ::jlrs::data::layout::tuple::Tuple1<WithGenericT<i64>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.WithSimpleVector")]
pub struct WithSimpleVector<'scope> {
    pub a: ::std::option::Option<::jlrs::data::managed::simple_vector::WeakSimpleVector<'scope>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.WithString")]
pub struct WithString<'scope> {
    pub a: ::std::option::Option<::jlrs::data::managed::string::WeakString<'scope>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.WithSymbol")]
pub struct WithSymbol<'scope> {
    pub a: ::std::option::Option<::jlrs::data::managed::symbol::WeakSymbol<'scope>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.WithTypeMapEntry")]
pub struct WithTypeMapEntry<'scope, 'data> {
    pub a: ::std::option::Option<::jlrs::data::managed::value::WeakValue<'scope, 'data>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.WithTypeMapLevel")]
pub struct WithTypeMapLevel<'scope, 'data> {
    pub a: ::std::option::Option<::jlrs::data::managed::value::WeakValue<'scope, 'data>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.WithTypeName")]
pub struct WithTypeName<'scope> {
    pub a: ::std::option::Option<::jlrs::data::managed::type_name::WeakTypeName<'scope>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.WithTypeVar")]
pub struct WithTypeVar<'scope> {
    pub a: ::std::option::Option<::jlrs::data::managed::type_var::WeakTypeVar<'scope>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.WithUnion")]
pub struct WithUnion<'scope> {
    pub a: ::std::option::Option<::jlrs::data::managed::union::WeakUnion<'scope>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.WithUnionAll")]
pub struct WithUnionAll<'scope> {
    pub a: ::std::option::Option<::jlrs::data::managed::union_all::WeakUnionAll<'scope>>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, IsBits)]
#[jlrs(julia_type = "Main.WithValueType")]
pub struct WithValueType {
    pub a: i64,
}

#[derive(ConstructType, HasLayout)]
#[jlrs(julia_type = "Main.WithValueType", constructor_for = "WithValueType", scope_lifetime = false, data_lifetime = false, layout_params = [], elided_params = ["N"], all_params = ["N"])]
pub struct WithValueTypeTypeConstructor<N> {
    _n: ::std::marker::PhantomData<N>,
}

#[repr(C)]
#[derive(
    Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg, CCallReturn,
)]
#[jlrs(julia_type = "Main.DoubleUVariant")]
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
#[jlrs(julia_type = "Main.WithGenericTU")]
pub struct WithGenericTU<T, U> {
    pub a: T,
    pub b: U,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, IsBits)]
#[jlrs(julia_type = "Main.HasElidedParam")]
pub struct HasElidedParam<T> {
    pub a: T,
}

#[derive(ConstructType)]
#[jlrs(julia_type = "Main.HasElidedParam")]
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
#[jlrs(julia_type = "Main.StandardEnum")]
#[repr(i32)]
pub enum StandardEnum {
    #[allow(non_camel_case_types)]
    #[jlrs(julia_enum_variant = "Main.se_a")]
    SeA = 1,
    #[allow(non_camel_case_types)]
    #[jlrs(julia_enum_variant = "Main.se_b")]
    SeB = 2,
    #[allow(non_camel_case_types)]
    #[jlrs(julia_enum_variant = "Main.se_c")]
    SeC = 3,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, IsBits)]
#[jlrs(julia_type = "Main.Elided")]
pub struct Elided<B> {
    pub a: B,
}

#[derive(ConstructType, HasLayout)]
#[jlrs(julia_type = "Main.Elided", constructor_for = "Elided", scope_lifetime = false, data_lifetime = false, layout_params = ["B"], elided_params = ["A"], all_params = ["A", "B"])]
pub struct ElidedTypeConstructor<A, B> {
    _a: ::std::marker::PhantomData<A>,
    _b: ::std::marker::PhantomData<B>,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "Main.WithElidedInUnion")]
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
