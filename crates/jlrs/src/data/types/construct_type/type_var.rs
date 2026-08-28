//! Type constructors to handle raw `TypeVar`s.

use std::{fmt, marker::PhantomData, sync::atomic::Ordering};

use crate::{
    convert::to_symbol::ToSymbol,
    data::{
        managed::{
            Managed,
            array::Vector,
            datatype::DataType,
            erase_scope_lifetime,
            simple_vector::SimpleVector,
            symbol::Symbol,
            type_var::{TypeVar, TypeVarData},
            value::{Value, ValueData},
        },
        types::{
            abstract_type::AnyType,
            construct_type::{
                ConstructType, bytes::ConstantStr, constants::ConstantChar, union_types::BottomType,
            },
        },
    },
    memory::{
        scope::{LocalScope, LocalScopeExt},
        target::{RootingTarget, Target},
    },
};

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
        $crate::data::types::construct_type::type_var::TypeVarConstructor::<$name>
    };
    ($name:literal) => {
        $crate::data::types::construct_type::type_var::TypeVarConstructor::<
            $crate::data::types::construct_type::type_var::Name<$name>,
        >
    };
    ($name:literal; $ub:ty) => {
        $crate::data::types::construct_type::type_var::TypeVarConstructor::<
            $crate::data::types::construct_type::type_var::Name<$name>,
            $ub,
        >
    };
    ($name:ty; $ub:ty) => {
        $crate::data::types::construct_type::type_var::TypeVarConstructor::<$name, $ub>
    };
    ($lb:ty; $name:literal; $ub:ty) => {
        $crate::data::types::construct_type::type_var::TypeVarConstructor::<
            $crate::data::types::construct_type::type_var::Name<$name>,
            $ub,
            $lb,
        >
    };
    ($lb:ty; $name:ty; $ub:ty) => {
        $crate::data::types::construct_type::type_var::TypeVarConstructor::<$name, $ub, $lb>
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
        $crate::data::types::construct_type::type_var::TypeVarFragment<$t1, $R>
    };
    ($t1:ty, $R:ty, $($rest:ty),+) => {
        $crate::data::types::construct_type::type_var::TypeVarFragment<$t1, tvars!($R, $($rest),+)>
    };
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
        target.with_local_scope::<_, 1>(|target, mut frame| {
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
        target.with_local_scope::<_, 1>(|target, mut frame| {
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
            .map(|elem| unsafe { elem.as_weak().leak().as_managed() })
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

    /// Filter unused `TypeVar`s.
    ///
    /// Safety: `arg_types` must contain the argument types of a function that uses this
    /// environment.
    ///
    /// If a `TypeVar` is used as the argument type of an exported function, it is exposed as a
    /// `UnionAll`. Due to this conversion, `TypeVar`s defined by the environment may end up not
    /// being used. To avoid exposing unused `TypeVar`s to Julia, the environment is filtered with
    /// this method.
    pub unsafe fn filter<'target, Tgt: Target<'target> + RootingTarget<'target>>(
        &self,
        target: Tgt,
        arg_types: Vector,
    ) -> TypeVarEnv<'target> {
        unsafe {
            let mut params = {
                let data = self.svec.data();
                let slice = data.as_atomic_slice().assume_immutable_non_null();
                slice
                    .iter()
                    .map(|v| erase_scope_lifetime(*v))
                    .map(|v| v.cast_unchecked::<TypeVar>())
                    .map(Mask::Unused)
                    .collect::<Vec<_>>()
            };

            if params.len() == 0 {
                return TypeVarEnv::empty(&target);
            }

            // First we check what tvars are depended on by the arguments and mark those as used
            let arg_accessor = arg_types.try_value_data().unwrap();
            let arg_slice = arg_accessor.as_slice();
            for arg_type in arg_slice {
                let arg_type = arg_type.load(Ordering::Relaxed).unwrap().as_value();
                for param in params.iter_mut() {
                    match param {
                        Mask::Used(_) => (),
                        Mask::Unused(tvar) => {
                            if arg_type.depends_on(*tvar) {
                                *param = Mask::Used(*tvar)
                            }
                        }
                    }
                }
            }

            // Remove the unused trailing parameters
            loop {
                match params.last() {
                    Some(Mask::Unused(_)) => {
                        let _ = params.pop();
                    }
                    Some(Mask::Used(_)) => break,
                    None => break,
                }
            }

            if params.len() == 0 {
                return TypeVarEnv::empty(&target);
            }

            // Some tvars might be absent in the signature, but affect a bound of a used tvar.
            // In the environment [T1, T2, T3], T3 can depend on T1 and T2, and T2 can depend on
            // T1. If T2 not mentioned by any of the arguments, it will currently be marked as
            // unused; if T3 depends on this parameter we need to mark it as used.

            // We'll iterate through the environment in reverse
            for i in (0..params.len() - 1).rev() {
                // `head` contains all parameters that may affect the first element of `tail`
                let (head, tail) = params.split_at_mut(i);
                match tail.first() {
                    Some(Mask::Used(tvar)) => {
                        // This parameter is used at this point, so check if it depends on any
                        // currently unused parameter.
                        for param in head {
                            match param {
                                Mask::Used(_) => continue,
                                Mask::Unused(u) => {
                                    if u.depends_on(*tvar) {
                                        *param = Mask::Used(*u);
                                    }
                                }
                            }
                        }
                    }
                    Some(Mask::Unused(_)) => continue,
                    None => continue,
                }
            }

            // Any parameter that is unused at this point can be dropped from the environment.
            let params = params
                .into_iter()
                .filter_map(|param| match param {
                    Mask::Used(param) => Some(param),
                    Mask::Unused(_) => None,
                })
                .collect::<Vec<_>>();

            if params.len() == 0 {
                return TypeVarEnv::empty(&target);
            }

            let svec = Tgt::into_concrete_type(SimpleVector::new(target, &params));
            TypeVarEnv { svec }
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

impl fmt::Debug for TypeVarEnv<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let items = self.svec.data();
        unsafe {
            let data = items.as_atomic_slice().assume_immutable_non_null();
            let fields = data.iter().map(|data| format!("{:?}", data));

            f.debug_set().entries(fields).finish()
        }
    }
}

enum Mask<T> {
    Used(T),
    Unused(T),
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
        target.with_local_scope::<_, 2>(|target, mut frame| {
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
        target.with_local_scope::<_, 2>(|target, mut frame| {
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
