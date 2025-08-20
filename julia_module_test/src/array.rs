use jlrs::{
    data::{
        managed::{
            array::{ArrayRet, RankedArrayRet, TypedArrayRet, TypedRankedArrayRet},
            value::{typed::TypedValue, ValueRet},
        },
        types::{
            abstract_type::Integer,
            construct_type::{ArrayTypeConstructor, ConstantIsize},
        },
    },
    prelude::{
        Array, ConstructTypedArray, DataType, Managed, RankedArray, TypedArray, TypedRankedArray,
    },
    tvar, tvars, weak_handle_unchecked,
};

// Array arguments
pub fn takes_array(a: Array) -> usize {
    let elty = a.element_type();

    if let Ok(elty) = elty.cast::<DataType>() {
        elty.size().unwrap_or(std::mem::size_of::<usize>() as _) as _
    } else {
        std::mem::size_of::<usize>()
    }
}

pub fn takes_ranked_array(a: RankedArray<1>) -> usize {
    let elty = a.element_type();

    if let Ok(elty) = elty.cast::<DataType>() {
        elty.size().unwrap_or(std::mem::size_of::<usize>() as _) as _
    } else {
        std::mem::size_of::<usize>()
    }
}

pub fn takes_typed_array(a: TypedArray<u32>) -> usize {
    let elty = a.element_type();

    if let Ok(elty) = elty.cast::<DataType>() {
        elty.size().unwrap_or(std::mem::size_of::<usize>() as _) as _
    } else {
        std::mem::size_of::<usize>()
    }
}

pub fn takes_typed_ranked_array(a: TypedRankedArray<u32, 1>) -> usize {
    let elty = a.element_type();

    if let Ok(elty) = elty.cast::<DataType>() {
        elty.size().unwrap_or(std::mem::size_of::<usize>() as _) as _
    } else {
        std::mem::size_of::<usize>()
    }
}

// Array return type
pub unsafe fn returns_array(dt: DataType) -> ArrayRet {
    let weak_handle = weak_handle_unchecked!();
    Array::new_for(weak_handle, dt.as_value(), [2, 2])
        .unwrap()
        .leak()
}

pub unsafe fn returns_rank0_array(dt: DataType) -> RankedArrayRet<0> {
    let weak_handle = weak_handle_unchecked!();
    RankedArray::<0>::new_for(weak_handle, dt.as_value(), [])
        .unwrap()
        .leak()
}

pub unsafe fn returns_rank1_array(dt: DataType) -> RankedArrayRet<1> {
    let weak_handle = weak_handle_unchecked!();
    RankedArray::<1>::new_for(weak_handle, dt.as_value(), [2])
        .unwrap()
        .leak()
}

pub unsafe fn returns_rank2_array(dt: DataType) -> RankedArrayRet<2> {
    let weak_handle = weak_handle_unchecked!();
    RankedArray::<2>::new_for(weak_handle, dt.as_value(), [2, 2])
        .unwrap()
        .leak()
}

pub unsafe fn returns_rank3_array(dt: DataType) -> RankedArrayRet<3> {
    let weak_handle = weak_handle_unchecked!();
    RankedArray::<3>::new_for(weak_handle, dt.as_value(), [2, 2, 2])
        .unwrap()
        .leak()
}

pub unsafe fn returns_typed_array() -> TypedArrayRet<f32> {
    let weak_handle = weak_handle_unchecked!();
    TypedArray::<f32>::new(weak_handle, [2, 2]).unwrap().leak()
}

pub unsafe fn returns_typed_rank2_array() -> TypedRankedArrayRet<f32, 2> {
    let weak_handle = weak_handle_unchecked!();
    TypedRankedArray::<f32, 2>::new(weak_handle, [2, 2])
        .unwrap()
        .leak()
}

// Generic arrays
pub type TEnv = tvars!(tvar!('T'));
pub fn takes_generic_typed_ranked_arrays_ctor(
    a: TypedValue<ArrayTypeConstructor<tvar!('T'), ConstantIsize<1>>>,
    _b: TypedValue<ArrayTypeConstructor<tvar!('T'), ConstantIsize<2>>>,
) -> ValueRet {
    a.as_typed_ranked_array().element_type().as_weak().leak()
}

pub type TMNEnv = tvars!(tvar!('T'), tvar!('M'), tvar!('N'));
pub fn takes_generic_typed_arrays_ctor(
    a: TypedValue<ArrayTypeConstructor<tvar!('T'), tvar!('M')>>,
    _b: TypedValue<ArrayTypeConstructor<tvar!('T'), tvar!('N')>>,
) -> ValueRet {
    a.as_typed_array().element_type().as_weak().leak()
}

pub type UEnv = tvars!(tvar!('U'));
pub fn takes_generic_typed_ranked_arrays(
    a: TypedRankedArray<tvar!('U'), 1>,
    _b: TypedRankedArray<tvar!('U'), 2>,
) -> ValueRet {
    a.element_type().as_weak().leak()
}

pub fn takes_generic_typed_arrays(
    a: TypedArray<tvar!('U')>,
    _b: TypedArray<tvar!('U')>,
) -> ValueRet {
    a.element_type().as_weak().leak()
}

pub fn takes_and_returns_generic_typed_ranked_array(
    a: TypedRankedArray<'_, 'static, tvar!('U'), 1>,
) -> TypedRankedArrayRet<tvar!('U'), 1> {
    a.as_weak().leak()
}

pub type RestrictedEnv = tvars!(tvar!('T'; Integer));
pub fn takes_restricted_generic_typed_arrays(a: TypedArray<tvar!('T')>) -> ValueRet {
    a.element_type().as_weak().leak()
}
