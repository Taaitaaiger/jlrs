//! Call Julia functions.
//!
//! This module provides the [`Call`], [`UnsafeCall`], [`CallExt`] and [`UnsafeCallExt`] traits,
//! the methods of these four traits can be used to call Julia functions in many ways. The safe
//! and unsafe variants of each trait provide the same functionality, but only the methods of the
//! unsafe variant can be used when using data that is borrowed from Rust. Calling Julia functions 
//! with such data is unsafe because this data must never be assigned to a global or outlive the
//! borrow some other way.
//!
//! The methods provided by `Call` are used to call the implementor as a Julia function with zero
//! or more positional arguments. These methods also have two variants, either the return value of
//! the call is rooted and returned as a [`Value`], or it's left unrooted and returned as a
//! [`ValueRef`]. It's fine to leave the return value unrooted if you never use it or if you can
//! guarantee that it's reachable while you do.
//!
//! Keyword arguments can be provided with [`CallExt::with_keywords`]. The other trait methods let
//! you wrap the implementor in another function that returns or prints the stack trace if an 
//! exception is thrown.

use crate::{
    error::{JlrsResult, JuliaResult, JuliaResultRef},
    memory::{
        global::Global, scope::Scope,
        {frame::Frame},
    },
    private::Private,
    wrappers::ptr::{
        private::Wrapper as _,
        value::{Value, MAX_SIZE},
        ValueRef,
    },
};
use jl_sys::{jl_call, jl_exception_occurred, jl_get_kwsorter};
use smallvec::SmallVec;
use std::ptr::NonNull;

/// A function and its keyword arguments.
pub struct WithKeywords<'scope, 'data> {
    func: Value<'scope, 'data>,
    kws: Value<'scope, 'data>,
}

impl<'scope, 'data> WithKeywords<'scope, 'data> {
    pub(crate) fn new(func: Value<'scope, 'data>, kws: Value<'scope, 'data>) -> Self {
        WithKeywords { func, kws }
    }
}

/// Call the implementor as a Julia function. There are currently three types that implement this
/// trait: [`Value`], [`Function`] and [`WithKeywords`]. In Julia every value can potentially be
/// callable as a function, there's no general way to confirm if it is because not everything that 
/// can be called is guaranteed to be a [`Function`].
/// 
/// Note that the methods of this traits do not support values that borrow data from Rust, the
/// methods from the [`UnsafeCall`] trait must be used for such data instead.
pub trait Call<'target, 'current>: private::Call {
    /// Call a function with no arguments and root the result in `scope`.
    fn call0<S, F>(self, scope: S) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'static, F>,
        F: Frame<'current>;

    /// Call a function with one argument and root the result in `scope`.
    fn call1<S, F>(self, scope: S, arg0: Value<'_, 'static>) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'static, F>,
        F: Frame<'current>;

    /// Call a function with two arguments and root the result in `scope`.
    fn call2<S, F>(
        self,
        scope: S,
        arg0: Value<'_, 'static>,
        arg1: Value<'_, 'static>,
    ) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'static, F>,
        F: Frame<'current>;

    /// Call a function with three arguments and root the result in `scope`.
    fn call3<S, F>(
        self,
        scope: S,
        arg0: Value<'_, 'static>,
        arg1: Value<'_, 'static>,
        arg2: Value<'_, 'static>,
    ) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'static, F>,
        F: Frame<'current>;

    /// Call a function with an arbitrary number arguments and root the result in `scope`.
    fn call<'value, V, S, F>(self, scope: S, args: V) -> JlrsResult<S::JuliaResult>
    where
        V: AsMut<[Value<'value, 'static>]>,
        S: Scope<'target, 'current, 'static, F>,
        F: Frame<'current>;

    /// Call a function with no arguments without rooting the result.
    fn call0_unrooted(self, _: Global<'target>) -> JuliaResultRef<'target, 'static>;

    /// Call a function with one argument without rooting the result.
    fn call1_unrooted(
        self,
        _: Global<'target>,
        arg0: Value<'_, 'static>,
    ) -> JuliaResultRef<'target, 'static>;

    /// Call a function with two arguments without rooting the result.
    fn call2_unrooted(
        self,
        _: Global<'target>,
        arg0: Value<'_, 'static>,
        arg1: Value<'_, 'static>,
    ) -> JuliaResultRef<'target, 'static>;

    /// Call a function with three arguments without rooting the result.
    fn call3_unrooted(
        self,
        _: Global<'target>,
        arg0: Value<'_, 'static>,
        arg1: Value<'_, 'static>,
        arg2: Value<'_, 'static>,
    ) -> JuliaResultRef<'target, 'static>;

    /// Call a function with an abitrary number of arguments without rooting the result.
    fn call_unrooted<'value, V>(
        self,
        _: Global<'target>,
        args: V,
    ) -> JuliaResultRef<'target, 'static>
    where
        V: AsMut<[Value<'value, 'static>]>;
}

/// Call the implementor as a Julia function. There are currently three types that implement this
/// trait: [`Value`], [`Function`] and [`WithKeywords`]. In Julia every value can potentially be
/// callable as a function, there's no general way to confirm if it is because not everything that 
/// can be called is guaranteed to be a [`Function`].
/// 
/// Unlike [`Call`], this trait does support working with values that borrow data from Rust. It's
/// your responsibility to guarantee these values are never used after the borrow ends, they 
/// shouldn't be assigned to a global for example.
pub trait UnsafeCall<'target, 'current, 'data>: private::Call {
    /// Call a function with no arguments and root the result in `scope`.
    unsafe fn unsafe_call0<S, F>(self, scope: S) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'data, F>,
        F: Frame<'current>;

    /// Call a function with one argument and root the result in `scope`.
    unsafe fn unsafe_call1<S, F>(
        self,
        scope: S,
        arg0: Value<'_, 'data>,
    ) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'data, F>,
        F: Frame<'current>;

    /// Call a function with two arguments and root the result in `scope`.
    unsafe fn unsafe_call2<S, F>(
        self,
        scope: S,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
    ) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'data, F>,
        F: Frame<'current>;

    /// Call a function with three arguments and root the result in `scope`.
    unsafe fn unsafe_call3<S, F>(
        self,
        scope: S,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
        arg2: Value<'_, 'data>,
    ) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'data, F>,
        F: Frame<'current>;

    /// Call a function with an arbitrary number arguments and root the result in `scope`.
    unsafe fn unsafe_call<'value, V, S, F>(self, scope: S, args: V) -> JlrsResult<S::JuliaResult>
    where
        V: AsMut<[Value<'value, 'data>]>,
        S: Scope<'target, 'current, 'data, F>,
        F: Frame<'current>;

    /// Call a function with no arguments without rooting the result.
    unsafe fn unsafe_call0_unrooted(self, _: Global<'target>) -> JuliaResultRef<'target, 'data>;

    /// Call a function with one argument without rooting the result.
    unsafe fn unsafe_call1_unrooted(
        self,
        _: Global<'target>,
        arg0: Value<'_, 'data>,
    ) -> JuliaResultRef<'target, 'data>;

    /// Call a function with two arguments without rooting the result.
    unsafe fn unsafe_call2_unrooted(
        self,
        _: Global<'target>,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
    ) -> JuliaResultRef<'target, 'data>;

    /// Call a function with three arguments without rooting the result.
    unsafe fn unsafe_call3_unrooted(
        self,
        _: Global<'target>,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
        arg2: Value<'_, 'data>,
    ) -> JuliaResultRef<'target, 'data>;

    /// Call a function with an abitrary number of arguments without rooting the result.
    unsafe fn unsafe_call_unrooted<'value, V>(
        self,
        _: Global<'target>,
        args: V,
    ) -> JuliaResultRef<'target, 'data>
    where
        V: AsMut<[Value<'value, 'data>]>;
}

/// Several additional methods that are only implemented by [`Function`] and [`Value`].
pub trait CallExt<'target, 'current, 'value>: Call<'target, 'current> {
    /// Returns a new Julia function that prints the stacktrace if an exception is thrown.
    fn tracing_call<F>(self, frame: &mut F) -> JlrsResult<JuliaResult<'current, 'static>>
    where
        F: Frame<'current>;

    /// Returns a new Julia function that catches the exception if one is thrown and throws a new
    /// exception, a `Jlrs.TracedException` which has two fields, `exc` and `stacktrace`.
    fn attach_stacktrace<F>(self, frame: &mut F) -> JlrsResult<JuliaResult<'current, 'static>>
    where
        F: Frame<'current>;

    /// Returns a new Julia function that prints the stacktrace if an exception is thrown, the new
    /// function is not rooted.
    fn tracing_call_unrooted(
        self,
        global: Global<'target>,
    ) -> JlrsResult<JuliaResultRef<'target, 'static>>;

    /// Returns a new Julia function that catches the exception if one is thrown and throws a new
    /// exception, a `Jlrs.TracedException` which has two fields, `exc` and `stacktrace`. The new
    /// function is not rooted.
    fn attach_stacktrace_unrooted(
        self,
        global: Global<'target>,
    ) -> JlrsResult<JuliaResultRef<'target, 'static>>;

    /// Provide keyword arguments to the function. The keyword arguments must be a `NamedTuple`.
    ///
    /// Example:
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # use jlrs::util::JULIA;
    /// # fn main() {
    /// # JULIA.with(|j| {
    /// # let mut julia = j.borrow_mut();
    ///   julia.scope(|global, frame| {
    ///       let func = Value::eval_string(&mut *frame, "func(; a=3, b=4, c=5) = a + b + c")?
    ///           .into_jlrs_result()?;
    ///
    ///       let a = Value::new(&mut *frame, 1isize)?;
    ///       let b = Value::new(&mut *frame, 2isize)?;
    ///       let nt = named_tuple!(&mut *frame, "a" => a, "b" => b)?;
    ///
    ///       let res = func.with_keywords(nt)?
    ///           .call0(&mut *frame)?
    ///           .into_jlrs_result()?
    ///           .unbox::<isize>()?;
    ///
    ///       assert_eq!(res, 8);
    ///
    ///       Ok(())
    ///   }).unwrap();
    /// # });
    /// # }
    fn with_keywords(
        self,
        kws: Value<'value, 'static>,
    ) -> JlrsResult<WithKeywords<'value, 'static>>;
}

/// Several additional methods that are only implemented by [`Function`] and [`Value`].
pub trait UnsafeCallExt<'target, 'current, 'value, 'data>:
    UnsafeCall<'target, 'current, 'data>
{
    /// Returns a new Julia function that prints the stacktrace if an exception is thrown.
    unsafe fn unsafe_tracing_call<F>(
        self,
        frame: &mut F,
    ) -> JlrsResult<JuliaResult<'current, 'data>>
    where
        F: Frame<'current>;

    /// Returns a new Julia function that catches the exception if one is thrown and throws a new
    /// exception, a `Jlrs.TracedException` which has two fields, `exc` and `stacktrace`.
    unsafe fn unsafe_attach_stacktrace<F>(
        self,
        frame: &mut F,
    ) -> JlrsResult<JuliaResult<'current, 'data>>
    where
        F: Frame<'current>;

    /// Returns a new Julia function that prints the stacktrace if an exception is thrown, the new
    /// function is not rooted.
    unsafe fn unsafe_tracing_call_unrooted(
        self,
        global: Global<'target>,
    ) -> JlrsResult<JuliaResultRef<'target, 'data>>;

    /// Returns a new Julia function that catches the exception if one is thrown and throws a new
    /// exception, a `Jlrs.TracedException` which has two fields, `exc` and `stacktrace`. The new
    /// function is not rooted.
    unsafe fn unsafe_attach_stacktrace_unrooted(
        self,
        global: Global<'target>,
    ) -> JlrsResult<JuliaResultRef<'target, 'data>>;

    /// Provide keyword arguments to the function. The keyword arguments must be a `NamedTuple`.
    ///
    /// Example:
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # use jlrs::util::JULIA;
    /// # fn main() {
    /// # JULIA.with(|j| {
    /// # let mut julia = j.borrow_mut();
    ///   julia.scope(|global, frame| {
    ///       let func = Value::eval_string(&mut *frame, "func(; a=3, b=4, c=5) = a + b + c")?
    ///           .into_jlrs_result()?;
    ///
    ///       let a = Value::new(&mut *frame, 1isize)?;
    ///       let b = Value::new(&mut *frame, 2isize)?;
    ///       let nt = named_tuple!(&mut *frame, "a" => a, "b" => b)?;
    ///
    ///       let res = func.with_keywords(nt)?
    ///           .call0(&mut *frame)?
    ///           .into_jlrs_result()?
    ///           .unbox::<isize>()?;
    ///
    ///       assert_eq!(res, 8);
    ///
    ///       Ok(())
    ///   }).unwrap();
    /// # });
    /// # }
    unsafe fn unsafe_with_keywords(
        self,
        kws: Value<'value, 'data>,
    ) -> JlrsResult<WithKeywords<'value, 'data>>;
}

impl private::Call for WithKeywords<'_, '_> {}

impl<'target, 'current> Call<'target, 'current> for WithKeywords<'_, 'static> {
    fn call0<S, F>(self, scope: S) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'static, F>,
        F: Frame<'current>,
    {
        unsafe {
            let func = jl_get_kwsorter(self.func.datatype().unwrap(Private).cast());
            let args = &mut [self.kws, self.func];
            let n = args.len();

            let res = jl_call(func, args.as_mut_ptr().cast(), n as _);
            let exc = jl_exception_occurred();

            if exc.is_null() {
                scope.call_result(Ok(NonNull::new_unchecked(res)), Private)
            } else {
                scope.call_result(Err(NonNull::new_unchecked(exc)), Private)
            }
        }
    }

    fn call1<S, F>(self, scope: S, arg0: Value<'_, 'static>) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'static, F>,
        F: Frame<'current>,
    {
        unsafe {
            let func = jl_get_kwsorter(self.func.datatype().unwrap(Private).cast());
            let args = &mut [self.kws, self.func, arg0];
            let n = args.len();

            let res = jl_call(func, args.as_mut_ptr().cast(), n as _);
            let exc = jl_exception_occurred();

            if exc.is_null() {
                scope.call_result(Ok(NonNull::new_unchecked(res)), Private)
            } else {
                scope.call_result(Err(NonNull::new_unchecked(exc)), Private)
            }
        }
    }

    fn call2<S, F>(
        self,
        scope: S,
        arg0: Value<'_, 'static>,
        arg1: Value<'_, 'static>,
    ) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'static, F>,
        F: Frame<'current>,
    {
        unsafe {
            let func = jl_get_kwsorter(self.func.datatype().unwrap(Private).cast());
            let args = &mut [self.kws, self.func, arg0, arg1];
            let n = args.len();

            let res = jl_call(func, args.as_mut_ptr().cast(), n as _);
            let exc = jl_exception_occurred();

            if exc.is_null() {
                scope.call_result(Ok(NonNull::new_unchecked(res)), Private)
            } else {
                scope.call_result(Err(NonNull::new_unchecked(exc)), Private)
            }
        }
    }

    fn call3<S, F>(
        self,
        scope: S,
        arg0: Value<'_, 'static>,
        arg1: Value<'_, 'static>,
        arg2: Value<'_, 'static>,
    ) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'static, F>,
        F: Frame<'current>,
    {
        unsafe {
            let func = jl_get_kwsorter(self.func.datatype().unwrap(Private).cast());
            let args = &mut [self.kws, self.func, arg0, arg1, arg2];
            let n = args.len();

            let res = jl_call(func, args.as_mut_ptr().cast(), n as _);
            let exc = jl_exception_occurred();

            if exc.is_null() {
                scope.call_result(Ok(NonNull::new_unchecked(res)), Private)
            } else {
                scope.call_result(Err(NonNull::new_unchecked(exc)), Private)
            }
        }
    }

    fn call<'value, V, S, F>(self, scope: S, mut args: V) -> JlrsResult<S::JuliaResult>
    where
        V: AsMut<[Value<'value, 'static>]>,
        S: Scope<'target, 'current, 'static, F>,
        F: Frame<'current>,
    {
        unsafe {
            let func = jl_get_kwsorter(self.func.datatype().unwrap(Private).cast());
            let args = args.as_mut();
            let mut vals: SmallVec<[Value; MAX_SIZE]> = SmallVec::with_capacity(2 + args.len());
            vals.push(self.kws);
            vals.push(self.func);

            for arg in args.iter().copied() {
                vals.push(arg);
            }

            let n = vals.len();
            let res = jl_call(func, vals.as_mut_ptr().cast(), n as _);
            let exc = jl_exception_occurred();

            if exc.is_null() {
                scope.call_result(Ok(NonNull::new_unchecked(res)), Private)
            } else {
                scope.call_result(Err(NonNull::new_unchecked(exc)), Private)
            }
        }
    }

    fn call0_unrooted(self, _: Global<'target>) -> JuliaResultRef<'target, 'static> {
        unsafe {
            let func = jl_get_kwsorter(self.func.datatype().unwrap(Private).cast());
            let args = &mut [self.kws, self.func];

            let res = jl_call(func, args.as_mut_ptr().cast(), 2);
            let exc = jl_exception_occurred();

            if exc.is_null() {
                Ok(ValueRef::wrap(res))
            } else {
                Ok(ValueRef::wrap(exc))
            }
        }
    }

    fn call1_unrooted(
        self,
        _: Global<'target>,
        arg0: Value<'_, 'static>,
    ) -> JuliaResultRef<'target, 'static> {
        unsafe {
            let func = jl_get_kwsorter(self.func.datatype().unwrap(Private).cast());
            let args = &mut [self.kws, self.func, arg0];

            let res = jl_call(func, args.as_mut_ptr().cast(), 3);
            let exc = jl_exception_occurred();

            if exc.is_null() {
                Ok(ValueRef::wrap(res))
            } else {
                Ok(ValueRef::wrap(exc))
            }
        }
    }

    fn call2_unrooted(
        self,
        _: Global<'target>,
        arg0: Value<'_, 'static>,
        arg1: Value<'_, 'static>,
    ) -> JuliaResultRef<'target, 'static> {
        unsafe {
            let func = jl_get_kwsorter(self.func.datatype().unwrap(Private).cast());
            let args = &mut [self.kws, self.func, arg0, arg1];

            let res = jl_call(func, args.as_mut_ptr().cast(), 4);
            let exc = jl_exception_occurred();

            if exc.is_null() {
                Ok(ValueRef::wrap(res))
            } else {
                Ok(ValueRef::wrap(exc))
            }
        }
    }

    fn call3_unrooted(
        self,
        _: Global<'target>,
        arg0: Value<'_, 'static>,
        arg1: Value<'_, 'static>,
        arg2: Value<'_, 'static>,
    ) -> JuliaResultRef<'target, 'static> {
        unsafe {
            let func = jl_get_kwsorter(self.func.datatype().unwrap(Private).cast());
            let args = &mut [self.kws, self.func, arg0, arg1, arg2];

            let res = jl_call(func, args.as_mut_ptr().cast(), 5);
            let exc = jl_exception_occurred();

            if exc.is_null() {
                Ok(ValueRef::wrap(res))
            } else {
                Ok(ValueRef::wrap(exc))
            }
        }
    }

    fn call_unrooted<'value, V>(
        self,
        _: Global<'target>,
        mut args: V,
    ) -> JuliaResultRef<'target, 'static>
    where
        V: AsMut<[Value<'value, 'static>]>,
    {
        unsafe {
            let func = jl_get_kwsorter(self.func.datatype().unwrap(Private).cast());
            let args = args.as_mut();
            let mut vals: SmallVec<[Value; MAX_SIZE]> = SmallVec::with_capacity(2 + args.len());
            vals.push(self.kws);
            vals.push(self.func);

            for arg in args.iter().copied() {
                vals.push(arg);
            }

            let n = vals.len();
            let res = jl_call(func, vals.as_mut_ptr().cast(), n as _);
            let exc = jl_exception_occurred();

            if exc.is_null() {
                Ok(ValueRef::wrap(res))
            } else {
                Ok(ValueRef::wrap(exc))
            }
        }
    }
}

impl<'target, 'current, 'data> UnsafeCall<'target, 'current, 'data> for WithKeywords<'_, 'data> {
    unsafe fn unsafe_call0<S, F>(self, scope: S) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'data, F>,
        F: Frame<'current>,
    {
        let func = jl_get_kwsorter(self.func.datatype().unwrap(Private).cast());
        let args = &mut [self.kws, self.func];
        let n = args.len();

        let res = jl_call(func, args.as_mut_ptr().cast(), n as _);
        let exc = jl_exception_occurred();

        if exc.is_null() {
            scope.call_result(Ok(NonNull::new_unchecked(res)), Private)
        } else {
            scope.call_result(Err(NonNull::new_unchecked(exc)), Private)
        }
    }

    unsafe fn unsafe_call1<S, F>(
        self,
        scope: S,
        arg0: Value<'_, 'data>,
    ) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'data, F>,
        F: Frame<'current>,
    {
        let func = jl_get_kwsorter(self.func.datatype().unwrap(Private).cast());
        let args = &mut [self.kws, self.func, arg0];
        let n = args.len();

        let res = jl_call(func, args.as_mut_ptr().cast(), n as _);
        let exc = jl_exception_occurred();

        if exc.is_null() {
            scope.call_result(Ok(NonNull::new_unchecked(res)), Private)
        } else {
            scope.call_result(Err(NonNull::new_unchecked(exc)), Private)
        }
    }

    unsafe fn unsafe_call2<S, F>(
        self,
        scope: S,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
    ) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'data, F>,
        F: Frame<'current>,
    {
        let func = jl_get_kwsorter(self.func.datatype().unwrap(Private).cast());
        let args = &mut [self.kws, self.func, arg0, arg1];
        let n = args.len();

        let res = jl_call(func, args.as_mut_ptr().cast(), n as _);
        let exc = jl_exception_occurred();

        if exc.is_null() {
            scope.call_result(Ok(NonNull::new_unchecked(res)), Private)
        } else {
            scope.call_result(Err(NonNull::new_unchecked(exc)), Private)
        }
    }

    unsafe fn unsafe_call3<S, F>(
        self,
        scope: S,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
        arg2: Value<'_, 'data>,
    ) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'data, F>,
        F: Frame<'current>,
    {
        let func = jl_get_kwsorter(self.func.datatype().unwrap(Private).cast());
        let args = &mut [self.kws, self.func, arg0, arg1, arg2];
        let n = args.len();

        let res = jl_call(func, args.as_mut_ptr().cast(), n as _);
        let exc = jl_exception_occurred();

        if exc.is_null() {
            scope.call_result(Ok(NonNull::new_unchecked(res)), Private)
        } else {
            scope.call_result(Err(NonNull::new_unchecked(exc)), Private)
        }
    }

    unsafe fn unsafe_call<'value, V, S, F>(
        self,
        scope: S,
        mut args: V,
    ) -> JlrsResult<S::JuliaResult>
    where
        V: AsMut<[Value<'value, 'data>]>,
        S: Scope<'target, 'current, 'data, F>,
        F: Frame<'current>,
    {
        let func = jl_get_kwsorter(self.func.datatype().unwrap(Private).cast());
        let args = args.as_mut();
        let mut vals: SmallVec<[Value; MAX_SIZE]> = SmallVec::with_capacity(2 + args.len());
        vals.push(self.kws);
        vals.push(self.func);

        for arg in args.iter().copied() {
            vals.push(arg);
        }

        let n = vals.len();
        let res = jl_call(func, vals.as_mut_ptr().cast(), n as _);
        let exc = jl_exception_occurred();

        if exc.is_null() {
            scope.call_result(Ok(NonNull::new_unchecked(res)), Private)
        } else {
            scope.call_result(Err(NonNull::new_unchecked(exc)), Private)
        }
    }

    unsafe fn unsafe_call0_unrooted(self, _: Global<'target>) -> JuliaResultRef<'target, 'data> {
        let func = jl_get_kwsorter(self.func.datatype().unwrap(Private).cast());
        let args = &mut [self.kws, self.func];

        let res = jl_call(func, args.as_mut_ptr().cast(), 2);
        let exc = jl_exception_occurred();

        if exc.is_null() {
            Ok(ValueRef::wrap(res))
        } else {
            Ok(ValueRef::wrap(exc))
        }
    }

    unsafe fn unsafe_call1_unrooted(
        self,
        _: Global<'target>,
        arg0: Value<'_, 'data>,
    ) -> JuliaResultRef<'target, 'data> {
        let func = jl_get_kwsorter(self.func.datatype().unwrap(Private).cast());
        let args = &mut [self.kws, self.func, arg0];

        let res = jl_call(func, args.as_mut_ptr().cast(), 3);
        let exc = jl_exception_occurred();

        if exc.is_null() {
            Ok(ValueRef::wrap(res))
        } else {
            Ok(ValueRef::wrap(exc))
        }
    }

    unsafe fn unsafe_call2_unrooted(
        self,
        _: Global<'target>,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
    ) -> JuliaResultRef<'target, 'data> {
        let func = jl_get_kwsorter(self.func.datatype().unwrap(Private).cast());
        let args = &mut [self.kws, self.func, arg0, arg1];

        let res = jl_call(func, args.as_mut_ptr().cast(), 4);
        let exc = jl_exception_occurred();

        if exc.is_null() {
            Ok(ValueRef::wrap(res))
        } else {
            Ok(ValueRef::wrap(exc))
        }
    }

    unsafe fn unsafe_call3_unrooted(
        self,
        _: Global<'target>,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
        arg2: Value<'_, 'data>,
    ) -> JuliaResultRef<'target, 'data> {
        let func = jl_get_kwsorter(self.func.datatype().unwrap(Private).cast());
        let args = &mut [self.kws, self.func, arg0, arg1, arg2];

        let res = jl_call(func, args.as_mut_ptr().cast(), 5);
        let exc = jl_exception_occurred();

        if exc.is_null() {
            Ok(ValueRef::wrap(res))
        } else {
            Ok(ValueRef::wrap(exc))
        }
    }

    unsafe fn unsafe_call_unrooted<'value, V>(
        self,
        _: Global<'target>,
        mut args: V,
    ) -> JuliaResultRef<'target, 'data>
    where
        V: AsMut<[Value<'value, 'data>]>,
    {
        let func = jl_get_kwsorter(self.func.datatype().unwrap(Private).cast());
        let args = args.as_mut();
        let mut vals: SmallVec<[Value; MAX_SIZE]> = SmallVec::with_capacity(2 + args.len());
        vals.push(self.kws);
        vals.push(self.func);

        for arg in args.iter().copied() {
            vals.push(arg);
        }

        let n = vals.len();
        let res = jl_call(func, vals.as_mut_ptr().cast(), n as _);
        let exc = jl_exception_occurred();

        if exc.is_null() {
            Ok(ValueRef::wrap(res))
        } else {
            Ok(ValueRef::wrap(exc))
        }
    }
}

pub(crate) mod private {
    pub trait Call {}
}