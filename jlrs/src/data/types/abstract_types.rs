//! Abstract Julia types
//!
//! These types can be used with the `ConstructType` trait.

use std::{marker::PhantomData, ptr::NonNull};

use jl_sys::{jl_any_type, jl_value_t};
use jlrs_macros::julia_version;

use super::construct_type::ConstructType;
use crate::{
    data::managed::{
        datatype::DataType, module::Module, type_var::TypeVar, union_all::UnionAll,
        value::ValueData, Managed,
    },
    memory::target::{ExtendedTarget, Target},
    private::Private,
};

/// Marker trait that must only be implemented by type constructors that build an abstract type.
pub unsafe trait AbstractType: ConstructType {}

macro_rules! impl_construct_julia_type_abstract {
    ($ty:ty, $mod:ident) => {
        unsafe impl ConstructType for $ty {
            fn construct_type<'target, T>(
                target: ExtendedTarget<'target, '_, '_, T>,
            ) -> ValueData<'target, 'static, T>
            where
                T: Target<'target>,
            {
                let (target, _) = target.split();
                Module::$mod(&target)
                    .global(target, stringify!($ty))
                    .unwrap()
            }
        }
    };
}

// TODO: Use data defined in julia.h when possible
/// Construct a new `Core.AbstractChar` type object.
pub struct AbstractChar;
unsafe impl AbstractType for AbstractChar {}
impl_construct_julia_type_abstract!(AbstractChar, core);

/// Construct a new `Core.AbstractFloat` type object.
pub struct AbstractFloat;
unsafe impl AbstractType for AbstractFloat {}
impl_construct_julia_type_abstract!(AbstractFloat, core);

/// Construct a new `Core.AbstractString` type object.
pub struct AbstractString;
unsafe impl AbstractType for AbstractString {}
impl_construct_julia_type_abstract!(AbstractString, core);

/// Construct a new `Core.Exception` type object.
pub struct Exception;
unsafe impl AbstractType for Exception {}
impl_construct_julia_type_abstract!(Exception, core);

/// Construct a new `Core.Function` type object.
pub struct Function;
unsafe impl AbstractType for Function {}
impl_construct_julia_type_abstract!(Function, core);

/// Construct a new `Core.IO` type object.
pub struct IO;
unsafe impl AbstractType for IO {}
impl_construct_julia_type_abstract!(IO, core);

/// Construct a new `Core.Integer` type object.
pub struct Integer;
unsafe impl AbstractType for Integer {}
impl_construct_julia_type_abstract!(Integer, core);

/// Construct a new `Core.Real` type object.
pub struct Real;
unsafe impl AbstractType for Real {}
impl_construct_julia_type_abstract!(Real, core);

/// Construct a new `Core.Number` type object.
pub struct Number;
unsafe impl AbstractType for Number {}
impl_construct_julia_type_abstract!(Number, core);

/// Construct a new `Core.Signed` type object.
pub struct Signed;
unsafe impl AbstractType for Signed {}
impl_construct_julia_type_abstract!(Signed, core);

/// Construct a new `Core.Unsigned` type object.
pub struct Unsigned;
unsafe impl AbstractType for Unsigned {}
impl_construct_julia_type_abstract!(Unsigned, core);

/// Construct a new `Base.AbstractDisplay` type object.
pub struct AbstractDisplay;
unsafe impl AbstractType for AbstractDisplay {}
impl_construct_julia_type_abstract!(AbstractDisplay, base);

/// Construct a new `Base.AbstractIrrational` type object.
pub struct AbstractIrrational;
unsafe impl AbstractType for AbstractIrrational {}
impl_construct_julia_type_abstract!(AbstractIrrational, base);

/// Construct a new `Base.AbstractMatch` type object.
pub struct AbstractMatch;
unsafe impl AbstractType for AbstractMatch {}
impl_construct_julia_type_abstract!(AbstractMatch, base);

/// Construct a new `Base.AbstractPattern` type object.
pub struct AbstractPattern;
unsafe impl AbstractType for AbstractPattern {}
impl_construct_julia_type_abstract!(AbstractPattern, base);

/// Construct a new `Base.IndexStyle` type object.
pub struct IndexStyle;
unsafe impl AbstractType for IndexStyle {}
impl_construct_julia_type_abstract!(IndexStyle, base);

pub struct AnyType;
unsafe impl AbstractType for AnyType {}
unsafe impl ConstructType for AnyType {
    fn construct_type<'target, T>(
        target: ExtendedTarget<'target, '_, '_, T>,
    ) -> ValueData<'target, 'static, T>
    where
        T: Target<'target>,
    {
        let (target, _) = target.split();
        unsafe {
            let ptr = NonNull::new_unchecked(jl_any_type.cast::<jl_value_t>());
            target.data_from_ptr(ptr, Private)
        }
    }
}

/// Construct a new `AbstractArray` type object from the provided type parameters.
pub struct AbstractArray<T: ConstructType, N: ConstructType> {
    _type: PhantomData<T>,
    _rank: PhantomData<N>,
}

unsafe impl<T: ConstructType, N: ConstructType> AbstractType for AbstractArray<T, N> {}

unsafe impl<T: ConstructType, N: ConstructType> ConstructType for AbstractArray<T, N> {
    fn construct_type<'target, Tgt>(
        target: ExtendedTarget<'target, '_, '_, Tgt>,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        let (target, frame) = target.split();

        frame
            .scope(|mut frame| {
                let ty_param = T::construct_type(frame.as_extended_target());
                let rank_param = N::construct_type(frame.as_extended_target());
                let params = [ty_param, rank_param];
                unsafe {
                    let applied = UnionAll::abstractarray_type(&frame)
                        .as_value()
                        .apply_type_unchecked(&mut frame, params);
                    Ok(UnionAll::rewrap(
                        target.into_extended_target(&mut frame),
                        applied.cast_unchecked::<DataType>(),
                    ))
                }
            })
            .unwrap()
    }
}

/// Construct a new `DenseArray` type object from the provided type parameters.
pub struct DenseArray<T: ConstructType, N: ConstructType> {
    _type: PhantomData<T>,
    _rank: PhantomData<N>,
}

unsafe impl<T: ConstructType, N: ConstructType> AbstractType for DenseArray<T, N> {}

unsafe impl<T: ConstructType, N: ConstructType> ConstructType for DenseArray<T, N> {
    fn construct_type<'target, Tgt>(
        target: ExtendedTarget<'target, '_, '_, Tgt>,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        let (target, frame) = target.split();

        frame
            .scope(|mut frame| {
                let ty_param = T::construct_type(frame.as_extended_target());
                let rank_param = N::construct_type(frame.as_extended_target());
                let params = [ty_param, rank_param];
                unsafe {
                    let applied = UnionAll::densearray_type(&frame)
                        .as_value()
                        .apply_type_unchecked(&mut frame, params);
                    Ok(UnionAll::rewrap(
                        target.into_extended_target(&mut frame),
                        applied.cast_unchecked::<DataType>(),
                    ))
                }
            })
            .unwrap()
    }
}

/// Construct a new `Ref` type object from the provided type parameters.
pub struct RefTypeConstructor<T: ConstructType> {
    _type: PhantomData<T>,
}

unsafe impl<T: ConstructType> AbstractType for RefTypeConstructor<T> {}

unsafe impl<T: ConstructType> ConstructType for RefTypeConstructor<T> {
    fn construct_type<'target, Tgt>(
        target: ExtendedTarget<'target, '_, '_, Tgt>,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        let (target, frame) = target.split();

        frame
            .scope(|mut frame| {
                let ty_param = T::construct_type(frame.as_extended_target());
                let params = [ty_param];
                unsafe {
                    let applied = UnionAll::ref_type(&frame)
                        .as_value()
                        .apply_type_unchecked(&mut frame, params);
                    Ok(UnionAll::rewrap(
                        target.into_extended_target(&mut frame),
                        applied.cast_unchecked::<DataType>(),
                    ))
                }
            })
            .unwrap()
    }
}

/// Construct a new `Type` type object from the provided type parameters.
pub struct TypeTypeConstructor<T: ConstructType> {
    _type: PhantomData<T>,
}

unsafe impl<T: ConstructType> AbstractType for TypeTypeConstructor<T> {}

unsafe impl<T: ConstructType> ConstructType for TypeTypeConstructor<T> {
    fn construct_type<'target, Tgt>(
        target: ExtendedTarget<'target, '_, '_, Tgt>,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        let (target, frame) = target.split();

        frame
            .scope(|mut frame| {
                let ty_param = T::construct_type(frame.as_extended_target());
                let params = [ty_param];
                unsafe {
                    let applied = UnionAll::type_type(&frame)
                        .as_value()
                        .apply_type_unchecked(&mut frame, params);
                    Ok(UnionAll::rewrap(
                        target.into_extended_target(&mut frame),
                        applied.cast_unchecked::<DataType>(),
                    ))
                }
            })
            .unwrap()
    }
}

/// Construct a new `AbstractChannel` type object from the provided type parameters.
pub struct AbstractChannel<T: ConstructType> {
    _type: PhantomData<T>,
}

unsafe impl<T: ConstructType> AbstractType for AbstractChannel<T> {}

unsafe impl<T: ConstructType> ConstructType for AbstractChannel<T> {
    fn construct_type<'target, Tgt>(
        target: ExtendedTarget<'target, '_, '_, Tgt>,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        let (target, frame) = target.split();

        frame
            .scope(|mut frame| {
                let ty_param = T::construct_type(frame.as_extended_target());
                let params = [ty_param];
                unsafe {
                    let applied = Module::base(&frame)
                        .global(&frame, "AbstractChannel")
                        .unwrap()
                        .as_value()
                        .apply_type_unchecked(&mut frame, params);
                    Ok(UnionAll::rewrap(
                        target.into_extended_target(&mut frame),
                        applied.cast_unchecked::<DataType>(),
                    ))
                }
            })
            .unwrap()
    }
}

/// Construct a new `AbstractDict` type object from the provided type parameters.
pub struct AbstractDict<K: ConstructType, V: ConstructType> {
    _key: PhantomData<K>,
    _value: PhantomData<V>,
}

unsafe impl<T: ConstructType, K: ConstructType> AbstractType for AbstractDict<T, K> {}

unsafe impl<K: ConstructType, V: ConstructType> ConstructType for AbstractDict<K, V> {
    fn construct_type<'target, Tgt>(
        target: ExtendedTarget<'target, '_, '_, Tgt>,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        let (target, frame) = target.split();

        frame
            .scope(|mut frame| {
                let key_param = K::construct_type(frame.as_extended_target());
                let value_param = V::construct_type(frame.as_extended_target());
                let params = [key_param, value_param];
                unsafe {
                    let applied = Module::base(&frame)
                        .global(&frame, "AbstractDict")
                        .unwrap()
                        .as_value()
                        .apply_type_unchecked(&mut frame, params);
                    Ok(UnionAll::rewrap(
                        target.into_extended_target(&mut frame),
                        applied.cast_unchecked::<DataType>(),
                    ))
                }
            })
            .unwrap()
    }
}

/// Construct a new `AbstractMatrix` type object from the provided type parameters.
pub struct AbstractMatrix<T: ConstructType> {
    _type: PhantomData<T>,
}

unsafe impl<T: ConstructType> AbstractType for AbstractMatrix<T> {}

unsafe impl<T: ConstructType> ConstructType for AbstractMatrix<T> {
    fn construct_type<'target, Tgt>(
        target: ExtendedTarget<'target, '_, '_, Tgt>,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        let (target, frame) = target.split();

        frame
            .scope(|mut frame| {
                let ty_param = T::construct_type(frame.as_extended_target());
                let params = [ty_param];
                unsafe {
                    let applied = Module::base(&frame)
                        .global(&frame, "AbstractMatrix")
                        .unwrap()
                        .as_value()
                        .apply_type_unchecked(&mut frame, params);
                    Ok(UnionAll::rewrap(
                        target.into_extended_target(&mut frame),
                        applied.cast_unchecked::<DataType>(),
                    ))
                }
            })
            .unwrap()
    }
}

/// Construct a new `AbstractRange` type object from the provided type parameters.
pub struct AbstractRange<T: ConstructType> {
    _type: PhantomData<T>,
}

unsafe impl<T: ConstructType> AbstractType for AbstractRange<T> {}

unsafe impl<T: ConstructType> ConstructType for AbstractRange<T> {
    fn construct_type<'target, Tgt>(
        target: ExtendedTarget<'target, '_, '_, Tgt>,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        let (target, frame) = target.split();

        frame
            .scope(|mut frame| {
                let ty_param = T::construct_type(frame.as_extended_target());
                let params = [ty_param];
                unsafe {
                    let applied = Module::base(&frame)
                        .global(&frame, "AbstractRange")
                        .unwrap()
                        .as_value()
                        .apply_type_unchecked(&mut frame, params);
                    Ok(UnionAll::rewrap(
                        target.into_extended_target(&mut frame),
                        applied.cast_unchecked::<DataType>(),
                    ))
                }
            })
            .unwrap()
    }
}

/// Construct a new `AbstractSet` type object from the provided type parameters.
pub struct AbstractSet<T: ConstructType> {
    _type: PhantomData<T>,
}

unsafe impl<T: ConstructType> AbstractType for AbstractSet<T> {}

unsafe impl<T: ConstructType> ConstructType for AbstractSet<T> {
    fn construct_type<'target, Tgt>(
        target: ExtendedTarget<'target, '_, '_, Tgt>,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        let (target, frame) = target.split();

        frame
            .scope(|mut frame| {
                let ty_param = T::construct_type(frame.as_extended_target());
                let params = [ty_param];
                unsafe {
                    let applied = Module::base(&frame)
                        .global(&frame, "AbstractSet")
                        .unwrap()
                        .as_value()
                        .apply_type_unchecked(&mut frame, params);
                    Ok(UnionAll::rewrap(
                        target.into_extended_target(&mut frame),
                        applied.cast_unchecked::<DataType>(),
                    ))
                }
            })
            .unwrap()
    }
}

#[julia_version(since = "1.9")]
/// Construct a new `AbstractSlices` type object from the provided type parameters.
pub struct AbstractSlices<T: ConstructType, N: ConstructType> {
    _type: PhantomData<T>,
    _n: PhantomData<N>,
}

#[julia_version(since = "1.9")]
unsafe impl<T: ConstructType, N: ConstructType> AbstractType for AbstractSlices<T, N> {}

#[julia_version(since = "1.9")]
unsafe impl<T: ConstructType, N: ConstructType> ConstructType for AbstractSlices<T, N> {
    fn construct_type<'target, Tgt>(
        target: ExtendedTarget<'target, '_, '_, Tgt>,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        let (target, frame) = target.split();

        frame
            .scope(|mut frame| {
                let ty_param = T::construct_type(frame.as_extended_target());
                let n_param = N::construct_type(frame.as_extended_target());
                let params = [ty_param, n_param];
                unsafe {
                    let applied = Module::base(&frame)
                        .global(&frame, "AbstractSlices")
                        .unwrap()
                        .as_value()
                        .apply_type_unchecked(&mut frame, params);
                    Ok(UnionAll::rewrap(
                        target.into_extended_target(&mut frame),
                        applied.cast_unchecked::<DataType>(),
                    ))
                }
            })
            .unwrap()
    }
}

/// Construct a new `AbstractUnitRange` type object from the provided type parameters.
pub struct AbstractUnitRange<T: ConstructType> {
    _type: PhantomData<T>,
}

unsafe impl<T: ConstructType> ConstructType for AbstractUnitRange<T> {
    fn construct_type<'target, Tgt>(
        target: ExtendedTarget<'target, '_, '_, Tgt>,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        let (target, frame) = target.split();

        frame
            .scope(|mut frame| {
                let ty_param = T::construct_type(frame.as_extended_target());
                let params = [ty_param];
                unsafe {
                    let applied = Module::base(&frame)
                        .global(&frame, "AbstractUnitRange")
                        .unwrap()
                        .as_value()
                        .apply_type_unchecked(&mut frame, params);
                    Ok(UnionAll::rewrap(
                        target.into_extended_target(&mut frame),
                        applied.cast_unchecked::<DataType>(),
                    ))
                }
            })
            .unwrap()
    }
}

unsafe impl<T: ConstructType> AbstractType for AbstractUnitRange<T> {}

/// Construct a new `AbstractVector` type object from the provided type parameters.
pub struct AbstractVector<T: ConstructType> {
    _type: PhantomData<T>,
}

unsafe impl<T: ConstructType> AbstractType for AbstractVector<T> {}

unsafe impl<T: ConstructType> ConstructType for AbstractVector<T> {
    fn construct_type<'target, Tgt>(
        target: ExtendedTarget<'target, '_, '_, Tgt>,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        let (target, frame) = target.split();

        frame
            .scope(|mut frame| {
                let ty_param = T::construct_type(frame.as_extended_target());
                let params = [ty_param];
                unsafe {
                    let applied = Module::base(&frame)
                        .global(&frame, "AbstractVector")
                        .unwrap()
                        .as_value()
                        .apply_type_unchecked(&mut frame, params);
                    Ok(UnionAll::rewrap(
                        target.into_extended_target(&mut frame),
                        applied.cast_unchecked::<DataType>(),
                    ))
                }
            })
            .unwrap()
    }
}

/// Construct a new `DenseMatrix` type object from the provided type parameters.
pub struct DenseMatrix<T: ConstructType> {
    _type: PhantomData<T>,
}

unsafe impl<T: ConstructType> AbstractType for DenseMatrix<T> {}

unsafe impl<T: ConstructType> ConstructType for DenseMatrix<T> {
    fn construct_type<'target, Tgt>(
        target: ExtendedTarget<'target, '_, '_, Tgt>,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        let (target, frame) = target.split();

        frame
            .scope(|mut frame| {
                let ty_param = T::construct_type(frame.as_extended_target());
                let params = [ty_param];
                unsafe {
                    let applied = Module::base(&frame)
                        .global(&frame, "DenseMatrix")
                        .unwrap()
                        .as_value()
                        .apply_type_unchecked(&mut frame, params);
                    Ok(UnionAll::rewrap(
                        target.into_extended_target(&mut frame),
                        applied.cast_unchecked::<DataType>(),
                    ))
                }
            })
            .unwrap()
    }
}

/// Construct a new `DenseVector` type object from the provided type parameters.
pub struct DenseVector<T: ConstructType> {
    _type: PhantomData<T>,
}

unsafe impl<T: ConstructType> AbstractType for DenseVector<T> {}

unsafe impl<T: ConstructType> ConstructType for DenseVector<T> {
    fn construct_type<'target, Tgt>(
        target: ExtendedTarget<'target, '_, '_, Tgt>,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        let (target, frame) = target.split();

        frame
            .scope(|mut frame| {
                let ty_param = T::construct_type(frame.as_extended_target());
                let params = [ty_param];
                unsafe {
                    let applied = Module::base(&frame)
                        .global(&frame, "DenseVector")
                        .unwrap()
                        .as_value()
                        .apply_type_unchecked(&mut frame, params);
                    Ok(UnionAll::rewrap(
                        target.into_extended_target(&mut frame),
                        applied.cast_unchecked::<DataType>(),
                    ))
                }
            })
            .unwrap()
    }
}

/// Construct a new `Enum` type object from the provided type parameters.
pub struct Enum<T: ConstructType> {
    _type: PhantomData<T>,
}

unsafe impl<T: ConstructType> AbstractType for Enum<T> {}

unsafe impl<T: ConstructType> ConstructType for Enum<T> {
    fn construct_type<'target, Tgt>(
        target: ExtendedTarget<'target, '_, '_, Tgt>,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        let (target, frame) = target.split();

        frame
            .scope(|mut frame| {
                let ty_param = T::construct_type(frame.as_extended_target());

                // Validate bound
                if let Ok(tvar) = ty_param.cast::<TypeVar>() {
                    unsafe {
                        let ub = tvar.upper_bound(&frame).as_value();
                        assert!(ub.subtype(Integer::construct_type(frame.as_extended_target())));
                    }
                } else {
                    assert!(ty_param.subtype(Integer::construct_type(frame.as_extended_target())));
                }

                let params = [ty_param];

                unsafe {
                    let applied = Module::base(&frame)
                        .global(&frame, "Enum")
                        .unwrap()
                        .as_value()
                        .apply_type_unchecked(&mut frame, params);
                    Ok(UnionAll::rewrap(
                        target.into_extended_target(&mut frame),
                        applied.cast_unchecked::<DataType>(),
                    ))
                }
            })
            .unwrap()
    }
}

/// Construct a new `OrdinalRange` type object from the provided type parameters.
pub struct OrdinalRange<T: ConstructType, S: ConstructType> {
    _type: PhantomData<T>,
    _s: PhantomData<S>,
}

unsafe impl<T: ConstructType, S: ConstructType> AbstractType for OrdinalRange<T, S> {}

unsafe impl<T: ConstructType, S: ConstructType> ConstructType for OrdinalRange<T, S> {
    fn construct_type<'target, Tgt>(
        target: ExtendedTarget<'target, '_, '_, Tgt>,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        let (target, frame) = target.split();

        frame
            .scope(|mut frame| {
                let ty_param = T::construct_type(frame.as_extended_target());
                let n_param = S::construct_type(frame.as_extended_target());
                let params = [ty_param, n_param];
                unsafe {
                    let applied = Module::base(&frame)
                        .global(&frame, "OrdinalRange")
                        .unwrap()
                        .as_value()
                        .apply_type_unchecked(&mut frame, params);
                    Ok(UnionAll::rewrap(
                        target.into_extended_target(&mut frame),
                        applied.cast_unchecked::<DataType>(),
                    ))
                }
            })
            .unwrap()
    }
}

/*
function print_vars(ty)
    while ty isa UnionAll
        println("  ", ty.var)
        ty = ty.body
    end
end

function list_abstract_types()
    for name in names(Core)
        item = getglobal(Core, name)
        if isabstracttype(item)
            println("Core.", name)
            if item isa UnionAll
                print_vars(item)
            end
        end
    end

    for name in names(Base)
        item = getglobal(Base, name)
        if isabstracttype(item)
            println("Base.", name)
            if item isa UnionAll
                print_vars(item)
            end
        end
    end
end


Abstract DataTypes:

///Core.AbstractChar
///Core.AbstractFloat
///Core.AbstractString
///Core.Any
///Core.Exception
///Core.Function
///Core.IO
///Core.Integer
///Core.Number
///Core.Real
///Core.Signed
///Core.Unsigned

///Base.AbstractDisplay
///Base.AbstractIrrational
///Base.AbstractMatch
///Base.AbstractPattern
///Base.IndexStyle
///
///
///Abstract UnionAlls:
///
///Core.AbstractArray{T,N}
///Core.DenseArray{T,N}
///Core.Ref{T}
///Core.Type{T}
///
///Base.AbstractChannel{T}
///Base.AbstractDict{K, V}
///Base.AbstractMatrix{T}
///Base.AbstractRange{T}
///Base.AbstractSet{T}
///Base.AbstractSlices{T,N}
///Base.AbstractUnitRange{T}
///Base.AbstractVector{T}
///Base.DenseMatrix{T}
///Base.DenseVector{T}
///Base.Enum{T<:Integer}
///Base.OrdinalRange{T,S}
*/
