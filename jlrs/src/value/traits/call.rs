//! Call Julia functions.

use crate::{
    error::JlrsResult,
    memory::traits::{frame::Frame, scope::Scope},
    value::{Value, WithKeywords, MAX_SIZE},
};
use jl_sys::{
    jl_call, jl_call0, jl_call1, jl_call2, jl_call3, jl_exception_occurred, jl_get_kwsorter,
};
use smallvec::SmallVec;

use super::private::Internal;

/// A trait that allows something to be called as a Julia function. There are currently two types
/// that implement this trait, [`Value`] and [`WithKeywords`]. In Julia every value can
/// potentially be a function that can be called, there's no general way to confirm if it is.
/// [`WithKeywords`] is used to call functions that take keyword arguments, keywords can be
/// provided to a function by calling [`Value::with_keywords`]. The positional arguments can be
/// provided with this trait's methods.
pub trait Call<'scope, 'frame, 'data> {
    /// Call a function with no arguments.
    fn call0<S, F>(self, scope: S) -> JlrsResult<S::CallResult>
    where
        S: Scope<'scope, 'frame, 'data, F>,
        F: Frame<'frame>;

    /// Call a function with one argument.
    fn call1<S, F>(self, scope: S, arg0: Value<'_, 'data>) -> JlrsResult<S::CallResult>
    where
        S: Scope<'scope, 'frame, 'data, F>,
        F: Frame<'frame>;

    /// Call a function with two arguments.
    fn call2<S, F>(
        self,
        scope: S,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
    ) -> JlrsResult<S::CallResult>
    where
        S: Scope<'scope, 'frame, 'data, F>,
        F: Frame<'frame>;

    /// Call a function with three arguments.
    fn call3<S, F>(
        self,
        scope: S,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
        arg2: Value<'_, 'data>,
    ) -> JlrsResult<S::CallResult>
    where
        S: Scope<'scope, 'frame, 'data, F>,
        F: Frame<'frame>;

    /// Call a function with an arbitrary number arguments.
    fn call<'value, V, S, F>(self, scope: S, args: V) -> JlrsResult<S::CallResult>
    where
        V: AsMut<[Value<'value, 'data>]>,
        S: Scope<'scope, 'frame, 'data, F>,
        F: Frame<'frame>;
}

impl<'scope, 'frame, 'data> Call<'scope, 'frame, 'data> for Value<'_, 'data> {
    fn call0<S, F>(self, scope: S) -> JlrsResult<S::CallResult>
    where
        S: Scope<'scope, 'frame, 'data, F>,
        F: Frame<'frame>,
    {
        unsafe {
            let res = jl_call0(self.ptr());
            let exc = jl_exception_occurred();
            if exc.is_null() {
                scope.call_result(Ok(res), Internal)
            } else {
                scope.call_result(Err(exc), Internal)
            }
        }
    }

    fn call1<S, F>(self, scope: S, arg0: Value<'_, 'data>) -> JlrsResult<S::CallResult>
    where
        S: Scope<'scope, 'frame, 'data, F>,
        F: Frame<'frame>,
    {
        unsafe {
            let res = jl_call1(self.ptr(), arg0.ptr());
            let exc = jl_exception_occurred();
            if exc.is_null() {
                scope.call_result(Ok(res), Internal)
            } else {
                scope.call_result(Err(exc), Internal)
            }
        }
    }

    fn call2<S, F>(
        self,
        scope: S,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
    ) -> JlrsResult<S::CallResult>
    where
        S: Scope<'scope, 'frame, 'data, F>,
        F: Frame<'frame>,
    {
        unsafe {
            let res = jl_call2(self.ptr(), arg0.ptr(), arg1.ptr());
            let exc = jl_exception_occurred();
            if exc.is_null() {
                scope.call_result(Ok(res), Internal)
            } else {
                scope.call_result(Err(exc), Internal)
            }
        }
    }

    fn call3<S, F>(
        self,
        scope: S,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
        arg2: Value<'_, 'data>,
    ) -> JlrsResult<S::CallResult>
    where
        S: Scope<'scope, 'frame, 'data, F>,
        F: Frame<'frame>,
    {
        unsafe {
            let res = jl_call3(self.ptr(), arg0.ptr(), arg1.ptr(), arg2.ptr());
            let exc = jl_exception_occurred();
            if exc.is_null() {
                scope.call_result(Ok(res), Internal)
            } else {
                scope.call_result(Err(exc), Internal)
            }
        }
    }

    fn call<'value, V, S, F>(self, scope: S, mut args: V) -> JlrsResult<S::CallResult>
    where
        V: AsMut<[Value<'value, 'data>]>,
        S: Scope<'scope, 'frame, 'data, F>,
        F: Frame<'frame>,
    {
        unsafe {
            let args = args.as_mut();
            let n = args.len();
            let res = jl_call(self.ptr().cast(), args.as_mut_ptr().cast(), n as _);
            let exc = jl_exception_occurred();
            if exc.is_null() {
                scope.call_result(Ok(res), Internal)
            } else {
                scope.call_result(Err(exc), Internal)
            }
        }
    }
}

impl<'scope, 'frame, 'data> Call<'scope, 'frame, 'data> for WithKeywords<'_, '_, 'data> {
    fn call0<S, F>(self, scope: S) -> JlrsResult<S::CallResult>
    where
        S: Scope<'scope, 'frame, 'data, F>,
        F: Frame<'frame>,
    {
        unsafe {
            let func = jl_get_kwsorter(self.func.datatype().expect("").ptr().cast());
            let args = &mut [self.kws, self.func];
            let n = args.len();

            let res = jl_call(func, args.as_mut_ptr().cast(), n as _);
            let exc = jl_exception_occurred();
            if exc.is_null() {
                scope.call_result(Ok(res), Internal)
            } else {
                scope.call_result(Err(exc), Internal)
            }
        }
    }

    fn call1<S, F>(self, scope: S, arg0: Value<'_, 'data>) -> JlrsResult<S::CallResult>
    where
        S: Scope<'scope, 'frame, 'data, F>,
        F: Frame<'frame>,
    {
        unsafe {
            let func = jl_get_kwsorter(self.func.datatype().expect("").ptr().cast());
            let args = &mut [self.kws, self.func, arg0];
            let n = args.len();

            let res = jl_call(func, args.as_mut_ptr().cast(), n as _);
            let exc = jl_exception_occurred();
            if exc.is_null() {
                scope.call_result(Ok(res), Internal)
            } else {
                scope.call_result(Err(exc), Internal)
            }
        }
    }

    fn call2<S, F>(
        self,
        scope: S,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
    ) -> JlrsResult<S::CallResult>
    where
        S: Scope<'scope, 'frame, 'data, F>,
        F: Frame<'frame>,
    {
        unsafe {
            let func = jl_get_kwsorter(self.func.datatype().expect("").ptr().cast());
            let args = &mut [self.kws, self.func, arg0, arg1];
            let n = args.len();

            let res = jl_call(func, args.as_mut_ptr().cast(), n as _);
            let exc = jl_exception_occurred();
            if exc.is_null() {
                scope.call_result(Ok(res), Internal)
            } else {
                scope.call_result(Err(exc), Internal)
            }
        }
    }

    fn call3<S, F>(
        self,
        scope: S,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
        arg2: Value<'_, 'data>,
    ) -> JlrsResult<S::CallResult>
    where
        S: Scope<'scope, 'frame, 'data, F>,
        F: Frame<'frame>,
    {
        unsafe {
            let func = jl_get_kwsorter(self.func.datatype().expect("").ptr().cast());
            let args = &mut [self.kws, self.func, arg0, arg1, arg2];
            let n = args.len();

            let res = jl_call(func, args.as_mut_ptr().cast(), n as _);
            let exc = jl_exception_occurred();
            if exc.is_null() {
                scope.call_result(Ok(res), Internal)
            } else {
                scope.call_result(Err(exc), Internal)
            }
        }
    }

    fn call<'value, V, S, F>(self, scope: S, mut args: V) -> JlrsResult<S::CallResult>
    where
        V: AsMut<[Value<'value, 'data>]>,
        S: Scope<'scope, 'frame, 'data, F>,
        F: Frame<'frame>,
    {
        unsafe {
            let func = jl_get_kwsorter(self.func.datatype().expect("").ptr().cast());
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
                scope.call_result(Ok(res), Internal)
            } else {
                scope.call_result(Err(exc), Internal)
            }
        }
    }
}
