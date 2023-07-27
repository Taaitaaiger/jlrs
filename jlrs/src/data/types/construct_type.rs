//! Construct Julia type objects from Rust types.

use std::{any::TypeId, ffi::c_void, marker::PhantomData, ptr::NonNull};

use fnv::FnvHashMap;
use jl_sys::{
    jl_array_typename, jl_bool_type, jl_bottom_type, jl_char_type, jl_float32_type,
    jl_float64_type, jl_int16_type, jl_int32_type, jl_int64_type, jl_int8_type, jl_pointer_type,
    jl_uint16_type, jl_uint32_type, jl_uint64_type, jl_uint8_type, jl_uniontype_type, jl_value_t,
    jl_voidpointer_type,
};

use super::abstract_types::AnyType;
use crate::{
    convert::to_symbol::ToSymbol,
    data::managed::{
        datatype::DataType,
        type_var::TypeVar,
        union::Union,
        union_all::UnionAll,
        value::{Value, ValueData},
        Managed,
    },
    gc_safe::{GcSafeOnceLock, GcSafeRwLock},
    memory::target::Target,
    prelude::Tuple,
    private::Private,
};

static CONSTRUCTED_TYPE_CACHE: GcSafeOnceLock<ConstructedTypes> = GcSafeOnceLock::new();

pub(crate) unsafe fn init_constructed_type_cache() {
    CONSTRUCTED_TYPE_CACHE.set(ConstructedTypes::new()).ok();
}

/// Associate a Julia type object with a Rust type.
///
/// Safety:
///
/// `ConstructType::construct_type` must either return a valid type object, or an instance of an
/// isbits type which is immediately used as a type parameter of another constructed type.
pub unsafe trait ConstructType: Sized {
    /// `Self`, but with all lifetimes set to `'static`.
    type Static: 'static + ConstructType;

    /// Indicates whether the type might be cacheable.
    ///
    /// If set to `false`, `construct_type` will never try to cache or look up the
    /// constructed type. It should be set to `false` if the constructed type isn't a `DataType`.
    const CACHEABLE: bool = true;

    /// Returns the `TypeId` of this type.
    #[inline]
    fn type_id() -> TypeId {
        TypeId::of::<Self::Static>()
    }

    /// Construct the type object and try to cache the result. If a cached entry is available, it
    /// is returned.
    #[inline]
    fn construct_type<'target, T>(target: T) -> ValueData<'target, 'static, T>
    where
        T: Target<'target>,
    {
        if Self::CACHEABLE {
            unsafe {
                CONSTRUCTED_TYPE_CACHE
                    .get_unchecked()
                    .find_or_construct::<Self, _>(target)
            }
        } else {
            Self::construct_type_uncached(target)
        }
    }

    /// Constructs the type object associated with this type.
    fn construct_type_uncached<'target, T>(target: T) -> ValueData<'target, 'static, T>
    where
        T: Target<'target>;

    /// Returns the base type object associated with this type.
    ///
    /// The base type object is the type object without any types applied to it. If there is no
    /// such type object, e.g. when `Self` is a value type, `None` is returned. The base type must
    /// assumed to be globally rooted.
    fn base_type<'target, T>(target: &T) -> Option<Value<'target, 'static>>
    where
        T: Target<'target>;
}

macro_rules! impl_construct_julia_type_constant {
    ($ty:ty, $const_ty:ty) => {
        unsafe impl<const N: $const_ty> ConstructType for $ty {
            type Static = $ty;

            const CACHEABLE: bool = false;

            #[inline]
            fn construct_type_uncached<'target, T>(target: T) -> ValueData<'target, 'static, T>
            where
                T: Target<'target>,
            {
                Value::new(target, N)
            }

            #[inline]
            fn base_type<'target, T>(_target: &T) -> Option<Value<'target, 'static>>
            where
                T: Target<'target>,
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
            fn construct_type_uncached<'target, T>(target: T) -> ValueData<'target, 'static, T>
            where
                T: Target<'target>,
            {
                unsafe {
                    let ptr = NonNull::new_unchecked($jl_ty.cast::<jl_value_t>());
                    target.data_from_ptr(ptr, Private)
                }
            }

            #[inline]
            fn base_type<'target, T>(_target: &T) -> Option<Value<'target, 'static>>
            where
                T: Target<'target>,
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
impl_construct_julia_type_constant!(ConstantI8<N>, i8);

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
    fn construct_type_uncached<'target, T>(target: T) -> ValueData<'target, 'static, T>
    where
        T: Target<'target>,
    {
        Value::new(target, N as isize)
    }

    #[inline]
    fn base_type<'target, T>(_target: &T) -> Option<Value<'target, 'static>>
    where
        T: Target<'target>,
    {
        None
    }
}

// TODO: UInt8 and/or Int8 objects are statically allocated in Julia and can be cached.
/// Constant `u8`.
pub struct ConstantU8<const N: u8>;
impl_construct_julia_type_constant!(ConstantU8<N>, u8);

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

/// Constant string. Not a type constructor, but can be used to implement multi-character type
/// vars.
pub trait ConstantStr: 'static {
    /// The string constant.
    const STR: &'static str;
}

/// The name of a `TypeVar`.
pub struct Name<const N: char>;

/// Trait to set the name of`TypeVar`.
///
/// Implemented by [`Name`], [`ConstantChar`], and implementations of [`ConstantStr`].
pub trait TypeVarName: 'static {
    /// The type returned by `name`.
    type Sym: ToSymbol;

    // Returns the name.
    fn name() -> Self::Sym;
}

impl<const N: char> TypeVarName for Name<N> {
    type Sym = String;

    #[inline]
    fn name() -> Self::Sym {
        String::from(N)
    }
}

impl<const N: char> TypeVarName for ConstantChar<N> {
    type Sym = String;

    #[inline]
    fn name() -> Self::Sym {
        String::from(N)
    }
}

impl<T: ConstantStr> TypeVarName for T {
    type Sym = &'static str;

    #[inline]
    fn name() -> Self::Sym {
        Self::STR
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

unsafe impl<N: TypeVarName, U: ConstructType, L: ConstructType> ConstructType
    for TypeVarConstructor<N, U, L>
{
    type Static = TypeVarConstructor<N, U::Static, L::Static>;

    fn construct_type_uncached<'target, T>(target: T) -> ValueData<'target, 'static, T>
    where
        T: Target<'target>,
    {
        target
            .with_local_scope::<_, _, 2>(|target, mut frame| {
                let upper_bound = U::construct_type(&mut frame);
                let lower_bound = L::construct_type(&mut frame);
                unsafe {
                    Ok(TypeVar::new_unchecked(
                        &target,
                        N::name(),
                        Some(lower_bound),
                        Some(upper_bound),
                    )
                    .as_value()
                    .root(target))
                }
            })
            .unwrap()
    }

    #[inline]
    fn base_type<'target, T>(_target: &T) -> Option<Value<'target, 'static>>
    where
        T: Target<'target>,
    {
        None
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
            target
                .with_local_scope::<_, _, 3>(|target, mut frame| {
                    let ty_param = T::construct_type(&mut frame);
                    let rank_param = N::construct_type(&mut frame);
                    let params = [ty_param, rank_param];
                    let applied = Self::base_type(&frame)
                        .unwrap_unchecked()
                        .apply_type_unchecked(&mut frame, params);

                    Ok(UnionAll::rewrap(
                        target,
                        applied.cast_unchecked::<DataType>(),
                    ))
                })
                .unwrap_unchecked()
        }
    }

    #[inline]
    fn base_type<'target, Tgt>(_target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        unsafe {
            let wrapper =
                NonNull::new_unchecked(NonNull::new_unchecked(jl_array_typename).as_ref().wrapper);
            // let ptr = NonNull::new_unchecked(jl_array_type.cast::<jl_value_t>());
            Some(
                <Value as crate::data::managed::private::ManagedPriv>::wrap_non_null(
                    wrapper,
                    crate::private::Private,
                ),
            )
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
        target
            .with_local_scope::<_, _, 2>(|target, mut frame| {
                let l = L::construct_type(&mut frame);
                let r = R::construct_type(&mut frame);

                unsafe { Ok(Union::new_unchecked(target, [l, r])) }
            })
            .unwrap()
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
}

pub struct BottomType;

unsafe impl ConstructType for BottomType {
    type Static = BottomType;

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
        target
            .with_local_scope::<_, _, 1>(|target, mut frame| {
                let ty = U::construct_type(&mut frame);
                unsafe {
                    Ok(UnionAll::pointer_type(&frame)
                        .as_value()
                        .apply_type_unchecked(target, [ty]))
                }
            })
            .unwrap()
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
    fn find_or_construct<'target, T: ConstructType, Tgt: Target<'target>>(
        &self,
        target: Tgt,
    ) -> ValueData<'target, 'static, Tgt> {
        let tid = T::type_id();

        {
            if let Some(res) = self.data.read().get(&tid).copied() {
                return res.root(target);
            }
        }

        do_construct::<T, _>(target, self, tid)
    }
}

#[inline(never)]
fn do_construct<'target, T: ConstructType, Tgt: Target<'target>>(
    target: Tgt,
    ct: &ConstructedTypes,
    tid: TypeId,
) -> ValueData<'target, 'static, Tgt> {
    unsafe {
        target
            .with_local_scope::<_, _, 1>(|target, mut frame| {
                let ty = T::construct_type_uncached(&mut frame);

                if ty.is::<DataType>() {
                    let dt = ty.cast_unchecked::<DataType>();
                    if !dt.has_free_type_vars() && (!dt.is::<Tuple>() || dt.is_concrete_type()) {
                        ct.data.write().insert(tid, ty.leak().as_value());
                    }
                }

                Ok(ty.root(target))
            })
            .unwrap_unchecked()
    }
}

unsafe impl Sync for ConstructedTypes {}
unsafe impl Send for ConstructedTypes {}
