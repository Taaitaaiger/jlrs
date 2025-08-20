//! Named tuples to provide keyword arguments to functions.
//!
//! This module provides a custom `NamedTuple` type to aid in guaranteeing that a `NamedTuple` is
//! provided when necessary. A `NamedTuple` should be created with the [`named_tuple`] macro,
//! several functions and methods are provided to create named tuples in more specialized cases,
//! and create new instances that add and/or remove pairs.
//! 
//! [`named_tuple`]: crate::named_tuple

use std::{fmt, marker::PhantomData, ptr::NonNull};

use fnv::FnvHashMap;
use jl_sys::{jl_field_index, jl_get_nth_field, jl_value_t};
use smallvec::SmallVec;

use crate::{
    catch::{catch_exceptions, unwrap_exc},
    convert::to_symbol::ToSymbol,
    data::{
        layout::tuple::{NTuple, Tuple},
        managed::{private::ManagedPriv, type_name::TypeName, union_all::UnionAll, Weak},
        types::{construct_type::ConstructType, typecheck::Typecheck},
    },
    memory::{
        scope::LocalScopeExt,
        target::{unrooted::Unrooted, TargetResult, TargetType},
    },
    prelude::{DataType, Managed, Symbol, Target, Value, ValueData},
    private::Private,
};

/// Create a new named tuple.
///
/// The syntax is `named_tuple!(target, k1 => v1, k1 => v2, ...)`; the keys must implement
/// `ToSymbol`, the values must be `Value`s. An error is returned if the named tuple contains any
/// duplicate keys.
///
/// Example:
///
/// ```
/// # use jlrs::prelude::*;
/// # fn main() {
/// # let mut julia = Builder::new().start_local().unwrap();
/// julia
///     .local_scope::<JlrsResult<_>, 5>(|mut frame| unsafe {
///         // The code we evaluate is a simple function definition, which is safe.
///         let func = unsafe {
///             Value::eval_string(&mut frame, "func(; a=3, b=4, c=5) = a + b + c")? // 1
///         };
///
///         let a = Value::new(&mut frame, 1isize); // 2
///         let b = Value::new(&mut frame, 2isize); // 3
///         let nt = named_tuple!(&mut frame, "a" => a, "b" => b)?; // 4
///
///         // Call the previously defined function. This function simply sums its three
///         // keyword arguments and has no side effects, so it's safe to call.
///         let res = unsafe {
///             func.provide_keywords(nt)
///                 .call(&mut frame, [])? // 5
///                 .unbox::<isize>()?
///         };
///
///         assert_eq!(res, 8);
///         Ok(())
///     }).unwrap();
/// # }
#[macro_export]
macro_rules! named_tuple {
    ($frame:expr, $name:expr => $value:expr) => {
        {
            let name = $crate::convert::to_symbol::ToSymbol::to_symbol(&$name, &$frame);
            $crate::data::managed::named_tuple::NamedTuple::from_n_pairs($frame, &[(name, $value)])
        }
    };
    ($frame:expr, $name:expr => $value:expr, $($rest:tt)+) => {
        {
            const N: usize = $crate::count!($($rest)+);
            let mut pairs: [::std::mem::MaybeUninit::<($crate::data::managed::symbol::Symbol, $crate::data::managed::value::Value)>; N] = [::std::mem::MaybeUninit::uninit(); N];
            let name = $crate::convert::to_symbol::ToSymbol::to_symbol(&$name, &$frame);

            pairs[0].write((name, $value));
            $crate::named_tuple!($frame, 1, &mut pairs, $($rest)+)
        }
    };
    ($frame:expr, $i:expr, $pairs:expr, $name:expr => $value:expr, $($rest:tt)+) => {
        {
            let name = $crate::convert::to_symbol::ToSymbol::to_symbol(&$name, &$frame);
            $pairs[$i].write((name, $value));
            $crate::named_tuple!($frame, $i + 1, $pairs, $($rest)+)
        }
    };
    ($frame:expr, $i:expr, $pairs:expr, $name:expr => $value:expr) => {
        {
            let name = $crate::convert::to_symbol::ToSymbol::to_symbol(&$name, &$frame);
            $pairs[$i].write((name, $value));

            let pairs: &[($crate::data::managed::symbol::Symbol, $crate::data::managed::value::Value); N] = unsafe {
                ::std::mem::transmute::<
                    &[::std::mem::MaybeUninit::<($crate::data::managed::symbol::Symbol, $crate::data::managed::value::Value)>; N],
                    &[($crate::data::managed::symbol::Symbol, $crate::data::managed::value::Value); N]
                >($pairs)
            };

            $crate::data::managed::named_tuple::NamedTuple::from_n_pairs($frame, pairs)
        }
    };
}

/// A `NamedTuple`.
///
/// Named tuples are mainly used as keyword arguments for Julia functions.
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct NamedTuple<'scope, 'data>(
    NonNull<jl_value_t>,
    PhantomData<&'scope ()>,
    PhantomData<&'data mut ()>,
);

impl<'scope, 'data> NamedTuple<'scope, 'data> {
    /// Create a new named tuple from `N` pairs of keywords and values.
    ///
    /// If an exception is thrown, e.g. due to duplicate keys, it is caught and returned.
    pub fn from_n_pairs<'target, Tgt, const N: usize>(
        target: Tgt,
        items: &[(Symbol<'_>, Value<'_, 'data>); N],
    ) -> NamedTupleResult<'target, 'data, Tgt>
    where
        Tgt: Target<'target>,
    {
        unsafe {
            let nt =
                match catch_exceptions(|| Self::from_n_pairs_unchecked(&target, items), unwrap_exc)
                {
                    Ok(nt) => Ok(nt.ptr()),
                    Err(e) => Err(e),
                };

            target.result_from_ptr(nt, Private)
        }
    }

    /// Create a new named tuple from `N` pairs of keywords and values without catching exceptions.
    ///
    /// Safety: if an exception is thrown, e.g. due to duplicate keys, it is not caught.
    pub unsafe fn from_n_pairs_unchecked<'target, Tgt, const N: usize>(
        target: Tgt,
        items: &[(Symbol<'_>, Value<'_, 'data>); N],
    ) -> NamedTupleData<'target, 'data, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, 3>(|target, mut frame| {
            let field_names = items.map(|(sym, _)| sym.as_value());
            let values = items.map(|(_, val)| val);
            let field_types = values.map(|val| val.datatype().as_value());

            unsafe {
                let names_tup = NTuple::<Symbol, N>::construct_type(&frame)
                    .as_value()
                    .cast_unchecked::<DataType>()
                    .instantiate_unchecked(&mut frame, &field_names);

                let field_types_tup = DataType::anytuple_type(&frame)
                    .as_value()
                    .apply_type_unchecked(&mut frame, &field_types);

                UnionAll::namedtuple_type(&frame)
                    .as_value()
                    .apply_type_unchecked(&mut frame, &[names_tup, field_types_tup])
                    .cast_unchecked::<DataType>()
                    .instantiate_unchecked(&frame, values)
                    .as_value()
                    .cast_unchecked::<NamedTuple>()
                    .root(target)
            }
        })
    }

    /// Create a new named tuple from an iterator of keywords and values.
    ///
    /// If an exception is thrown, e.g. due to duplicate keys, it is caught and returned.
    pub fn from_iter<'target, 'd, Tgt>(
        target: Tgt,
        items: impl ExactSizeIterator<Item = (Symbol<'d>, Value<'d, 'data>)>,
    ) -> NamedTupleResult<'target, 'data, Tgt>
    where
        Tgt: Target<'target>,
    {
        unsafe {
            let nt =
                match catch_exceptions(|| Self::from_iter_unchecked(&target, items), unwrap_exc) {
                    Ok(nt) => Ok(nt.ptr()),
                    Err(e) => Err(e),
                };

            target.result_from_ptr(nt, Private)
        }
    }

    /// Create a new named tuple from an iterator of keywords and values without catching
    /// exceptions.
    ///
    /// Safety: if an exception is thrown, e.g. due to duplicate keys, it is not caught.
    pub unsafe fn from_iter_unchecked<'target, 'd, Tgt>(
        target: Tgt,
        items: impl ExactSizeIterator<Item = (Symbol<'d>, Value<'d, 'data>)>,
    ) -> NamedTupleData<'target, 'data, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, 4>(|target, mut frame| {
            let n = items.len();
            let mut keys = SmallVec::<[_; 8]>::new();
            let mut values = SmallVec::<[_; 8]>::new();

            items.fold((&mut keys, &mut values), |(keys, values), (key, value)| {
                keys.push(key.as_value());
                values.push(value);
                (keys, values)
            });

            let field_types = values
                .iter()
                .copied()
                .map(|val| val.datatype().as_value())
                .collect::<SmallVec<[_; 8]>>();

            unsafe {
                let mut syms = SmallVec::<[_; 8]>::with_capacity(n);
                let st = DataType::symbol_type(&frame).as_value();
                for _ in 0..n {
                    syms.push(st);
                }

                let names_tup = DataType::anytuple_type(&frame)
                    .as_value()
                    .apply_type_unchecked(&mut frame, syms)
                    .cast_unchecked::<DataType>()
                    .instantiate_unchecked(&mut frame, &keys);

                let field_types_tup = DataType::anytuple_type(&frame)
                    .as_value()
                    .apply_type_unchecked(&mut frame, &field_types);

                UnionAll::namedtuple_type(&frame)
                    .as_value()
                    .apply_type_unchecked(&mut frame, &[names_tup, field_types_tup])
                    .cast_unchecked::<DataType>()
                    .instantiate_unchecked(&frame, values)
                    .as_value()
                    .cast_unchecked::<NamedTuple>()
                    .root(target)
            }
        })
    }

    /// Create a new named tuple from `keys` and `values`.
    ///
    /// There must be as many keys as values or this function will panic.
    ///
    /// If an exception is thrown, e.g. due to duplicate keys, it is caught and returned.
    pub fn new<'target, Tgt>(
        target: Tgt,
        keys: &[Symbol],
        values: &[Value<'_, 'data>],
    ) -> NamedTupleResult<'target, 'data, Tgt>
    where
        Tgt: Target<'target>,
    {
        unsafe {
            let nt =
                match catch_exceptions(|| Self::new_unchecked(&target, keys, values), unwrap_exc) {
                    Ok(nt) => Ok(nt.ptr()),
                    Err(e) => Err(e),
                };

            target.result_from_ptr(nt, Private)
        }
    }

    /// Create a new named tuple from `keys` and `values` without catching exceptions.
    ///
    /// There must be as many keys as values or this function will panic.
    ///
    /// Safety: if an exception is thrown, e.g. due to duplicate keys, it is not caught.
    pub unsafe fn new_unchecked<'target, Tgt>(
        target: Tgt,
        keys: &[Symbol],
        values: &[Value<'_, 'data>],
    ) -> NamedTupleData<'target, 'data, Tgt>
    where
        Tgt: Target<'target>,
    {
        let n = keys.len();
        assert_eq!(n, values.len(), "mismatched number of keys and values");

        target.with_local_scope::<_, 4>(|target, mut frame| {
            let field_types = values
                .iter()
                .copied()
                .map(|val| val.datatype().as_value())
                .collect::<Vec<_>>();

            unsafe {
                let mut syms = SmallVec::<[_; 8]>::with_capacity(n);
                let st = DataType::symbol_type(&frame).as_value();
                for _ in 0..n {
                    syms.push(st);
                }

                let keys_v = std::slice::from_raw_parts(keys.as_ptr().cast(), n);

                let names_tup = DataType::anytuple_type(&frame)
                    .as_value()
                    .apply_type_unchecked(&mut frame, syms)
                    .cast_unchecked::<DataType>()
                    .instantiate_unchecked(&mut frame, keys_v);

                let field_types_tup = DataType::anytuple_type(&frame)
                    .as_value()
                    .apply_type_unchecked(&mut frame, &field_types);

                UnionAll::namedtuple_type(&frame)
                    .as_value()
                    .apply_type_unchecked(&mut frame, &[names_tup, field_types_tup])
                    .cast_unchecked::<DataType>()
                    .instantiate_unchecked(&frame, values)
                    .as_value()
                    .cast_unchecked::<NamedTuple>()
                    .root(target)
            }
        })
    }

    /// Returns the field names of this named tuple.
    ///
    /// The field names of a type are normally accessed through its [`DataType`], because
    /// they are encoded differently for `NamedTuple`s this custom method must be used instead.
    pub fn field_names(self) -> &'scope [Symbol<'scope>] {
        let dt = self.as_value().datatype();

        let names_param = dt.parameter(0);
        if names_param.is_none() {
            return &[];
        }

        let names_param = names_param.unwrap();
        if !names_param.is::<Tuple>() {
            return &[];
        }

        let sz = names_param.datatype().size().unwrap() as usize / std::mem::size_of::<Symbol>();
        let names = names_param.unwrap(Private).cast::<Symbol>();
        unsafe { std::slice::from_raw_parts(names, sz) }
    }

    /// Returns `true` if this named tuple contains `keyword`.
    pub fn contains<K: ToSymbol>(self, keyword: K) -> bool {
        let dt = self.as_value().datatype();

        unsafe {
            let sym = keyword.to_symbol_priv(Private);
            jl_field_index(dt.unwrap(Private), sym.unwrap(Private), 0) >= 0
        }
    }

    /// Returns the value associated with `keyword` if it is present in this named tuple.
    pub fn get<'target, K, Tgt>(
        self,
        target: Tgt,
        keyword: K,
    ) -> Option<ValueData<'target, 'data, Tgt>>
    where
        K: ToSymbol,
        Tgt: Target<'target>,
    {
        let dt = self.as_value().datatype();

        unsafe {
            let sym = keyword.to_symbol_priv(Private);
            let idx = jl_field_index(dt.unwrap(Private), sym.unwrap(Private), 0);
            if idx < 0 {
                return None;
            }

            let val = jl_get_nth_field(self.as_value().unwrap(Private), idx as usize);
            Some(Value::wrap_non_null(NonNull::new(val)?, Private).root(target))
        }
    }

    /// Creates a new named tuple from `self` where `key` has been removed.
    pub fn remove<'target, Tgt>(
        self,
        target: Tgt,
        key: Symbol,
    ) -> NamedTupleData<'target, 'data, Tgt>
    where
        Tgt: Target<'target>,
    {
        self.filter(target, &[key])
    }

    /// Creates a new named tuple from `self` where `key` is set to `value`.
    pub fn set<'target, Tgt>(
        self,
        target: Tgt,
        key: Symbol<'_>,
        value: Value<'_, 'data>,
    ) -> NamedTupleData<'target, 'data, Tgt>
    where
        Tgt: Target<'target>,
    {
        self.extend_iter(target, [(key, value)].into_iter())
    }

    /// Creates a new named tuple from `self`, excluding any keyword present in `remove`.
    pub fn filter<'target, Tgt>(
        self,
        target: Tgt,
        remove: &[Symbol],
    ) -> NamedTupleData<'target, 'data, Tgt>
    where
        Tgt: Target<'target>,
    {
        let n_roots = self.as_value().n_fields();
        let fnames = self.field_names();
        target.with_unsized_local_scope(n_roots, |target, mut frame| {
            let mut keys = Vec::with_capacity(n_roots - remove.len());
            let mut values = Vec::with_capacity(n_roots - remove.len());

            for key in fnames.iter().copied() {
                if !remove.contains(&key) {
                    let value = self.as_value().get_field(&mut frame, key).unwrap();

                    keys.push(key);
                    values.push(value);
                }
            }

            // Safety: there cannot be any duplicates
            unsafe { Self::new_unchecked(target, keys.as_ref(), values.as_ref()) }
        })
    }

    /// Creates a new named tuple from `self`, and adds additional pairs from `keys` and `values`.
    /// New values override the old ones.
    ///
    /// This method panics if the number of elements in `keys` and `values` don't match.
    pub fn extend<'target, Tgt>(
        self,
        target: Tgt,
        keys: &[Symbol],
        values: &[Value<'_, 'data>],
    ) -> NamedTupleData<'target, 'data, Tgt>
    where
        Tgt: Target<'target>,
    {
        let n = keys.len();
        assert_eq!(n, values.len(), "mismatched number of keys and values");

        let nt = self.as_value();
        let n_fields = nt.n_fields();

        let field_names = self.field_names();

        target.with_unsized_local_scope(n_fields, |target, mut frame| {
            let mut map = FnvHashMap::default();

            unsafe {
                for i in 0..n_fields {
                    let key = field_names[i];
                    let value = nt.get_nth_field(&mut frame, i).unwrap();
                    map.insert(key, value);
                }

                for (key, value) in keys.iter().copied().zip(values.iter().copied()) {
                    map.insert(key, value);
                }

                // There cannot be any duplicates
                Self::from_iter_unchecked(target, map.iter().map(|(a, b)| (*a, *b)))
            }
        })
    }

    /// Create a new named tuple from this one and an iterator of key-value pairs. New values
    /// override the old ones.
    pub fn extend_iter<'target, 'd, Tgt>(
        self,
        target: Tgt,
        items: impl Iterator<Item = (Symbol<'d>, Value<'d, 'data>)>,
    ) -> NamedTupleData<'target, 'data, Tgt>
    where
        Tgt: Target<'target>,
    {
        let nt = self.as_value();
        let n_fields = nt.n_fields();

        let field_names = self.field_names();

        target.with_unsized_local_scope(n_fields, |target, mut frame| {
            let mut map = FnvHashMap::default();

            unsafe {
                for i in 0..n_fields {
                    let key = field_names[i];
                    let value = nt.get_nth_field(&mut frame, i).unwrap();
                    map.insert(key, value);
                }

                for (key, value) in items {
                    map.insert(key, value);
                }

                // There cannot be any duplicates
                Self::from_iter_unchecked(target, map.iter().map(|(a, b)| (*a, *b)))
            }
        })
    }

    /// Combines [`Self::filter`] and [`Self::extend`]. All duplicates are removed, later elements
    /// win.
    ///
    /// This method panics if the number of elements in `keys` and `values` don't match.
    pub fn filter_extend<'target, Tgt>(
        self,
        target: Tgt,
        remove: &[Symbol],
        keys: &[Symbol],
        values: &[Value<'_, 'data>],
    ) -> NamedTupleData<'target, 'data, Tgt>
    where
        Tgt: Target<'target>,
    {
        let n = keys.len();
        assert_eq!(n, values.len(), "mismatched number of keys and values");

        let field_names = self.field_names();
        let n_fields = field_names.len();

        target.with_unsized_local_scope(n_fields, |target, mut frame| {
            let mut retained_keys = Vec::with_capacity(n_fields - remove.len());
            let mut retained_values = Vec::with_capacity(n_fields - remove.len());

            for key in field_names.iter().copied() {
                if !remove.contains(&key) {
                    let value = self
                        .as_value()
                        .get_field(&mut frame, key)
                        .expect("missing field");

                    retained_keys.push(key);
                    retained_values.push(value);
                }
            }

            let mut map = FnvHashMap::default();
            for (key, value) in retained_keys
                .iter()
                .copied()
                .zip(retained_values.iter().copied())
            {
                map.insert(key, value);
            }
            for (key, value) in keys.iter().copied().zip(values.iter().copied()) {
                map.insert(key, value);
            }

            unsafe { Self::from_iter_unchecked(target, map.iter().map(|(a, b)| (*a, *b))) }
        })
    }

    /// Combines [`Self::filter`] and [`Self::extend_iter`]. All duplicates are removed, later
    /// pairs win.
    pub fn filter_extend_iter<'target, 'd, Tgt>(
        self,
        target: Tgt,
        remove: &[Symbol],
        items: impl ExactSizeIterator<Item = (Symbol<'d>, Value<'d, 'data>)>,
    ) -> NamedTupleData<'target, 'data, Tgt>
    where
        Tgt: Target<'target>,
    {
        let field_names = self.field_names();
        let n_roots = field_names.len();

        target.with_unsized_local_scope(n_roots, |target, mut frame| {
            let mut retained_keys = Vec::with_capacity(n_roots - remove.len());
            let mut retained_values = Vec::with_capacity(n_roots - remove.len());

            for key in field_names.iter().copied() {
                if !remove.contains(&key) {
                    let value = self
                        .as_value()
                        .get_field(&mut frame, key)
                        .expect("missing field");

                    retained_keys.push(key);
                    retained_values.push(value);
                }
            }

            let mut map = FnvHashMap::default();
            for (key, value) in retained_keys
                .iter()
                .copied()
                .zip(retained_values.iter().copied())
            {
                map.insert(key, value);
            }
            for (key, value) in items {
                map.insert(key, value);
            }

            unsafe { Self::from_iter_unchecked(target, map.iter().map(|(a, b)| (*a, *b))) }
        })
    }
}

impl fmt::Debug for NamedTuple<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_value().fmt(f)
    }
}

impl<'scope, 'data> ManagedPriv<'scope, 'data> for NamedTuple<'scope, 'data> {
    type Wraps = jl_value_t;
    type WithLifetimes<'target, 'da> = NamedTuple<'target, 'da>;
    const NAME: &'static str = "NamedTuple";

    // Safety: `inner` must not have been freed yet, the result must never be
    // used after the GC might have freed it.
    #[inline]
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(
            inner,
            ::std::marker::PhantomData,
            ::std::marker::PhantomData,
        )
    }

    #[inline]
    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

unsafe impl Typecheck for NamedTuple<'_, '_> {
    #[inline]
    fn typecheck(t: DataType) -> bool {
        unsafe { t.type_name() == TypeName::of_namedtuple(&Unrooted::new()) }
    }
}

/// A [`NamedTuple`] that has not been explicitly rooted.
pub type WeakNamedTuple<'scope, 'data> = Weak<'scope, 'data, NamedTuple<'scope, 'data>>;

/// A [`WeakNamedTuple`] with static lifetimes. This is a useful shorthand for signatures of
/// `ccall`able functions that return a [`NamedTuple`].
pub type NamedTupleRet = WeakNamedTuple<'static, 'static>;

/// `NamedTuple` or `WeakNamedTuple`, depending on the target type `Tgt`.
pub type NamedTupleData<'target, 'data, Tgt> =
    <Tgt as TargetType<'target>>::Data<'data, NamedTuple<'target, 'data>>;

/// `JuliaResult<NamedTuple>` or `WeakJuliaResult<WeakNamedTuple>`, depending on the target type `Tgt`.
pub type NamedTupleResult<'target, 'data, Tgt> =
    TargetResult<'target, 'data, NamedTuple<'target, 'data>, Tgt>;
