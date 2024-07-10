//! Construct Julia type objects from Rust types.

use std::{any::TypeId, ffi::c_void, marker::PhantomData, ptr::NonNull, string::FromUtf8Error};

use fnv::FnvHashMap;
use jl_sys::{
    jl_bool_type, jl_bottom_type, jl_char_type, jl_float32_type, jl_float64_type, jl_int16_type,
    jl_int32_type, jl_int64_type, jl_int8_type, jl_pointer_type, jl_uint16_type, jl_uint32_type,
    jl_uint64_type, jl_uint8_type, jl_uniontype_type, jl_value_t, jl_voidpointer_type,
};
use rustc_hash::FxHashSet;

use super::abstract_type::{AbstractType, AnyType};
use crate::{
    convert::to_symbol::ToSymbol,
    data::{
        layout::{is_bits::IsBits, typed_layout::HasLayout},
        managed::{
            array::dimensions::DimsExt,
            datatype::DataType,
            simple_vector::SimpleVector,
            type_var::{TypeVar, TypeVarData},
            union::Union,
            union_all::UnionAll,
            value::{Value, ValueData},
            Managed,
        },
    },
    gc_safe::{GcSafeOnceLock, GcSafeRwLock},
    memory::{
        scope::LocalScope,
        target::{unrooted::Unrooted, RootingTarget, Target},
    },
    prelude::{ConstructTypedArray, Symbol, Tuple},
    private::Private,
};

static CONSTRUCTED_TYPE_CACHE: GcSafeOnceLock<ConstructedTypes> = GcSafeOnceLock::new();

#[cfg_attr(
    not(any(
        feature = "local-rt",
        feature = "async-rt",
        feature = "multi-rt",
        feature = "ccall"
    )),
    allow(unused)
)]
pub(crate) unsafe fn init_constructed_type_cache() {
    CONSTRUCTED_TYPE_CACHE.set(ConstructedTypes::new()).ok();
}

/// Define a fast key type for a constructible type.
///
/// See [`FastKey`] for an example.
#[macro_export]
macro_rules! define_fast_key {
    ($(#[$meta:meta])* $vis:vis $ty:ident, $for_ty:ty) => {
        $(#[$meta])*
        $vis struct $ty;

        unsafe impl $crate::data::types::construct_type::FastKey for $ty {
            type For = $for_ty;

            #[inline]
            fn construct_type_fast<'target, Tgt>(
                target: &Tgt,
            ) -> $crate::data::managed::value::Value<'target, 'static>
            where
                Tgt: $crate::memory::target::Target<'target>,
            {
                static REF: $crate::data::static_data::StaticConstructibleType<$for_ty> =
                    $crate::data::static_data::StaticConstructibleType::<$for_ty>::new();
                REF.get_or_init(&target)
            }
        }
    };
}

/// Define a fast array key type for a constructible array type.
///
/// See [`FastArrayKey`] for an example.
#[macro_export]
macro_rules! define_fast_array_key {
    ($(#[$meta:meta])* $vis:vis $ty:ident, $elem_ty:ty, $rank:literal) => {
        $(#[$meta])*
        $vis struct $ty;

        unsafe impl $crate::data::types::construct_type::FastKey for $ty {
            type For = $crate::data::managed::array::TypedRankedArray<
                'static,
                'static,
                <$elem_ty as $crate::data::types::construct_type::ConstructType>::Static,
                $rank
            >;

            #[inline]
            fn construct_type_fast<'target, Tgt>(
                target: &Tgt,
            ) -> $crate::data::managed::value::Value<'target, 'static>
            where
                Tgt: $crate::memory::target::Target<'target>,
            {
                type Ty = $crate::data::types::construct_type::RankedArrayType<$elem_ty, $rank>;
                static REF: $crate::data::static_data::StaticConstructibleType<Ty> =
                    $crate::data::static_data::StaticConstructibleType::<Ty>::new();
                REF.get_or_init(&target)
            }
        }

        unsafe impl $crate::data::types::construct_type::FastArrayKey<$rank> for $ty {
            type ElemType = $elem_ty;
        }
    };
}

/// Shorthand macro for [`TypeVarConstructor`].
///
/// In Julia, `TypeVar`s have a name, an upper bound, and a lower bound. In most cases, you will
/// only care about the name and maybe the upper bound. The `TypeVarConstructor` type is quite
/// verbose, this macro provides a useful shorthand.
///
/// This macro expands as follows:
///
/// `tvar!('S') -> TypeVarConstructor<Name<'S'>>`
/// `tvar!(Name<'S'>) -> TypeVarConstructor<Name<'S'>>`
/// `tvar!('S'; AnyType) -> TypeVarConstructor<Name<'S'>, AnyType>`
/// `tvar!(Name<'S'>; AnyType) -> TypeVarConstructor<Name<'S'>, AnyType>`
/// `tvar!(BottomType; 'S'; AnyType) -> TypeVarConstructor<Name<'S'>, AnyType, BottomType>`
/// `tvar!(BottomType; Name<'S'>; AnyType) -> TypeVarConstructor<Name<'S'>, AnyType, BottomType>`
///
/// As you can see, it's similar to Julia's `lb <: S <: ub` syntax for `TypeVar`s, except that
/// `<:` has been replaced with `;` (because Rust macros don't allow `<:` in this position).
#[macro_export]
macro_rules! tvar {
    ($name:ty) => {
        $crate::data::types::construct_type::TypeVarConstructor::<$name>
    };
    ($name:literal) => {
        $crate::data::types::construct_type::TypeVarConstructor::<
            $crate::data::types::construct_type::Name<$name>,
        >
    };
    ($name:literal; $ub:ty) => {
        $crate::data::types::construct_type::TypeVarConstructor::<
            $crate::data::types::construct_type::Name<$name>,
            $ub,
        >
    };
    ($name:ty; $ub:ty) => {
        $crate::data::types::construct_type::TypeVarConstructor::<$name, $ub>
    };
    ($lb:ty; $name:literal; $ub:ty) => {
        $crate::data::types::construct_type::TypeVarConstructor::<
            $crate::data::types::construct_type::Name<$name>,
            $ub,
            $lb,
        >
    };
    ($lb:ty; $name:ty; $ub:ty) => {
        $crate::data::types::construct_type::TypeVarConstructor::<$name, $ub, $lb>
    };
}

/// Combine multiple [`TypeVarConstructor`]s into a single type. The resulting type implements
/// [`TypeVars`].
///
/// This macro has a very niche application: it can be used with the [`julia_module`] macro to
/// expose a function with a signature that has a where-clause, for example
///
/// `function foo(a::A) where {T, N, A <: AbstractArray{T, N}} end`.
///
/// This macro is used to generate the `{T, N, A <: AbstractArray{T, N}}` PART.
///
/// [`julia_module`]: crate::prelude::julia_module
#[macro_export]
macro_rules! tvars {
    ($t:ty) => {
        $t
    };
    ($t1:ty, $R:ty) => {
        $crate::data::types::construct_type::TypeVarFragment<$t1, $R>
    };
    ($t1:ty, $R:ty, $($rest:ty),+) => {
        $crate::data::types::construct_type::TypeVarFragment<$t1, tvars!($R, $($rest),+)>
    };
}

/// Encode bytes into `ConstantBytes`.
///
/// See [`ConstantBytes`] for more information.
#[macro_export]
macro_rules! bytes {
    ($t1:literal, $R:literal) => {
        $crate::data::types::construct_type::ConstantBytes<
            $crate::data::types::construct_type::ConstantU8<$t1>,
            $crate::data::types::construct_type::ConstantU8<$R>
        >
    };
    ($t1:literal, $R:literal, $($rest:literal),+) => {
        $crate::data::types::construct_type::ConstantBytes<
            $crate::data::types::construct_type::ConstantU8<$t1>,
            $crate::bytes!($R, $($rest),+)
        >
    };
}

/// Associate a Julia type object with a Rust type.
///
/// Safety:
///
/// `ConstructType::construct_type` must either return a valid type object, or an instance of an
/// isbits type which is immediately used as a type parameter of another constructed type.
#[cfg_attr(
    feature = "diagnostics",
    diagnostic::on_unimplemented(
        message = "the trait bound `{Self}: ConstructType` is not satisfied",
        label = "the trait `ConstructType` is not implemented for `{Self}`",
        note = "Custom types that implement `ConstructType` should be generated with JlrsCore.reflect",
        note = "Do not implement `ForeignType`, `OpaqueType`, or `ParametricVariant` unless this type is exported to Julia with `julia_module!`"
    )
)]

pub unsafe trait ConstructType: Sized {
    /// `Self`, but with all lifetimes set to `'static`. This ensures `Self::Static` has a type
    /// id.
    type Static: 'static + ConstructType;

    /// Indicates whether the type might be cacheable.
    ///
    /// If set to `false`, `construct_type` will never try to cache or look up the
    /// constructed type. It should be set to `false` if the constructed type is not globally
    /// rooted.
    const CACHEABLE: bool = true;

    /// Returns the `TypeId` of `Self::Static`.
    #[inline]
    fn type_id() -> TypeId {
        TypeId::of::<Self::Static>()
    }

    /// Construct the type object and try to cache the result. If a cached entry is available, it
    /// is returned.
    #[inline]
    fn construct_type<'target, Tgt>(target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        if Self::CACHEABLE {
            unsafe {
                let unrooted = Unrooted::new();
                CONSTRUCTED_TYPE_CACHE
                    .get_unchecked()
                    .find_or_construct::<Self>(unrooted)
                    .root(target)
            }
        } else {
            Self::construct_type_uncached(target)
        }
    }

    /// Constructs the type object associated with this type.
    fn construct_type_uncached<'target, Tgt>(target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>;

    /// Construct the type object with an environment of `TypeVar`s and try to cache the result.
    /// If a cached entry is available, it is returned.
    ///
    /// No new type vars are constructed, if one is used and it don't already exist in `env`,
    /// this method panics. The result may have free `TypeVar`s, you can call
    /// [`DataType::wrap_with_env`] to create the appropriate `UnionAll`.
    #[inline]
    fn construct_type_with_env<'target, Tgt>(
        target: Tgt,
        env: &TypeVarEnv,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        if Self::CACHEABLE {
            unsafe {
                let unrooted = Unrooted::new();
                CONSTRUCTED_TYPE_CACHE
                    .get_unchecked()
                    .find_or_construct_with_env::<Self>(unrooted, env)
                    .root(target)
            }
        } else {
            Self::construct_type_with_env_uncached(target, env)
        }
    }

    /// Constructs the type object associated with this type.
    ///
    /// No new type vars are constructed, if one is used and it don't already exist in `env`,
    /// this method panics.
    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        env: &TypeVarEnv,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>;

    /// Returns the base type object associated with this type.
    ///
    /// The base type object is the type object without any types applied to it. If there is no
    /// such type object, e.g. when `Self` is a value type, `None` is returned. The base type must
    /// be globally rooted.
    fn base_type<'target, Tgt>(target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>;
}

/// Returns the type constructed with the `ConstructType` implementation of `T1` if it is a
/// concrete type, otherwise that of `T2`.
pub struct IfConcreteElse<T1: ConstructType, T2: ConstructType> {
    _marker: PhantomData<(T1, T2)>,
}

unsafe impl<T1: ConstructType, T2: ConstructType> ConstructType for IfConcreteElse<T1, T2> {
    type Static = IfConcreteElse<T1::Static, T2::Static>;

    fn construct_type_uncached<'target, Tgt>(target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        let t1 = T1::construct_type(&target);
        unsafe {
            let v = t1.as_value();
            if v.is::<DataType>() {
                if v.cast_unchecked::<DataType>().is_concrete_type() {
                    return t1.root(target);
                }
            }

            T2::construct_type(target)
        }
    }

    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        env: &crate::data::types::construct_type::TypeVarEnv,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        let t1 = T1::construct_type_with_env_uncached(&target, env);
        unsafe {
            let v = t1.as_value();
            if v.is::<DataType>() {
                if v.cast_unchecked::<DataType>().is_concrete_type() {
                    return t1.root(target);
                }
            }

            T2::construct_type_with_env_uncached(target, env)
        }
    }

    fn base_type<'target, Tgt>(target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        let t1 = T1::construct_type(&target);
        unsafe {
            let v = t1.as_value();
            if v.is::<DataType>() {
                if v.cast_unchecked::<DataType>().is_concrete_type() {
                    return T1::base_type(target);
                }
            }

            T2::base_type(target)
        }
    }
}

/// Cache a single constructible type.
///
/// By default, constructible types with type parameters are significantly slower to access than
/// types without type parameters. The reason is that types without type parameters can be cached
/// in a local static variable, while types with type parameters are stored in a global hash map.
///
/// It's not possible to generally cache types with type parameters in a local static variable,
/// but specific types can be cached by implementing this trait for a new type.
///
/// Safety: the constructed type must not have any free type parameters, [`define_fast_key`]
/// should be used to create the type and implementation.
///
/// Example:
///
/// ```
/// # use jlrs::prelude::*;
/// use jlrs::data::types::{abstract_type::AbstractSet, construct_type::{ConstructType, Key}};
///
/// define_fast_key!(pub AbstractSetF64, AbstractSet<f64>);
///
/// # fn main() {
/// # let mut julia = Builder::new().start_local().unwrap();
///
/// julia.local_scope::<_, 2>(|mut frame| {
///     let ty = Key::<AbstractSetF64>::construct_type(&mut frame);
///     let ty2 = AbstractSet::<f64>::construct_type(&mut frame);
///     assert_eq!(ty, ty2);
/// });
/// # }
/// ```
pub unsafe trait FastKey: 'static {
    type For: ConstructType;
    fn construct_type_fast<'target, Tgt>(target: &Tgt) -> Value<'target, 'static>
    where
        Tgt: Target<'target>;
}

/// Cache a single constructible array type.
///
/// Similar to [`FastKey`] but specifically intended to be used with array types. The implementation
/// must not be generic over `N`, but must be set to a specific, non-negative value.
///
/// All implementations of `FastArrayKey` implement [`ConstructTypedArray`].
///
/// Safety: the constructed type must not have any free type parameters and must be an array type,
/// [`define_fast_array_key!`] should be used to create the type and implementation.
///
/// Example:
///
/// ```
/// # use jlrs::prelude::*;
/// use jlrs::data::types::construct_type::{ConstructType, Key};
///
/// define_fast_array_key!(pub VecF32, f32, 1);
///
/// # fn main() {
/// # let mut julia = Builder::new().start_local().unwrap();
///
/// julia.local_scope::<_, 3>(|mut frame| {
///     let ty = Key::<VecF32>::construct_type(&mut frame);
///     let ty2 = TypedRankedArray::<f32, 1>::construct_type(&mut frame);
///     assert_eq!(ty, ty2);
///
///     let v = VecF32::new(&mut frame, 4);
///     assert!(v.is_ok());
/// });
/// # }
/// ```
pub unsafe trait FastArrayKey<const N: isize>: 'static + FastKey {
    /// The element type of this array type.
    type ElemType: ConstructType;

    /// Assert that `Self::RANK` is non-negative.
    const ASSERT_VALID_RANK: () = assert!(N >= 0, "Array rank must be non-negative");
}

impl<T: FastArrayKey<N>, const N: isize> ConstructTypedArray<T::ElemType, N> for T {
    #[inline]
    fn array_type<'target, D, Tgt>(target: Tgt, _dims: &D) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
        D: DimsExt,
    {
        let _ = Self::ASSERT_VALID_RANK;
        Key::<T>::construct_type(target)
    }
}

/// Type used to expose [`ConstructType`] for types that implement [`FastKey`].
///
/// See [`FastKey`] and [`FastArrayKey`] for examples.
pub struct Key<K: FastKey>(PhantomData<K>);

unsafe impl<K: FastKey> ConstructType for Key<K> {
    type Static = <K::For as ConstructType>::Static;
    const CACHEABLE: bool = false;

    fn construct_type_uncached<'target, Tgt>(target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        K::construct_type_fast(&target).root(target)
    }

    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        _env: &TypeVarEnv,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        K::construct_type_fast(&target).root(target)
    }

    fn base_type<'target, Tgt>(target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        unsafe {
            let v = K::construct_type_fast(target);
            if v.is::<DataType>() {
                let dt = v.cast_unchecked::<DataType>();
                Some(dt.type_name().wrapper())
            } else {
                Some(v)
            }
        }
    }
}

unsafe impl<K> AbstractType for Key<K>
where
    K: FastKey,
    K::For: AbstractType,
{
}

/// One or more `TypeVar`s. Types that implement this trait should be generated with the [`tvars`]
/// and [`tvar`] macros.
pub trait TypeVars {
    /// The number of `TypeVar`s encoded by `Self`.
    const SIZE: usize;

    /// Construct the `TypeVars` and convert them to a context that can be used with
    /// [`ConstructType::construct_type_with_env`].
    fn into_env<'target, Tgt: RootingTarget<'target>>(target: Tgt) -> TypeVarEnv<'target>;

    #[doc(hidden)]
    // internal trait method used by `into_context`.
    fn extend_env<'target, Tgt: Target<'target>>(target: &Tgt, env: &mut TypeVarEnv, offset: usize);
}

impl<N: TypeVarName, U: ConstructType, L: ConstructType> TypeVars for TypeVarConstructor<N, U, L> {
    const SIZE: usize = 1;

    fn into_env<'target, Tgt: RootingTarget<'target>>(target: Tgt) -> TypeVarEnv<'target> {
        target.with_local_scope::<_, _, 1>(|target, mut frame| {
            let svec = SimpleVector::with_capacity(&mut frame, Self::SIZE);
            let mut env = TypeVarEnv { svec };

            Self::extend_env(&frame, &mut env, 0);

            let svec = Tgt::into_concrete_type(svec.root(target));
            TypeVarEnv { svec }
        })
    }

    fn extend_env<'target, Tgt: Target<'target>>(
        target: &Tgt,
        env: &mut TypeVarEnv,
        offset: usize,
    ) {
        target.local_scope::<_, 1>(|mut frame| {
            let sym = N::symbol(&frame);
            if let Some(_) = env.get(sym) {
                panic!("Duplicate tvar");
            }

            let tvar = Self::new(&mut frame, env);
            env.set(offset, tvar);
        })
    }
}

/// Type that combines two or more `TypeVarConstructor`s.
///
/// Rust doesn't have variadic generics, which prevents us from writing `TypeVars<TV1, TV2, ...>`,
/// instead this type lets us build it recursively:
/// `TypeVarFragment<TV1, TypeVarFragment<TV2, ...>>`. It's neither necessary nor recommended to
/// write out this type manually, you should use the [`tvars`] macro instead:
/// `tvars!(TV1, TV2, ...)`.
pub struct TypeVarFragment<T1: TypeVars, R: TypeVars>(PhantomData<T1>, PhantomData<R>);

impl<T1: TypeVars, R: TypeVars> TypeVars for TypeVarFragment<T1, R> {
    const SIZE: usize = T1::SIZE + R::SIZE;

    fn into_env<'target, Tgt: RootingTarget<'target>>(target: Tgt) -> TypeVarEnv<'target> {
        target.with_local_scope::<_, _, 1>(|target, mut frame| {
            let svec = SimpleVector::with_capacity(&mut frame, Self::SIZE);
            let mut env = TypeVarEnv { svec };

            T1::extend_env(&frame, &mut env, 0);
            R::extend_env(&frame, &mut env, T1::SIZE);

            let svec = Tgt::into_concrete_type(svec.root(target));
            TypeVarEnv { svec }
        })
    }

    fn extend_env<'target, Tgt: Target<'target>>(
        target: &Tgt,
        env: &mut TypeVarEnv,
        offset: usize,
    ) {
        T1::extend_env(target, env, offset);
        R::extend_env(target, env, offset + T1::SIZE);
    }
}

/// An environment of [`TypeVar`]s, i.e. all `TypeVar`s that appear in a function signature.
#[derive(Debug)]
pub struct TypeVarEnv<'scope> {
    svec: SimpleVector<'scope>,
}

impl<'scope> TypeVarEnv<'scope> {
    /// Returns the `TypeVar` with name `sym` if it exists.
    pub fn get(&self, sym: Symbol) -> Option<TypeVar<'scope>> {
        let unrooted = sym.unrooted_target();
        let svec = self.svec.data();

        (0..svec.len())
            .filter_map(|idx| svec.get(unrooted, idx))
            .map(|elem| unsafe { elem.as_value().cast_unchecked::<TypeVar>() })
            .find(|elem| elem.name() == sym)
            .map(|elem| unsafe { elem.as_ref().leak().as_managed() })
    }

    /// Returns `true` if the environment is empty.
    pub fn is_empty(&self) -> bool {
        self.svec.len() == 0
    }

    /// Create an empty environment.
    pub fn empty<Tgt: Target<'scope>>(tgt: &Tgt) -> Self {
        TypeVarEnv {
            svec: SimpleVector::emptysvec(tgt),
        }
    }

    /// Access this environment as a `SimpleVector`.
    pub fn to_svec(&self) -> SimpleVector<'scope> {
        self.svec
    }

    fn set(&mut self, offset: usize, tvar: TypeVar) {
        unsafe {
            let len = self.svec.len();
            assert!(offset < len);
            let data = self.svec.data();
            data.set(offset, Some(tvar.as_value())).unwrap();
        }
    }
}

macro_rules! impl_construct_julia_type_constant {
    ($ty:ty, $const_ty:ty) => {
        unsafe impl<const N: $const_ty> ConstructType for $ty {
            type Static = $ty;

            const CACHEABLE: bool = false;

            #[inline]
            fn construct_type_uncached<'target, Tgt>(
                target: Tgt,
            ) -> ValueData<'target, 'static, Tgt>
            where
                Tgt: Target<'target>,
            {
                Value::new(target, N)
            }

            #[inline]
            fn construct_type_with_env_uncached<'target, Tgt>(
                target: Tgt,
                _: &TypeVarEnv,
            ) -> ValueData<'target, 'static, Tgt>
            where
                Tgt: Target<'target>,
            {
                Value::new(target, N)
            }

            #[inline]
            fn base_type<'target, Tgt>(_target: &Tgt) -> Option<Value<'target, 'static>>
            where
                Tgt: Target<'target>,
            {
                None
            }
        }
    };
}

macro_rules! impl_construct_julia_type_constant_cached {
    ($ty:ty, $const_ty:ty) => {
        unsafe impl<const N: $const_ty> ConstructType for $ty {
            type Static = $ty;

            const CACHEABLE: bool = true;

            #[inline]
            fn construct_type_uncached<'target, Tgt>(
                target: Tgt,
            ) -> ValueData<'target, 'static, Tgt>
            where
                Tgt: Target<'target>,
            {
                Value::new(target, N)
            }

            #[inline]
            fn construct_type_with_env_uncached<'target, Tgt>(
                target: Tgt,
                _: &TypeVarEnv,
            ) -> ValueData<'target, 'static, Tgt>
            where
                Tgt: Target<'target>,
            {
                Value::new(target, N)
            }

            #[inline]
            fn base_type<'target, Tgt>(_target: &Tgt) -> Option<Value<'target, 'static>>
            where
                Tgt: Target<'target>,
            {
                None
            }
        }
    };
}

macro_rules! impl_construct_julia_type_primitive {
    ($ty:ty, $jl_ty:ident) => {
        unsafe impl ConstructType for $ty {
            type Static = $ty;

            const CACHEABLE: bool = false;

            #[inline]
            fn construct_type_uncached<'target, Tgt>(
                target: Tgt,
            ) -> ValueData<'target, 'static, Tgt>
            where
                Tgt: Target<'target>,
            {
                unsafe {
                    let ptr = NonNull::new_unchecked($jl_ty.cast::<jl_value_t>());
                    target.data_from_ptr(ptr, Private)
                }
            }

            #[inline]
            fn construct_type_with_env_uncached<'target, Tgt>(
                target: Tgt,
                _env: &TypeVarEnv,
            ) -> ValueData<'target, 'static, Tgt>
            where
                Tgt: Target<'target>,
            {
                unsafe {
                    let ptr = NonNull::new_unchecked($jl_ty.cast::<jl_value_t>());
                    target.data_from_ptr(ptr, Private)
                }
            }

            #[inline]
            fn base_type<'target, Tgt>(_target: &Tgt) -> Option<Value<'target, 'static>>
            where
                Tgt: Target<'target>,
            {
                unsafe {
                    let ptr = NonNull::new_unchecked($jl_ty.cast::<jl_value_t>());
                    Some(
                        <Value as $crate::data::managed::private::ManagedPriv>::wrap_non_null(
                            ptr,
                            $crate::private::Private,
                        ),
                    )
                }
            }
        }
    };
}

/// Constant `i8`.
pub struct ConstantI8<const N: i8>;
impl_construct_julia_type_constant_cached!(ConstantI8<N>, i8);

/// Constant `i16`.
pub struct ConstantI16<const N: i16>;
impl_construct_julia_type_constant!(ConstantI16<N>, i16);

/// Constant `i32`.
pub struct ConstantI32<const N: i32>;
impl_construct_julia_type_constant!(ConstantI32<N>, i32);

/// Constant `i64`.
pub struct ConstantI64<const N: i64>;
impl_construct_julia_type_constant!(ConstantI64<N>, i64);

/// Constant `isize`.
pub struct ConstantIsize<const N: isize>;
impl_construct_julia_type_constant!(ConstantIsize<N>, isize);

/// Constant `isize`.
pub struct ConstantSize<const N: usize>;
unsafe impl<const N: usize> ConstructType for ConstantSize<N> {
    type Static = ConstantSize<N>;

    const CACHEABLE: bool = false;

    #[inline]
    fn construct_type_uncached<'target, Tgt>(target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        Value::new(target, N as isize)
    }

    #[inline]
    fn base_type<'target, Tgt>(_target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        None
    }

    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        _env: &TypeVarEnv,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        Value::new(target, N as isize)
    }
}

/// Constant `u8`.
pub struct ConstantU8<const N: u8>;
impl_construct_julia_type_constant_cached!(ConstantU8<N>, u8);

/// Constant `u16`.
pub struct ConstantU16<const N: u16>;
impl_construct_julia_type_constant!(ConstantU16<N>, u16);

/// Constant `u32`.
pub struct ConstantU32<const N: u32>;
impl_construct_julia_type_constant!(ConstantU32<N>, u32);

/// Constant `u64`.
pub struct ConstantU64<const N: u64>;
impl_construct_julia_type_constant!(ConstantU64<N>, u64);

/// Constant `usize`.
pub struct ConstantUsize<const N: usize>;
impl_construct_julia_type_constant!(ConstantUsize<N>, usize);

/// Constant `bool`.
pub struct ConstantBool<const N: bool>;
impl_construct_julia_type_constant!(ConstantBool<N>, bool);

/// Constant `char`.
pub struct ConstantChar<const N: char>;
impl_construct_julia_type_constant!(ConstantChar<N>, char);

/// Trait implemented by types that encode bytes.
pub trait EncodedBytes: 'static {
    type B: AsRef<[u8]>;
    fn encoded_bytes() -> Self::B;
}

/// Trait implemented by types that encode a string.
pub trait EncodedString: 'static {
    type S: AsRef<str>;
    fn encoded_string() -> Self::S;
}

/// Constant string. Not a type constructor, but can be used to construct `TypeVar`s with names
/// longer than one character.
pub trait ConstantStr: 'static {
    /// The string constant.
    const STR: &'static str;
}

impl<S: ConstantStr> EncodedString for S {
    type S = &'static str;
    fn encoded_string() -> Self::S {
        Self::STR
    }
}

/// Constant byte slice.
pub trait ConstantByteSlice: 'static {
    /// The byte slice constant.
    const BYTES: &'static [u8];
}

impl<S: ConstantByteSlice> EncodedBytes for S {
    type B = &'static [u8];
    fn encoded_bytes() -> Self::B {
        Self::BYTES
    }
}

/// Trait implemented by `ConstantU8` and `ConstantBytes` to build a list of constant bytes.
pub trait ConstantBytesFragment: 'static {
    /// The size of this fragment.
    const SIZE: usize;

    #[doc(hidden)]
    fn extend(slice: &mut [u8], offset: usize);
}

/// Constant bytes.
///
/// `ConstantBytes` implements `TypeVarName`. In general you should prefer to implement
/// [`ConstantStr`] over using `ConstantBytes`.
///
/// While it isn't possible to use static string slice as a const generic parameter, it is
/// possible to recursively encode its bytes into a type. For example, the string "Foo" can be
/// represented as follows:
///
/// `type Foo = ConstantBytes<ConstantU8<70>, ConstantBytes<ConstantU8<111>, ConstantU8<111>>>`.
///
/// The [`bytes`] macro is less verbose, but only accepts `u8`'s:
///
/// `type Foo = bytes!(70, 111, 111)`
///
/// The [`encode_as_constant_bytes`] macro converts a string literal to `ConstantBytes`:
///
/// `type Foo = encode_as_constant_bytes!("Foo")`.
///
/// The main advantage of `encode_as_constant_bytes` is that it doesn't represent the string as a
/// list like the `bytes` macro does, but represents it as a tree with the minimal depth:
///
/// `type Foo = ConstantBytes<ConstantBytes<ConstantU8<70>, ConstantU8<111>>, ConstantU8<111>>`
///
/// [`encode_as_constant_bytes`]: jlrs_macros::encode_as_constant_bytes
pub struct ConstantBytes<L: ConstantBytesFragment, R: ConstantBytesFragment>(
    PhantomData<L>,
    PhantomData<R>,
);

impl<L: ConstantBytesFragment, R: ConstantBytesFragment> ConstantBytes<L, R> {
    /// Convert the encoded bytes to `Vec<u8>`.
    pub fn into_vec() -> Vec<u8> {
        let mut v = vec![0; Self::SIZE];
        Self::extend(v.as_mut_slice(), 0);
        v
    }

    /// Try to convert the encoded bytes into a string.
    pub fn into_string() -> Result<String, FromUtf8Error> {
        let v = Self::into_vec();
        String::from_utf8(v)
    }
}

impl<L: ConstantBytesFragment, R: ConstantBytesFragment> EncodedBytes for ConstantBytes<L, R> {
    type B = Vec<u8>;
    fn encoded_bytes() -> Vec<u8> {
        Self::into_vec()
    }
}

impl<L: ConstantBytesFragment, R: ConstantBytesFragment> EncodedString for ConstantBytes<L, R> {
    type S = String;
    fn encoded_string() -> String {
        Self::into_string().expect("Invalid string")
    }
}

impl<L: ConstantBytesFragment, R: ConstantBytesFragment> TypeVarName for ConstantBytes<L, R> {
    fn symbol<'target, Tgt: Target<'target>>(target: &Tgt) -> Symbol<'target> {
        Self::into_string()
            .expect("Invalid string")
            .to_symbol(target)
    }
}

impl<const N: u8> ConstantBytesFragment for ConstantU8<N> {
    const SIZE: usize = 1;

    #[inline]
    fn extend(slice: &mut [u8], offset: usize) {
        slice[offset] = N;
    }
}

impl<L: ConstantBytesFragment, R: ConstantBytesFragment> ConstantBytesFragment
    for ConstantBytes<L, R>
{
    const SIZE: usize = L::SIZE + R::SIZE;

    #[inline]
    fn extend(slice: &mut [u8], offset: usize) {
        L::extend(slice, offset);
        R::extend(slice, offset + L::SIZE);
    }
}

/// The name of a `TypeVar`, alternative for [`ConstantChar`].
pub struct Name<const N: char>;

/// Trait to set the name of a `TypeVar`.
///
/// Implemented by [`Name`], [`ConstantChar`], and implementations of [`ConstantStr`].
pub trait TypeVarName: 'static {
    /// Returns the name as a symbol.
    fn symbol<'target, Tgt: Target<'target>>(target: &Tgt) -> Symbol<'target>;
}

impl<const N: char> TypeVarName for Name<N> {
    #[inline]
    fn symbol<'target, Tgt: Target<'target>>(target: &Tgt) -> Symbol<'target> {
        let mut bytes = [0; 4];
        let s = N.encode_utf8(&mut bytes);
        s.to_symbol(target)
    }
}

impl<const N: char> TypeVarName for ConstantChar<N> {
    #[inline]
    fn symbol<'target, Tgt: Target<'target>>(target: &Tgt) -> Symbol<'target> {
        let mut bytes = [0; 4];
        let s = N.encode_utf8(&mut bytes);
        s.to_symbol(target)
    }
}

impl<T: ConstantStr> TypeVarName for T {
    #[inline]
    fn symbol<'target, Tgt: Target<'target>>(target: &Tgt) -> Symbol<'target> {
        Self::STR.to_symbol(target)
    }
}

/// Construct a new `TypeVar` from the provided type parameters.
pub struct TypeVarConstructor<
    N: TypeVarName,
    U: ConstructType = AnyType,
    L: ConstructType = BottomType,
> {
    _name: PhantomData<N>,
    _upper: PhantomData<U>,
    _lower: PhantomData<L>,
}

impl<N: TypeVarName, U: ConstructType, L: ConstructType> TypeVarConstructor<N, U, L> {
    fn new<'target, Tgt>(target: Tgt, env: &TypeVarEnv) -> TypeVarData<'target, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, _, 2>(|target, mut frame| {
            let upper_bound = U::construct_type_with_env(&mut frame, env);
            let lower_bound = L::construct_type_with_env(&mut frame, env);
            unsafe {
                TypeVar::new_unchecked(
                    &target,
                    N::symbol(&target),
                    Some(lower_bound),
                    Some(upper_bound),
                )
                .root(target)
            }
        })
    }
}

unsafe impl<N: TypeVarName, U: ConstructType, L: ConstructType> ConstructType
    for TypeVarConstructor<N, U, L>
{
    type Static = TypeVarConstructor<N, U::Static, L::Static>;

    fn construct_type_uncached<'target, Tgt>(target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, _, 2>(|target, mut frame| {
            let upper_bound = U::construct_type(&mut frame);
            let lower_bound = L::construct_type(&mut frame);
            unsafe {
                TypeVar::new_unchecked(
                    &target,
                    N::symbol(&target),
                    Some(lower_bound),
                    Some(upper_bound),
                )
                .as_value()
                .root(target)
            }
        })
    }

    #[inline]
    fn base_type<'target, Tgt>(target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        Some(DataType::tvar_type(target).as_value())
    }

    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        env: &TypeVarEnv,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        let sym = N::symbol(&target);
        env.get(sym).unwrap().as_value().root(target)
    }
}

/// Construct a new `Array` type from the provided type parameters.
pub struct ArrayTypeConstructor<T: ConstructType, N: ConstructType> {
    _type: PhantomData<T>,
    _rank: PhantomData<N>,
}

unsafe impl<T: ConstructType, N: ConstructType> ConstructType for ArrayTypeConstructor<T, N> {
    type Static = ArrayTypeConstructor<T::Static, N::Static>;

    fn construct_type_uncached<'target, Tgt>(target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        unsafe {
            target.with_local_scope::<_, _, 3>(|target, mut frame| {
                let ty_param = T::construct_type(&mut frame);
                let rank_param = N::construct_type(&mut frame);
                if rank_param.is::<isize>() {
                    if rank_param.unbox_unchecked::<isize>() < 0 {
                        panic!("ArrayTypeConstructor rank must be a TypeVar or non-negative ConstantIsize, got {rank_param:?}")
                    }
                }
                let params = [ty_param, rank_param];
                Self::base_type(&frame)
                    .unwrap_unchecked()
                    .apply_type_unchecked(&mut frame, params)
                    .cast_unchecked::<DataType>()
                    .rewrap(target)
            })
        }
    }

    #[inline]
    fn base_type<'target, Tgt>(target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        let wrapper = UnionAll::array_type(target).as_value();
        Some(wrapper)
    }

    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        env: &TypeVarEnv,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        unsafe {
            target.with_local_scope::<_, _, 3>(|target, mut frame| {
                let ty_param = T::construct_type_with_env(&mut frame, env);
                let rank_param = N::construct_type_with_env(&mut frame, env);
                if rank_param.is::<isize>() {
                    if rank_param.unbox_unchecked::<isize>() < 0 {
                        panic!("ArrayTypeConstructor rank must be a TypeVar or non-negative ConstantIsize, got {rank_param:?}")
                    }
                }
                let params = [ty_param, rank_param];
                Self::base_type(&frame)
                    .unwrap_unchecked()
                    .apply_type_unchecked(&mut frame, params)
                    .cast_unchecked::<DataType>()
                    .wrap_with_env(target, env)
            })
        }
    }
}

pub type RankedArrayType<T, const N: isize> = ArrayTypeConstructor<T, ConstantIsize<N>>;

/// Construct a new `Union` type from the provided type parameters. Larger unions can be built
/// by nesting `UnionTypeConstructor`.
pub struct UnionTypeConstructor<L: ConstructType, R: ConstructType> {
    _l: PhantomData<L>,
    _r: PhantomData<R>,
}

unsafe impl<L: ConstructType, R: ConstructType> ConstructType for UnionTypeConstructor<L, R> {
    type Static = UnionTypeConstructor<L::Static, R::Static>;

    fn construct_type_uncached<'target, Tgt>(target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, _, 2>(|target, mut frame| {
            let l = L::construct_type(&mut frame);
            let r = R::construct_type(&mut frame);

            unsafe { Union::new_unchecked(target, [l, r]) }
        })
    }

    #[inline]
    fn base_type<'target, Tgt>(_target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        unsafe {
            let ptr = NonNull::new_unchecked(jl_uniontype_type.cast::<jl_value_t>());
            Some(
                <Value as crate::data::managed::private::ManagedPriv>::wrap_non_null(
                    ptr,
                    crate::private::Private,
                ),
            )
        }
    }

    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        env: &TypeVarEnv,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, _, 2>(|target, mut frame| {
            let l = L::construct_type_with_env(&mut frame, env);
            let r = R::construct_type_with_env(&mut frame, env);

            unsafe { Union::new_unchecked(target, [l, r]) }
        })
    }
}

/// Trait implemented by `UnionTypeConstructor` if all variants are bits types.
pub trait BitsUnionCtor: BitsUnionCtorVariant {
    /// Returns the number of unique variants.
    fn n_unique_variants() -> usize {
        Self::get_variants().len()
    }

    /// Returns the set of type ids of all unique variants, and the number of variants.
    fn get_variants() -> FxHashSet<TypeId>;
}

/// Trait implemented by type constructors that have an `IsBits` layout, and unions of such types.
pub trait BitsUnionCtorVariant: ConstructType {
    const N: usize;
    #[doc(hidden)]
    fn add_variants(ids: &mut FxHashSet<TypeId>);
}

impl<'scope, 'data, T> BitsUnionCtorVariant for T
where
    T: ConstructType + HasLayout<'scope, 'data>,
    T::Layout: IsBits,
{
    const N: usize = 1;
    fn add_variants(ids: &mut FxHashSet<TypeId>) {
        ids.insert(Self::type_id());
    }
}

impl<L: BitsUnionCtorVariant, R: BitsUnionCtorVariant> BitsUnionCtorVariant
    for UnionTypeConstructor<L, R>
{
    const N: usize = L::N + R::N;
    fn add_variants(ids: &mut FxHashSet<TypeId>) {
        L::add_variants(ids);
        R::add_variants(ids);
    }
}

impl<L: BitsUnionCtorVariant, R: BitsUnionCtorVariant> BitsUnionCtor
    for UnionTypeConstructor<L, R>
{
    fn get_variants() -> FxHashSet<TypeId> {
        let mut set = FxHashSet::<TypeId>::default();

        L::add_variants(&mut set);
        R::add_variants(&mut set);

        set
    }
}

pub struct BottomType;

unsafe impl ConstructType for BottomType {
    type Static = BottomType;

    #[inline]
    fn construct_type_uncached<'target, Tgt>(target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        unsafe {
            let ptr = NonNull::new_unchecked(jl_bottom_type.cast::<jl_value_t>());
            target.data_from_ptr(ptr, Private)
        }
    }

    #[inline]
    fn base_type<'target, Tgt>(_target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        unsafe {
            let ptr = NonNull::new_unchecked(jl_bottom_type.cast::<jl_value_t>());
            Some(
                <Value as crate::data::managed::private::ManagedPriv>::wrap_non_null(
                    ptr,
                    crate::private::Private,
                ),
            )
        }
    }

    #[inline]
    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        _env: &TypeVarEnv,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        unsafe {
            let ptr = NonNull::new_unchecked(jl_bottom_type.cast::<jl_value_t>());
            target.data_from_ptr(ptr, Private)
        }
    }
}

impl_construct_julia_type_primitive!(u8, jl_uint8_type);
impl_construct_julia_type_primitive!(u16, jl_uint16_type);
impl_construct_julia_type_primitive!(u32, jl_uint32_type);
impl_construct_julia_type_primitive!(u64, jl_uint64_type);

#[cfg(target_pointer_width = "64")]
impl_construct_julia_type_primitive!(usize, jl_uint64_type);
#[cfg(target_pointer_width = "32")]
impl_construct_julia_type_primitive!(usize, jl_uint32_type);

impl_construct_julia_type_primitive!(i8, jl_int8_type);
impl_construct_julia_type_primitive!(i16, jl_int16_type);
impl_construct_julia_type_primitive!(i32, jl_int32_type);
impl_construct_julia_type_primitive!(i64, jl_int64_type);

#[cfg(target_pointer_width = "64")]
impl_construct_julia_type_primitive!(isize, jl_int64_type);
#[cfg(target_pointer_width = "32")]
impl_construct_julia_type_primitive!(isize, jl_int32_type);

impl_construct_julia_type_primitive!(f32, jl_float32_type);
impl_construct_julia_type_primitive!(f64, jl_float64_type);

impl_construct_julia_type_primitive!(bool, jl_bool_type);
impl_construct_julia_type_primitive!(char, jl_char_type);

impl_construct_julia_type_primitive!(*mut c_void, jl_voidpointer_type);

unsafe impl<U: ConstructType> ConstructType for *mut U {
    type Static = *mut U::Static;

    fn construct_type_uncached<'target, Tgt>(target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, _, 1>(|target, mut frame| {
            let ty = U::construct_type(&mut frame);
            unsafe {
                UnionAll::pointer_type(&frame)
                    .as_value()
                    .apply_type_unchecked(target, [ty])
            }
        })
    }

    #[inline]
    fn base_type<'target, Tgt>(_target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        unsafe {
            let ptr = NonNull::new_unchecked(jl_pointer_type.cast::<jl_value_t>());
            Some(
                <Value as crate::data::managed::private::ManagedPriv>::wrap_non_null(
                    ptr,
                    crate::private::Private,
                ),
            )
        }
    }

    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        env: &TypeVarEnv,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, _, 1>(|target, mut frame| {
            let ty = U::construct_type_with_env(&mut frame, env);
            unsafe {
                UnionAll::pointer_type(&frame)
                    .as_value()
                    .apply_type_unchecked(target, [ty])
            }
        })
    }
}

struct ConstructedTypes {
    data: GcSafeRwLock<FnvHashMap<TypeId, Value<'static, 'static>>>,
}

impl ConstructedTypes {
    fn new() -> Self {
        ConstructedTypes {
            data: GcSafeRwLock::new(FnvHashMap::default()),
        }
    }

    #[inline]
    fn find_or_construct<'target, T: ConstructType>(
        &self,
        target: Unrooted<'target>,
    ) -> ValueData<'target, 'static, Unrooted<'target>> {
        let tid = T::type_id();

        {
            if let Some(res) = self.data.read().get(&tid).copied() {
                return res.root(target);
            }
        }

        do_construct::<T>(target, self, tid)
    }

    #[inline]
    fn find_or_construct_with_env<'target, T: ConstructType>(
        &self,
        target: Unrooted<'target>,
        env: &TypeVarEnv,
    ) -> ValueData<'target, 'static, Unrooted<'target>> {
        let tid = T::type_id();

        {
            if let Some(res) = self.data.read().get(&tid).copied() {
                return res.root(target);
            }
        }

        do_construct_with_context::<T>(target, self, tid, env)
    }
}

// #[inline(never)]
#[cold]
fn do_construct<'target, T: ConstructType>(
    target: Unrooted<'target>,
    ct: &ConstructedTypes,
    tid: TypeId,
) -> ValueData<'target, 'static, Unrooted<'target>> {
    unsafe {
        target.with_local_scope::<_, _, 1>(|target, mut frame| {
            let ty = T::construct_type_uncached(&mut frame);

            if ty.is::<DataType>() {
                let dt = ty.cast_unchecked::<DataType>();
                if !dt.has_free_type_vars() && (!dt.is::<Tuple>() || dt.is_concrete_type()) {
                    ct.data.write().insert(tid, ty.leak().as_value());
                }
            } else if ty.is::<u8>() || ty.is::<i8>() {
                ct.data.write().insert(tid, ty.leak().as_value());
            }

            ty.root(target)
        })
    }
}

// #[inline(never)]
#[cold]
fn do_construct_with_context<'target, T: ConstructType>(
    target: Unrooted<'target>,
    ct: &ConstructedTypes,
    tid: TypeId,
    env: &TypeVarEnv,
) -> ValueData<'target, 'static, Unrooted<'target>> {
    unsafe {
        target.with_local_scope::<_, _, 1>(|target, mut frame| {
            let ty = T::construct_type_with_env_uncached(&mut frame, env);

            if ty.is::<DataType>() {
                let dt = ty.cast_unchecked::<DataType>();
                if !dt.has_free_type_vars() && (!dt.is::<Tuple>() || dt.is_concrete_type()) {
                    ct.data.write().insert(tid, ty.leak().as_value());
                }
            } else if ty.is::<u8>() || ty.is::<i8>() {
                ct.data.write().insert(tid, ty.leak().as_value());
            }

            ty.root(target)
        })
    }
}

unsafe impl Sync for ConstructedTypes {}
unsafe impl Send for ConstructedTypes {}
