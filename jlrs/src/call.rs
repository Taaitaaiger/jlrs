//! Call Julia functions.
//!
//! This module provides two traits that are used to call Julia functions. The [`Call`] trait
//! provides methods to call the implementor as a Julia function with some number of arguments and
//! either root the result or not. There's also [`CallExt`], which most imporantly lets you
//! provide keyword arguments to a function.

use std::ptr::NonNull;

use crate::{
    error::{JlrsError, JlrsResult, JuliaResult, JuliaResultRef},
    layout::typecheck::NamedTuple,
    memory::{
        global::Global,
        traits::{frame::Frame, scope::Scope},
    },
    private::Private,
    wrappers::ptr::{
        function::Function,
        module::Module,
        private::Wrapper as _,
        value::{Value, MAX_SIZE},
        ValueRef, Wrapper,
    },
};
use jl_sys::{
    jl_call, jl_call0, jl_call1, jl_call2, jl_call3, jl_exception_occurred, jl_get_kwsorter,
};
use smallvec::SmallVec;

/// A function and its keyword arguments.
pub struct WithKeywords<'func, 'func_data, 'kw, 'data> {
    func: Value<'func, 'func_data>,
    kws: Value<'kw, 'data>,
}

/// A trait that allows something to be called as a Julia function. There are currently three
/// types that implement this trait, [`Value`], [`Function`] and [`WithKeywords`]. In Julia every
/// value can potentially be a function that can be called, there's no general way to confirm if
/// it is because not everything that can be called is guaranteed to be a [`Function`].
pub trait Call<'target, 'current, 'data> {
    /// Call a function with no arguments.
    fn call0<S, F>(self, scope: S) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'data, F>,
        F: Frame<'current>;

    /// Call a function with one argument.
    fn call1<S, F>(self, scope: S, arg0: Value<'_, 'data>) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'data, F>,
        F: Frame<'current>;

    /// Call a function with two arguments.
    fn call2<S, F>(
        self,
        scope: S,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
    ) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'data, F>,
        F: Frame<'current>;

    /// Call a function with three arguments.
    fn call3<S, F>(
        self,
        scope: S,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
        arg2: Value<'_, 'data>,
    ) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'data, F>,
        F: Frame<'current>;

    /// Call a function with an arbitrary number arguments.
    fn call<'value, V, S, F>(self, scope: S, args: V) -> JlrsResult<S::JuliaResult>
    where
        V: AsMut<[Value<'value, 'data>]>,
        S: Scope<'target, 'current, 'data, F>,
        F: Frame<'current>;

    /// Call a function with no arguments without rooting the result.
    fn call0_unrooted(self, _: Global<'target>) -> JuliaResultRef<'target, 'data>;

    /// Call a function with one argument without rooting the result.
    fn call1_unrooted(
        self,
        _: Global<'target>,
        arg0: Value<'_, 'data>,
    ) -> JuliaResultRef<'target, 'data>;

    /// Call a function with two arguments without rooting the result.
    fn call2_unrooted(
        self,
        _: Global<'target>,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
    ) -> JuliaResultRef<'target, 'data>;

    /// Call a function with three arguments without rooting the result.
    fn call3_unrooted(
        self,
        _: Global<'target>,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
        arg2: Value<'_, 'data>,
    ) -> JuliaResultRef<'target, 'data>;

    /// Call a function with an abitrary number of arguments without rooting the result.
    fn call_unrooted<'value, V>(
        self,
        _: Global<'target>,
        args: V,
    ) -> JuliaResultRef<'target, 'data>
    where
        V: AsMut<[Value<'value, 'data>]>;
}

/// Several additional methods that are only implemented by [`Function`] and [`Value`].
pub trait CallExt<'target, 'current, 'value, 'data>: Call<'target, 'current, 'data> {
    /// Returns a new Julia function that prints the stacktrace if an exception is thrown.
    fn tracing_call<F>(self, frame: &mut F) -> JlrsResult<JuliaResult<'current, 'data>>
    where
        F: Frame<'current>;

    /// Returns a new Julia function that catches the exception if one is thrown and throws a new
    /// exception, a `Jlrs.TracedException` which has two fields, `exc` and `stacktrace`.
    fn attach_stacktrace<F>(self, frame: &mut F) -> JlrsResult<JuliaResult<'current, 'data>>
    where
        F: Frame<'current>;

    /// Returns a new Julia function that prints the stacktrace if an exception is thrown, the new
    /// function is not rooted.
    fn tracing_call_unrooted(
        self,
        global: Global<'target>,
    ) -> JlrsResult<JuliaResultRef<'target, 'data>>;

    /// Returns a new Julia function that catches the exception if one is thrown and throws a new
    /// exception, a `Jlrs.TracedException` which has two fields, `exc` and `stacktrace`. The new
    /// function is not rooted.
    fn attach_stacktrace_unrooted(
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
    fn with_keywords<'kws_scope, 'kws_data>(
        self,
        kws: Value<'kws_scope, 'kws_data>,
    ) -> JlrsResult<WithKeywords<'value, 'data, 'kws_scope, 'kws_data>>;
}

impl<'target, 'current, 'data> Call<'target, 'current, 'data> for Value<'_, 'data> {
    fn call0<S, F>(self, scope: S) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'data, F>,
        F: Frame<'current>,
    {
        unsafe {
            let res = jl_call0(self.unwrap(Private));
            let exc = jl_exception_occurred();

            if exc.is_null() {
                scope.call_result(Ok(NonNull::new_unchecked(res)), Private)
            } else {
                scope.call_result(Err(NonNull::new_unchecked(exc)), Private)
            }
        }
    }

    fn call1<S, F>(self, scope: S, arg0: Value<'_, 'data>) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'data, F>,
        F: Frame<'current>,
    {
        unsafe {
            let res = jl_call1(self.unwrap(Private), arg0.unwrap(Private));
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
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
    ) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'data, F>,
        F: Frame<'current>,
    {
        unsafe {
            let res = jl_call2(
                self.unwrap(Private),
                arg0.unwrap(Private),
                arg1.unwrap(Private),
            );
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
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
        arg2: Value<'_, 'data>,
    ) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'data, F>,
        F: Frame<'current>,
    {
        unsafe {
            let res = jl_call3(
                self.unwrap(Private),
                arg0.unwrap(Private),
                arg1.unwrap(Private),
                arg2.unwrap(Private),
            );
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
        V: AsMut<[Value<'value, 'data>]>,
        S: Scope<'target, 'current, 'data, F>,
        F: Frame<'current>,
    {
        unsafe {
            let args = args.as_mut();
            let n = args.len();
            let res = jl_call(
                self.unwrap(Private).cast(),
                args.as_mut_ptr().cast(),
                n as _,
            );
            let exc = jl_exception_occurred();

            if exc.is_null() {
                scope.call_result(Ok(NonNull::new_unchecked(res)), Private)
            } else {
                scope.call_result(Err(NonNull::new_unchecked(exc)), Private)
            }
        }
    }

    fn call0_unrooted(self, _: Global<'target>) -> JuliaResultRef<'target, 'data> {
        unsafe {
            let res = jl_call0(self.unwrap(Private));
            let exc = jl_exception_occurred();

            if exc.is_null() {
                Ok(ValueRef::wrap(res))
            } else {
                Err(ValueRef::wrap(exc))
            }
        }
    }

    fn call1_unrooted(
        self,
        _: Global<'target>,
        arg0: Value<'_, 'data>,
    ) -> JuliaResultRef<'target, 'data> {
        unsafe {
            let res = jl_call1(self.unwrap(Private), arg0.unwrap(Private));
            let exc = jl_exception_occurred();

            if exc.is_null() {
                Ok(ValueRef::wrap(res))
            } else {
                Err(ValueRef::wrap(exc))
            }
        }
    }

    fn call2_unrooted(
        self,
        _: Global<'target>,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
    ) -> JuliaResultRef<'target, 'data> {
        unsafe {
            let res = jl_call2(
                self.unwrap(Private),
                arg0.unwrap(Private),
                arg1.unwrap(Private),
            );
            let exc = jl_exception_occurred();

            if exc.is_null() {
                Ok(ValueRef::wrap(res))
            } else {
                Err(ValueRef::wrap(exc))
            }
        }
    }

    fn call3_unrooted(
        self,
        _: Global<'target>,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
        arg2: Value<'_, 'data>,
    ) -> JuliaResultRef<'target, 'data> {
        unsafe {
            let res = jl_call3(
                self.unwrap(Private),
                arg0.unwrap(Private),
                arg1.unwrap(Private),
                arg2.unwrap(Private),
            );
            let exc = jl_exception_occurred();

            if exc.is_null() {
                Ok(ValueRef::wrap(res))
            } else {
                Err(ValueRef::wrap(exc))
            }
        }
    }

    fn call_unrooted<'value, V>(
        self,
        _: Global<'target>,
        mut args: V,
    ) -> JuliaResultRef<'target, 'data>
    where
        V: AsMut<[Value<'value, 'data>]>,
    {
        unsafe {
            let args = args.as_mut();
            let n = args.len();
            let res = jl_call(
                self.unwrap(Private).cast(),
                args.as_mut_ptr().cast(),
                n as _,
            );
            let exc = jl_exception_occurred();

            if exc.is_null() {
                Ok(ValueRef::wrap(res))
            } else {
                Err(ValueRef::wrap(exc))
            }
        }
    }
}

impl<'target, 'current, 'value, 'data> CallExt<'target, 'current, 'value, 'data>
    for Value<'value, 'data>
{
    fn attach_stacktrace<F>(self, frame: &mut F) -> JlrsResult<JuliaResult<'current, 'data>>
    where
        F: Frame<'current>,
    {
        unsafe {
            let global = frame.global();
            Module::main(global)
                .submodule_ref("Jlrs")?
                .wrapper_unchecked()
                .function_ref("attachstacktrace")?
                .wrapper_unchecked()
                .call1(&mut *frame, self.as_value())
        }
    }

    fn tracing_call<F>(self, frame: &mut F) -> JlrsResult<JuliaResult<'current, 'data>>
    where
        F: Frame<'current>,
    {
        unsafe {
            let global = frame.global();
            Module::main(global)
                .submodule_ref("Jlrs")?
                .wrapper_unchecked()
                .function_ref("tracingcall")?
                .wrapper_unchecked()
                .call1(&mut *frame, self.as_value())
        }
    }

    fn tracing_call_unrooted(
        self,
        global: Global<'target>,
    ) -> JlrsResult<JuliaResultRef<'target, 'data>> {
        unsafe {
            let func = Module::main(global)
                .submodule_ref("Jlrs")?
                .wrapper_unchecked()
                .function_ref("tracingcall")?
                .wrapper_unchecked()
                .call1_unrooted(global, self.as_value());

            Ok(func)
        }
    }

    fn attach_stacktrace_unrooted(
        self,
        global: Global<'target>,
    ) -> JlrsResult<JuliaResultRef<'target, 'data>> {
        unsafe {
            let func = Module::main(global)
                .submodule_ref("Jlrs")?
                .wrapper_unchecked()
                .function_ref("attachstacktrace")?
                .wrapper_unchecked()
                .call1_unrooted(global, self.as_value());

            Ok(func)
        }
    }

    fn with_keywords<'kws_scope, 'kws_data>(
        self,
        kws: Value<'kws_scope, 'kws_data>,
    ) -> JlrsResult<WithKeywords<'value, 'data, 'kws_scope, 'kws_data>> {
        if !kws.is::<NamedTuple>() {
            Err(JlrsError::NotANamedTuple)?
        }
        Ok(WithKeywords { func: self, kws })
    }
}

impl<'target, 'current, 'data> Call<'target, 'current, 'data> for Function<'_, 'data> {
    fn call0<S, F>(self, scope: S) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'data, F>,
        F: Frame<'current>,
    {
        unsafe {
            let res = jl_call0(self.unwrap(Private));
            let exc = jl_exception_occurred();

            if exc.is_null() {
                scope.call_result(Ok(NonNull::new_unchecked(res)), Private)
            } else {
                scope.call_result(Err(NonNull::new_unchecked(exc)), Private)
            }
        }
    }

    fn call1<S, F>(self, scope: S, arg0: Value<'_, 'data>) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'data, F>,
        F: Frame<'current>,
    {
        unsafe {
            let res = jl_call1(self.unwrap(Private), arg0.unwrap(Private));
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
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
    ) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'data, F>,
        F: Frame<'current>,
    {
        unsafe {
            let res = jl_call2(
                self.unwrap(Private),
                arg0.unwrap(Private),
                arg1.unwrap(Private),
            );
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
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
        arg2: Value<'_, 'data>,
    ) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'data, F>,
        F: Frame<'current>,
    {
        unsafe {
            let res = jl_call3(
                self.unwrap(Private),
                arg0.unwrap(Private),
                arg1.unwrap(Private),
                arg2.unwrap(Private),
            );
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
        V: AsMut<[Value<'value, 'data>]>,
        S: Scope<'target, 'current, 'data, F>,
        F: Frame<'current>,
    {
        unsafe {
            let args = args.as_mut();
            let n = args.len();
            let res = jl_call(
                self.unwrap(Private).cast(),
                args.as_mut_ptr().cast(),
                n as _,
            );
            let exc = jl_exception_occurred();

            if exc.is_null() {
                scope.call_result(Ok(NonNull::new_unchecked(res)), Private)
            } else {
                scope.call_result(Err(NonNull::new_unchecked(exc)), Private)
            }
        }
    }

    fn call0_unrooted(self, _: Global<'target>) -> JuliaResultRef<'target, 'data> {
        unsafe {
            let res = jl_call0(self.unwrap(Private));
            let exc = jl_exception_occurred();

            if exc.is_null() {
                Ok(ValueRef::wrap(res))
            } else {
                Err(ValueRef::wrap(exc))
            }
        }
    }

    fn call1_unrooted(
        self,
        _: Global<'target>,
        arg0: Value<'_, 'data>,
    ) -> JuliaResultRef<'target, 'data> {
        unsafe {
            let res = jl_call1(self.unwrap(Private), arg0.unwrap(Private));
            let exc = jl_exception_occurred();

            if exc.is_null() {
                Ok(ValueRef::wrap(res))
            } else {
                Err(ValueRef::wrap(exc))
            }
        }
    }

    fn call2_unrooted(
        self,
        _: Global<'target>,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
    ) -> JuliaResultRef<'target, 'data> {
        unsafe {
            let res = jl_call2(
                self.unwrap(Private),
                arg0.unwrap(Private),
                arg1.unwrap(Private),
            );
            let exc = jl_exception_occurred();

            if exc.is_null() {
                Ok(ValueRef::wrap(res))
            } else {
                Err(ValueRef::wrap(exc))
            }
        }
    }

    fn call3_unrooted(
        self,
        _: Global<'target>,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
        arg2: Value<'_, 'data>,
    ) -> JuliaResultRef<'target, 'data> {
        unsafe {
            let res = jl_call3(
                self.unwrap(Private),
                arg0.unwrap(Private),
                arg1.unwrap(Private),
                arg2.unwrap(Private),
            );
            let exc = jl_exception_occurred();

            if exc.is_null() {
                Ok(ValueRef::wrap(res))
            } else {
                Err(ValueRef::wrap(exc))
            }
        }
    }

    fn call_unrooted<'value, V>(
        self,
        _: Global<'target>,
        mut args: V,
    ) -> JuliaResultRef<'target, 'data>
    where
        V: AsMut<[Value<'value, 'data>]>,
    {
        unsafe {
            let args = args.as_mut();
            let n = args.len();
            let res = jl_call(
                self.unwrap(Private).cast(),
                args.as_mut_ptr().cast(),
                n as _,
            );
            let exc = jl_exception_occurred();

            if exc.is_null() {
                Ok(ValueRef::wrap(res))
            } else {
                Err(ValueRef::wrap(exc))
            }
        }
    }
}

impl<'target, 'current, 'value, 'data> CallExt<'target, 'current, 'value, 'data>
    for Function<'value, 'data>
{
    fn attach_stacktrace<F>(self, frame: &mut F) -> JlrsResult<JuliaResult<'current, 'data>>
    where
        F: Frame<'current>,
    {
        unsafe {
            let global = frame.global();
            Module::main(global)
                .submodule_ref("Jlrs")?
                .wrapper_unchecked()
                .function_ref("attachstacktrace")?
                .wrapper_unchecked()
                .call1(&mut *frame, self.as_value())
        }
    }

    fn tracing_call<F>(self, frame: &mut F) -> JlrsResult<JuliaResult<'current, 'data>>
    where
        F: Frame<'current>,
    {
        unsafe {
            let global = frame.global();
            Module::main(global)
                .submodule_ref("Jlrs")?
                .wrapper_unchecked()
                .function_ref("tracingcall")?
                .wrapper_unchecked()
                .call1(&mut *frame, self.as_value())
        }
    }

    fn tracing_call_unrooted(
        self,
        global: Global<'target>,
    ) -> JlrsResult<JuliaResultRef<'target, 'data>> {
        unsafe {
            let func = Module::main(global)
                .submodule_ref("Jlrs")?
                .wrapper_unchecked()
                .function_ref("tracingcall")?
                .wrapper_unchecked()
                .call1_unrooted(global, self.as_value());

            Ok(func)
        }
    }

    fn attach_stacktrace_unrooted(
        self,
        global: Global<'target>,
    ) -> JlrsResult<JuliaResultRef<'target, 'data>> {
        unsafe {
            let func = Module::main(global)
                .submodule_ref("Jlrs")?
                .wrapper_unchecked()
                .function_ref("attachstacktrace")?
                .wrapper_unchecked()
                .call1_unrooted(global, self.as_value());

            Ok(func)
        }
    }

    fn with_keywords<'kws_scope, 'kws_data>(
        self,
        kws: Value<'kws_scope, 'kws_data>,
    ) -> JlrsResult<WithKeywords<'value, 'data, 'kws_scope, 'kws_data>> {
        if !kws.is::<NamedTuple>() {
            Err(JlrsError::NotANamedTuple)?
        }
        Ok(WithKeywords {
            func: self.as_value(),
            kws,
        })
    }
}

impl<'target, 'current, 'data> Call<'target, 'current, 'data>
    for WithKeywords<'_, 'data, '_, 'data>
{
    fn call0<S, F>(self, scope: S) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'data, F>,
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

    fn call1<S, F>(self, scope: S, arg0: Value<'_, 'data>) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'data, F>,
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
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
    ) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'data, F>,
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
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
        arg2: Value<'_, 'data>,
    ) -> JlrsResult<S::JuliaResult>
    where
        S: Scope<'target, 'current, 'data, F>,
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
        V: AsMut<[Value<'value, 'data>]>,
        S: Scope<'target, 'current, 'data, F>,
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

    fn call0_unrooted(self, _: Global<'target>) -> JuliaResultRef<'target, 'data> {
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
        arg0: Value<'_, 'data>,
    ) -> JuliaResultRef<'target, 'data> {
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
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
    ) -> JuliaResultRef<'target, 'data> {
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
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
        arg2: Value<'_, 'data>,
    ) -> JuliaResultRef<'target, 'data> {
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
    ) -> JuliaResultRef<'target, 'data>
    where
        V: AsMut<[Value<'value, 'data>]>,
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
