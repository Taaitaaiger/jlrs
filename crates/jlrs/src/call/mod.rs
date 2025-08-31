//! Call Julia functions.
//!
//! This module provides the [`Call`], [`CallAsync`] and [`ProvideKeywords`] traits. Their methods
//! can be used to call Julia functions, including inner and outer constructors; schedule a
//! function call as a new Julia task; and provide keyword arguments respectively.
//!
//! Let's add a few numbers with Julia's `+` function:
//!
//! ```
//! use jlrs::prelude::*;
//!
//! # fn main() {
//! # let mut julia = Builder::new().start_local().unwrap();
//!
//! julia.local_scope::<_, 9>(|mut frame| {
//!     // Create a few Julia values
//!     let i = Value::new(&mut frame, 2u64);
//!     let j = Value::new(&mut frame, 1u32);
//!     let k = Value::new(&mut frame, 3u16);
//!
//!     // The `+` function can be found in the base module
//!     let add_func = Module::base(&frame)
//!         .global(&mut frame, "+")
//!         .expect("Add function not found");
//!
//!     let i_plus_j = unsafe { add_func.call(&mut frame, [i, j]) };
//!     assert!(i_plus_j.is_ok());
//!     assert_eq!(i_plus_j.unwrap().unbox::<u64>().expect("wrong type"), 3);
//!
//!     // The `+` function accepts any number of variables
//!     let i_plus_j_plus_k = unsafe { add_func.call(&mut frame, [i, j, k]) };
//!     assert!(i_plus_j_plus_k.is_ok());
//!     assert_eq!(
//!         i_plus_j_plus_k.unwrap().unbox::<u64>().expect("wrong type"),
//!         6
//!     );
//!
//!     // You can provide an arbitary number of arguments with `Call::call`
//!     let i_plus_j_plus_k_plus_k = unsafe { add_func.call(&mut frame, [i, j, k, k]) };
//!     assert!(i_plus_j_plus_k_plus_k.is_ok());
//!     assert_eq!(
//!         i_plus_j_plus_k_plus_k
//!             .unwrap()
//!             .unbox::<u64>()
//!             .expect("wrong type"),
//!         9
//!     );
//!
//!     // Exception are caught
//!     let sum_of_nothing = unsafe { add_func.call(&mut frame, []) };
//!     assert!(sum_of_nothing.is_err());
//!
//!     // You can call the function without using a try-catch block with `Call::call_unchecked`
//!     // Be aware that Julia exception handling works by jumping to the nearest catch block. You
//!     // must either guarantee that the function never throws, or use `catch::catch_exceptions`
//!     // to manually create a try-catch block.
//!     let i_plus_j_plus_k_plus_k_unchecked =
//!         unsafe { add_func.call_unchecked(&mut frame, [i, j, k, k]) };
//!     assert_eq!(
//!         i_plus_j_plus_k_plus_k_unchecked
//!             .unbox::<u64>()
//!             .expect("wrong type"),
//!         9
//!     );
//! });
//! # }
//! ```
//!
//! In the example above we added several numbers of different types by calling the same function.
//! Julia functions are generic, they can have multiple methods with different signatures. When a
//! function is called, the method is selected based on the number and types of all arguments.
//! That this selection depends on the type of all function arguments is what makes Julia's
//! functions multiple dispatch.
//!
//! A minor technical detail that's useful to be aware of is that every function has a unique
//! type, and every type in Julia has a method table. If an instance of a type is called as a
//! function this table is used to find the method that is called. Because every type has a
//! method table, every  Julia value is potentially callable. A fun way to see that in action
//! is by making `Int`s callable:
//!
//! ```
//! use jlrs::prelude::*;
//!
//! # fn main() {
//! # let mut julia = Builder::new().start_local().unwrap();
//!
//! julia.local_scope::<_, 4>(|mut frame| {
//!     let i = Value::new(&mut frame, 1isize);
//!     let j = Value::new(&mut frame, 2isize);
//!
//!     // We can 't call `Int`s yet...
//!     let i_plus_j = unsafe { i.call(&mut frame, [j]) };
//!     assert!(i_plus_j.is_err());
//!
//!     unsafe {
//!         // ... but if we add a method to `Int`'s method table...
//!         Value::eval_string(&frame, "(i::Int)(j::Int) = i + j").expect("unexpected exception");
//!     }
//!
//!     // ... we can!
//!     let i_plus_j = unsafe { i.call(&mut frame, [j]) };
//!     assert!(i_plus_j.is_ok());
//!
//!     let i_plus_j = i_plus_j.unwrap().unbox::<isize>().expect("wrong type");
//!
//!     assert_eq!(i_plus_j, 3);
//! });
//! # }
//! ```
//!
//! In the first example we acquired a handle to the `+` function via the `Base` module. The
//! `Base`, `Core` and `Main` modules can be accessed by calling `Module::base`, `Module::core`,
//! and `Module::main`. The root module of a package can be accessed by calling
//! `Module::package_root_module`. Any installed package can be accessed, but you might need to
//! evaluate an explicit `using` statement first:
//!
//! ```
//! use jlrs::prelude::*;
//!
//! # fn main() {
//! # let mut julia = Builder::new().start_local().unwrap();
//!
//! julia.local_scope::<_, 1>(|mut frame| {
//!     let mut lin_alg = Module::package_root_module(&frame, "LinearAlgebra");
//!     if lin_alg.is_none() {
//!         unsafe {
//!             Value::eval_string(&frame, "using LinearAlgebra")
//!                 .expect("LinearAlgebra package has not been installed");
//!         }
//!
//!         lin_alg = Module::package_root_module(&frame, "LinearAlgebra");
//!     }
//!     assert!(lin_alg.is_some());
//!
//!     let mul_mut_func = lin_alg.unwrap().global(&mut frame, "mul!");
//!     assert!(mul_mut_func.is_ok());
//! });
//! # }
//! ```
//!
//! Keyword arguments can be provided by creating a `NamedTuple` with the [`named_tuple`] macro
//! and calling [`ProvideKeywords::provide_keywords`]:
//!
//! ```
//! use jlrs::prelude::*;
//!
//! # fn main() {
//! # let mut julia = Builder::new().start_local().unwrap();
//!
//! julia.local_scope::<_, 5>(|mut frame| {
//!     unsafe {
//!         Value::eval_string(&frame, "my_kw_func(x; kw1=0, kw2=1) = x + kw1 + kw2")
//!             .expect("unexpected exception");
//!     }
//!
//!     let x = Value::new(&mut frame, 0isize);
//!     let kw1 = Value::new(&mut frame, 3isize);
//!     let kws = named_tuple!(&mut frame, "kw1" => kw1).unwrap();
//!
//!     // Access the function in the `Main` module and provide it with our keyword arguments:
//!     let func = Module::main(&frame)
//!         .global(&mut frame, "my_kw_func")
//!         .expect("cannot find `my_kw_func` in `Main` module");
//!
//!     // Positional arguments are provided via `call[n]`:
//!     let res = unsafe { func.call_kw(&mut frame, [x], kws).expect("unexpected exception") };
//!     let unboxed = res.unbox::<isize>().expect("wrong type");
//!
//!     assert_eq!(unboxed, 4);
//! });
//! # }
//! ```
//!
//! Constructors can be called by calling the type object:
//!
//! ```
//! use jlrs::prelude::*;
//!
//! # fn main() {
//! # let mut julia = Builder::new().start_local().unwrap();
//!
//! julia.local_scope::<_, 3>(|mut frame| {
//!     unsafe {
//!         Value::eval_string(&frame, "struct Foo a::Int; b::Int; Foo(a) = new(a, a); end")
//!             .expect("unexpected exception");
//!     }
//!
//!     let foo_ty = Module::main(&frame)
//!         .global(&mut frame, "Foo")
//!         .expect("Cannot find `Foo` in `Main` module");
//!
//!     let v = Value::new(&mut frame, 1isize);
//!     let foo = unsafe { foo_ty.call(&mut frame, [v]) };
//!
//!     assert!(foo.is_ok());
//! });
//! # }
//! ```
//!
//! Constructors of parametric types can be called directly if all parameters can be inferred from
//! the arguments:
//!
//! ```
//! use jlrs::prelude::*;
//!
//! # fn main() {
//! # let mut julia = Builder::new().start_local().unwrap();
//!
//! julia.local_scope::<_, 3>(|mut frame| {
//!     unsafe {
//!         Value::eval_string(&frame, "struct Foo{T} a::T; end").expect("unexpected exception");
//!     }
//!
//!     let foo_ty = Module::main(&frame)
//!         .global(&mut frame, "Foo")
//!         .expect("Cannot find `Foo` in `Main` module");
//!
//!     let v = Value::new(&mut frame, 1isize);
//!     let foo = unsafe { foo_ty.call(&mut frame, [v]) };
//!
//!     assert!(foo.is_ok());
//! });
//! # }
//! ```
//!
//! If some types can't be inferred from the arguments you must apply them manually before trying
//! to call the function:
//!
//! ```
//! use jlrs::{data::managed::union_all::UnionAll, prelude::*};
//!
//! use crate::jlrs::data::types::construct_type::ConstructType;
//!
//! # fn main() {
//! # let mut julia = Builder::new().start_local().unwrap();
//!
//! julia.local_scope::<_, 5>(|mut frame| {
//!     unsafe {
//!         Value::eval_string(&frame, "struct Foo{V,T} a::T; end").expect("unexpected exception");
//!     }
//!
//!     // Foo{V,T}
//!     let foo_ua = Module::main(&frame)
//!         .global(&mut frame, "Foo")
//!         .expect("Cannot find `Foo` in `Main` module")
//!         .cast::<UnionAll>()
//!         .expect("`Foo` is not a `UnionAll`");
//!
//!     // V = true
//!     let true_v = Value::true_v(&frame);
//!     // T = Int
//!     let int_ty = isize::construct_type(&mut frame);
//!
//!     // foo_ty = Foo{true, Int}
//!     let foo_ty = unsafe {
//!         foo_ua
//!             .apply_types(&mut frame, [true_v, int_ty])
//!             .expect("Cannot apply types to `Foo`")
//!     };
//!
//!     let v = Value::new(&mut frame, 1isize);
//!     let foo = unsafe { foo_ty.call(&mut frame, [v]) };
//!
//!     assert!(foo.is_ok());
//! });
//! # }
//! ```
//!
//! [`named_tuple`]: crate::named_tuple

#[cfg(feature = "async")]
pub mod call_async;

use std::ptr::NonNull;

#[cfg(feature = "async")]
pub use call_async::CallAsync;
use jl_sys::{jl_call, jl_exception_occurred};
use jlrs_sys::jlrs_call_unchecked;

use crate::{
    args::Values,
    data::managed::{
        named_tuple::NamedTuple,
        private::ManagedPriv,
        value::{Value, ValueResult},
    },
    memory::target::Target,
    prelude::ValueData,
    private::Private,
    util::kwcall_function,
};

/// A function and its keyword arguments.
#[deprecated = "Use one of the methods from Call or AsyncCall that takes keyword arguments"]
pub struct WithKeywords<'scope, 'data> {
    func: Value<'scope, 'data>,
    keywords: NamedTuple<'scope, 'data>,
}

#[allow(deprecated)]
impl<'scope, 'data> WithKeywords<'scope, 'data> {
    pub(crate) fn new(func: Value<'scope, 'data>, keywords: NamedTuple<'scope, 'data>) -> Self {
        WithKeywords { func, keywords }
    }

    /// Returns the function.
    pub fn function(&self) -> Value<'scope, 'data> {
        self.func
    }

    /// Returns the keywords.
    pub fn keywords(&self) -> NamedTuple<'scope, 'data> {
        self.keywords
    }
}

/// Call Julia functions.
///
/// There are two types that implement this trait: [`Value`] and [`WithKeywords`]. Any `Value`
/// can be called, `WithKeywords` combines a function with its keyword arguments. If you call a
/// value that isn't a function, an error is returned.
///
/// All of these methods are unsafe, there is no way to distinguish between obviously safe
/// functions like `+`, and obviously unsafe ones like `unsafe_load` except through their names.
/// If multithreading is used, either via the multithreaded runtime or internally in Julia,
/// potential thread-safety issues must also be taken into account.
///
/// More information can be found in the [`safety`] module.
///
/// [`safety`]: crate::safety
pub trait Call<'data>: private::CallPriv {
    /// Call a function with no arguments.
    ///
    /// Safety: there is no way to distinguish between obviously safe functions like `+`, and
    /// obviously unsafe ones like `unsafe_load` except through their names. If multithreading is
    /// used, either via the multithreaded runtime or internally in Julia, potential thread-safety
    /// issues must also be taken into account.
    ///
    /// More information can be found in the [`safety`] module.
    ///
    /// [`safety`]: crate::safety
    #[inline]
    #[deprecated = "Use Call::call instead"]
    unsafe fn call0<'target, Tgt>(self, target: Tgt) -> ValueResult<'target, 'data, Tgt>
    where
        Tgt: Target<'target>,
    {
        unsafe { self.call(target, []) }
    }

    /// Call a function with one argument.
    ///
    /// Safety: there is no way to distinguish between obviously safe functions like `+`, and
    /// obviously unsafe ones like `unsafe_load` except through their names. If multithreading is
    /// used, either via the multithreaded runtime or internally in Julia, potential thread-safety
    /// issues must also be taken into account.
    ///
    /// More information can be found in the [`safety`] module.
    ///
    /// [`safety`]: crate::safety
    #[inline]
    #[deprecated = "Use Call::call instead"]
    unsafe fn call1<'target, Tgt>(
        self,
        target: Tgt,
        arg0: Value<'_, 'data>,
    ) -> ValueResult<'target, 'data, Tgt>
    where
        Tgt: Target<'target>,
    {
        unsafe { self.call(target, [arg0]) }
    }

    /// Call a function with two arguments.
    ///
    /// Safety: there is no way to distinguish between obviously safe functions like `+`, and
    /// obviously unsafe ones like `unsafe_load` except through their names. If multithreading is
    /// used, either via the multithreaded runtime or internally in Julia, potential thread-safety
    /// issues must also be taken into account.
    ///
    /// More information can be found in the [`safety`] module.
    ///
    /// [`safety`]: crate::safety
    #[inline]
    #[deprecated = "Use Call::call instead"]
    unsafe fn call2<'target, Tgt>(
        self,
        target: Tgt,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
    ) -> ValueResult<'target, 'data, Tgt>
    where
        Tgt: Target<'target>,
    {
        unsafe { self.call(target, [arg0, arg1]) }
    }

    /// Call a function with three arguments.
    ///
    /// Safety: there is no way to distinguish between obviously safe functions like `+`, and
    /// obviously unsafe ones like `unsafe_load` except through their names. If multithreading is
    /// used, either via the multithreaded runtime or internally in Julia, potential thread-safety
    /// issues must also be taken into account.
    ///
    /// More information can be found in the [`safety`] module.
    ///
    /// [`safety`]: crate::safety
    #[inline]
    #[deprecated = "Use Call::call instead"]
    unsafe fn call3<'target, Tgt>(
        self,
        target: Tgt,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
        arg2: Value<'_, 'data>,
    ) -> ValueResult<'target, 'data, Tgt>
    where
        Tgt: Target<'target>,
    {
        unsafe { self.call(target, [arg0, arg1, arg2]) }
    }

    /// Call a function with an arbitrary number arguments.
    ///
    /// Safety: there is no way to distinguish between obviously safe functions like `+`, and
    /// obviously unsafe ones like `unsafe_load` except through their names. If multithreading is
    /// used, either via the multithreaded runtime or internally in Julia, potential thread-safety
    /// issues must also be taken into account.
    ///
    /// More information can be found in the [`safety`] module.
    ///
    /// [`safety`]: crate::safety
    unsafe fn call<'target, 'value, V, Tgt, const N: usize>(
        self,
        target: Tgt,
        args: V,
    ) -> ValueResult<'target, 'data, Tgt>
    where
        V: Values<'value, 'data, N>,
        Tgt: Target<'target>;

    unsafe fn call_kw<'target, 'value, V, Tgt, const N: usize>(
        self,
        target: Tgt,
        args: V,
        kwargs: NamedTuple<'_, 'data>,
    ) -> ValueResult<'target, 'data, Tgt>
    where
        V: Values<'value, 'data, N>,
        Tgt: Target<'target>;

    /// Call a function with any number of arguments. Exceptions are not caught.
    ///
    /// Other `call`-methods use a try-catch block internally to
    ///
    /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
    /// correctness. More information can be found in the [`safety`] module.
    ///
    /// [`safety`]: crate::safety
    unsafe fn call_unchecked<'target, 'value, V, Tgt, const N: usize>(
        self,
        target: Tgt,
        args: V,
    ) -> ValueData<'target, 'data, Tgt>
    where
        V: Values<'value, 'data, N>,
        Tgt: Target<'target>;
}

/// Provide keyword arguments to a Julia function.
#[deprecated = "Use one of the methods from Call or AsyncCall that takes keyword arguments"]
pub trait ProvideKeywords<'value, 'data>: Call<'data> {
    /// Provide keyword arguments to the function. The keyword arguments must be a `NamedTuple`.
    ///
    /// Example:
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # fn main() {
    /// # let mut julia = Builder::new().start_local().unwrap();
    /// julia
    ///     .local_scope::<_, 5>(|mut frame| unsafe {
    ///         // The code we evaluate is a simple function definition, which is safe.
    ///         let func = unsafe {
    ///             Value::eval_string(&mut frame, "func(; a=3, b=4, c=5) = a + b + c")? // 1
    ///         };
    ///
    ///         let a = Value::new(&mut frame, 1isize); // 2
    ///         let b = Value::new(&mut frame, 2isize); // 3
    ///         let nt = named_tuple!(&mut frame, "a" => a, "b" => b).unwrap(); // 4
    ///
    ///         // Call the previously defined function. This function simply sums its three
    ///         // keyword arguments and has no side effects, so it's safe to call.
    ///         let res = unsafe {
    ///             func.call_kw(&mut frame, [], nt)? // 5
    ///                 .unbox::<isize>()?
    ///         };
    ///
    ///         assert_eq!(res, 8);
    ///         JlrsResult::Ok(())
    ///     }).unwrap();
    /// # }
    #[deprecated = "Use one of the methods from Call or AsyncCall that takes keyword arguments"]
    #[allow(deprecated)]
    fn provide_keywords(self, keywords: NamedTuple<'value, 'data>) -> WithKeywords<'value, 'data>;
}

#[allow(deprecated)]
impl<'data> Call<'data> for WithKeywords<'_, 'data> {
    #[inline]
    unsafe fn call<'target, 'value, V, Tgt, const N: usize>(
        self,
        target: Tgt,
        args: V,
    ) -> ValueResult<'target, 'data, Tgt>
    where
        V: Values<'value, 'data, N>,
        Tgt: Target<'target>,
    {
        unsafe {
            let func = kwcall_function(&target);

            let values = args.into_extended_pointers_with_start(
                [
                    self.keywords().unwrap(Private),
                    self.function().unwrap(Private),
                ],
                Private,
            );
            let values = values.as_ref();

            let res = jl_call(func, values.as_ptr() as *mut _, values.len() as _);
            let exc = jl_exception_occurred();

            let res = if exc.is_null() {
                Ok(NonNull::new_unchecked(res))
            } else {
                Err(NonNull::new_unchecked(exc))
            };

            target.result_from_ptr(res, Private)
        }
    }

    #[inline]
    unsafe fn call_unchecked<'target, 'value, V, Tgt, const N: usize>(
        self,
        target: Tgt,
        args: V,
    ) -> ValueData<'target, 'data, Tgt>
    where
        V: Values<'value, 'data, N>,
        Tgt: Target<'target>,
    {
        unsafe {
            let func = kwcall_function(&target);

            let values = args.into_extended_pointers_with_start(
                [
                    self.keywords().unwrap(Private),
                    self.function().unwrap(Private),
                ],
                Private,
            );
            let values = values.as_ref();

            let res = jlrs_call_unchecked(func, values.as_ptr() as *mut _, values.len() as _);
            target.data_from_ptr(NonNull::new_unchecked(res), Private)
        }
    }

    #[inline]
    unsafe fn call_kw<'target, 'value, V, Tgt, const N: usize>(
        self,
        _target: Tgt,
        _args: V,
        _kwargs: NamedTuple<'_, 'data>,
    ) -> ValueResult<'target, 'data, Tgt>
    where
        V: Values<'value, 'data, N>,
        Tgt: Target<'target>,
    {
        unimplemented!("WithKeywords cannot take additional keyword arguments")
    }
}

mod private {
    #[allow(deprecated)]
    use super::WithKeywords;
    use crate::data::managed::value::Value;
    pub trait CallPriv: Sized {}
    #[allow(deprecated)]
    impl CallPriv for WithKeywords<'_, '_> {}
    impl CallPriv for Value<'_, '_> {}
}
