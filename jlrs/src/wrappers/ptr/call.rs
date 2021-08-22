//! Call Julia functions.
//!
//! This module provides the [`Call`] and [`CallExt`] traits, their methods can be used to call
//! Julia functions.
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
    error::{JlrsResult, JuliaResultRef},
    memory::{frame::Frame, global::Global, scope::Scope},
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

    /// Returns the function.
    pub fn function(&self) -> Value<'scope, 'data> {
        self.func
    }

    /// Returns the keywords.
    pub fn keywords(&self) -> Value<'scope, 'data> {
        self.kws
    }
}

/// Call the implementor as a Julia function. There are currently three types that implement this
/// trait: [`Value`], [`Function`] and [`WithKeywords`]. In Julia every value can potentially be
/// callable as a function, there's no general way to confirm if it is because not everything that
/// can be called is guaranteed to be a [`Function`].
///
/// Note that all of these methods are unsafe. There are several reasons for this. First and
/// foremost these methods let you call arbitrary Julia functions which can't be checked for
/// correctness. If the second lifetime of an argument is not `'static`, it must never be assigned
/// to a global. If the function returns a task that performs IO, it's not automatically
/// rescheduled.
///
/// [`Function`]: crate::wrappers::ptr::function::Function
pub trait Call<'data>: private::Call {
    /// Call a function with no arguments and root the result in `scope`.
    ///
    /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
    /// correctness. If the function returns a task that performs IO, it's not automatically
    /// rescheduled. If the scope is an `OutputScope`, the result must be returned from the
    /// closure immediately.
    unsafe fn call0<'target, 'current, S, F>(self, scope: S) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'data, F>,
        F: Frame<'current>;

    /// Call a function with one argument and root the result in `scope`.
    ///
    /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
    /// correctness. If the second lifetime of an argument is not `'static`, it must never be
    /// assigned to a global. If the function returns a task that performs IO, it's not
    /// automatically rescheduled. If the scope is an `OutputScope`, the result must be returned
    /// from the closure immediately.
    unsafe fn call1<'target, 'current, S, F>(
        self,
        scope: S,
        arg0: Value<'_, 'data>,
    ) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'data, F>,
        F: Frame<'current>;

    /// Call a function with two arguments and root the result in `scope`.
    ///
    /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
    /// correctness. If the second lifetime of an argument is not `'static`, it must never be
    /// assigned to a global. If the function returns a task that performs IO, it's not
    /// automatically rescheduled. If the scope is an `OutputScope`, the result must be returned
    /// from the closure immediately.
    unsafe fn call2<'target, 'current, S, F>(
        self,
        scope: S,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
    ) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'data, F>,
        F: Frame<'current>;

    /// Call a function with three arguments and root the result in `scope`.
    ///
    /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
    /// correctness. If the second lifetime of an argument is not `'static`, it must never be
    /// assigned to a global. If the function returns a task that performs IO, it's not
    /// automatically rescheduled. If the scope is an `OutputScope`, the result must be returned
    /// from the closure immediately.
    unsafe fn call3<'target, 'current, S, F>(
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
    ///
    /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
    /// correctness. If the second lifetime of an argument is not `'static`, it must never be
    /// assigned to a global. If the function returns a task that performs IO, it's not
    /// automatically rescheduled. If the scope is an `OutputScope`, the result must be returned
    /// from the closure immediately.
    unsafe fn call<'target, 'current, 'value, V, S, F>(
        self,
        scope: S,
        args: V,
    ) -> JlrsResult<S::JuliaResult>
    where
        V: AsMut<[Value<'value, 'data>]>,
        S: Scope<'target, 'current, 'data, F>,
        F: Frame<'current>;

    /// Call a function with no arguments without rooting the result.
    ///
    /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
    /// correctness. If the function returns a task that performs IO, it's not automatically
    /// rescheduled. If the scope is an `OutputScope`, the result must be returned from the
    /// closure immediately.
    unsafe fn call0_unrooted<'target>(self, _: Global<'target>) -> JuliaResultRef<'target, 'data>;

    /// Call a function with one argument without rooting the result.
    ///
    /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
    /// correctness. If the second lifetime of an argument is not `'static`, it must never be
    /// assigned to a global. If the function returns a task that performs IO, it's not
    /// automatically rescheduled. If the scope is an `OutputScope`, the result must be returned
    /// from the closure immediately.
    unsafe fn call1_unrooted<'target>(
        self,
        _: Global<'target>,
        arg0: Value<'_, 'data>,
    ) -> JuliaResultRef<'target, 'data>;

    /// Call a function with two arguments without rooting the result.
    ///
    /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
    /// correctness. If the second lifetime of an argument is not `'static`, it must never be
    /// assigned to a global. If the function returns a task that performs IO, it's not
    /// automatically rescheduled. If the scope is an `OutputScope`, the result must be returned
    /// from the closure immediately.
    unsafe fn call2_unrooted<'target>(
        self,
        _: Global<'target>,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
    ) -> JuliaResultRef<'target, 'data>;

    /// Call a function with three arguments without rooting the result.
    ///
    /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
    /// correctness. If the second lifetime of an argument is not `'static`, it must never be
    /// assigned to a global. If the function returns a task that performs IO, it's not
    /// automatically rescheduled. If the scope is an `OutputScope`, the result must be returned
    /// from the closure immediately.
    unsafe fn call3_unrooted<'target>(
        self,
        _: Global<'target>,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
        arg2: Value<'_, 'data>,
    ) -> JuliaResultRef<'target, 'data>;

    /// Call a function with an abitrary number of arguments without rooting the result.
    ///
    /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
    /// correctness. If the second lifetime of an argument is not `'static`, it must never be
    /// assigned to a global. If the function returns a task that performs IO, it's not
    /// automatically rescheduled. If the scope is an `OutputScope`, the result must be returned
    /// from the closure immediately.
    unsafe fn call_unrooted<'target, 'value, V>(
        self,
        _: Global<'target>,
        args: V,
    ) -> JuliaResultRef<'target, 'data>
    where
        V: AsMut<[Value<'value, 'data>]>;
}

/// Several additional methods that are only implemented by [`Function`] and [`Value`].
///
/// [`Function`]: crate::wrappers::ptr::function::Function
pub trait CallExt<'target, 'current, 'value, 'data>: Call<'data> {
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
    ///   julia.scope(|global, frame| unsafe {
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
    fn with_keywords(self, kws: Value<'value, 'data>) -> JlrsResult<WithKeywords<'value, 'data>>;
}

impl<'data> Call<'data> for WithKeywords<'_, 'data> {
    unsafe fn call0<'target, 'current, S, F>(self, scope: S) -> JlrsResult<S::JuliaResult>
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

    unsafe fn call1<'target, 'current, S, F>(
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

    unsafe fn call2<'target, 'current, S, F>(
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

    unsafe fn call3<'target, 'current, S, F>(
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

    unsafe fn call<'target, 'current, 'value, V, S, F>(
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

    unsafe fn call0_unrooted<'target>(self, _: Global<'target>) -> JuliaResultRef<'target, 'data> {
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

    unsafe fn call1_unrooted<'target>(
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

    unsafe fn call2_unrooted<'target>(
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

    unsafe fn call3_unrooted<'target>(
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

    unsafe fn call_unrooted<'target, 'value, V>(
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
    use crate::wrappers::ptr::{function::Function, opaque_closure::OpaqueClosure, value::Value};

    use super::WithKeywords;
    pub trait Call {}
    impl Call for WithKeywords<'_, '_> {}
    impl Call for Function<'_, '_> {}
    impl Call for Value<'_, '_> {}
    impl Call for OpaqueClosure<'_> {}
}
