//! Call Julia functions.
//!
//! This module provides the [`Call`], [`CallAsync`] and [`ProvideKeywords`] traits. Their methods
//! can be used to call Julia functions, including inner and outer constructors, schedule and
//! await tasks, and provide keyword argumentes respectively
//!
//! The methods provided by `Call` and `CallAsync` call the implementor as a Julia function with
//! some number of positional arguments. These methods have two variants, either the returned data
//! is rooted or it isn't. It's fine to leave the return value unrooted if you never use it or if
//! you can guarantee that it's reachable from some GC root while you do. Keyword arguments can be
//! provided by calling [`ProvideKeywords::provide_keywords`].

use crate::{
    error::{JlrsResult, JuliaResult, JuliaResultRef},
    memory::{global::Global, scope::PartialScope},
    private::Private,
    wrappers::ptr::{
        private::WrapperPriv as _,
        value::ValueRef,
        value::{Value, MAX_SIZE},
    },
};
use jl_sys::{jl_call, jl_exception_occurred, jl_get_kwsorter};
use smallvec::SmallVec;
use std::ptr::NonNull;

/// A function and its keyword arguments.
pub struct WithKeywords<'scope, 'data> {
    func: Value<'scope, 'data>,
    keywords: Value<'scope, 'data>,
}

impl<'scope, 'data> WithKeywords<'scope, 'data> {
    pub(crate) fn new(func: Value<'scope, 'data>, keywords: Value<'scope, 'data>) -> Self {
        WithKeywords { func, keywords }
    }

    /// Returns the function.
    pub fn function(&self) -> Value<'scope, 'data> {
        self.func
    }

    /// Returns the keywords.
    pub fn keywords(&self) -> Value<'scope, 'data> {
        self.keywords
    }
}

/// Call the implementor as a Julia function.
///
/// There are currently four types that implement this trait: [`Value`], [`Function`],
/// [`WithKeywords`], and [`OpaqueClosure`] if the `internal-types` feature is enabled. Because
/// `Value` implements this trait it's not necessary to cast it before calling it.
///
/// Constructors can be called with the methods defined by this trait, both the inner and outer
/// constructors of a `DataType` can be called by converting the `DataType` to a `Value` and
/// calling it.
///
/// Note that all of these methods are unsafe. There are several reasons for this. First and
/// foremost these methods let you call arbitrary Julia functions which can't be checked for
/// correctness. If the second lifetime of an argument is not `'static`, it must never be assigned
/// to a global. More information can be found in the [`safety`] module.
///
/// [`Function`]: crate::wrappers::ptr::function::Function
/// [`OpaqueClosure`]: crate::wrappers::ptr::internal::opaque_closure::OpaqueClosure
/// [`safety`]: crate::safety
pub trait Call<'data>: private::CallPriv {
    /// Call a function with no arguments and root the result in `scope`.
    ///
    /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
    /// correctness. More information can be found in the [`safety`] module.
    ///
    /// [`safety`]: crate::safety
    unsafe fn call0<'target, S>(self, scope: S) -> JlrsResult<JuliaResult<'target, 'data>>
    where
        S: PartialScope<'target>;

    /// Call a function with one argument and root the result in `scope`.
    ///
    /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
    /// correctness. More information can be found in the [`safety`] module.
    ///
    /// [`safety`]: crate::safety
    unsafe fn call1<'target, S>(
        self,
        scope: S,
        arg0: Value<'_, 'data>,
    ) -> JlrsResult<JuliaResult<'target, 'data>>
    where
        S: PartialScope<'target>;

    /// Call a function with two arguments and root the result in `scope`.
    ///
    /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
    /// correctness. More information can be found in the [`safety`] module.
    ///
    /// [`safety`]: crate::safety
    unsafe fn call2<'target, S>(
        self,
        scope: S,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
    ) -> JlrsResult<JuliaResult<'target, 'data>>
    where
        S: PartialScope<'target>;

    /// Call a function with three arguments and root the result in `scope`.
    ///
    /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
    /// correctness. More information can be found in the [`safety`] module.
    ///
    /// [`safety`]: crate::safety
    unsafe fn call3<'target, S>(
        self,
        scope: S,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
        arg2: Value<'_, 'data>,
    ) -> JlrsResult<JuliaResult<'target, 'data>>
    where
        S: PartialScope<'target>;

    /// Call a function with an arbitrary number arguments and root the result in `scope`.
    ///
    /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
    /// correctness. More information can be found in the [`safety`] module.
    ///
    /// [`safety`]: crate::safety
    unsafe fn call<'target, 'value, V, S>(
        self,
        scope: S,
        args: V,
    ) -> JlrsResult<JuliaResult<'target, 'data>>
    where
        V: AsRef<[Value<'value, 'data>]>,
        S: PartialScope<'target>;

    /// Call a function with no arguments without rooting the result.
    ///
    /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
    /// correctness. More information can be found in the [`safety`] module.
    ///
    /// [`safety`]: crate::safety
    unsafe fn call0_unrooted<'target>(self, _: Global<'target>) -> JuliaResultRef<'target, 'data>;

    /// Call a function with one argument without rooting the result.
    ///
    /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
    /// correctness. More information can be found in the [`safety`] module.
    ///
    /// [`safety`]: crate::safety
    unsafe fn call1_unrooted<'target>(
        self,
        _: Global<'target>,
        arg0: Value<'_, 'data>,
    ) -> JuliaResultRef<'target, 'data>;

    /// Call a function with two arguments without rooting the result.
    ///
    /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
    /// correctness. More information can be found in the [`safety`] module.
    ///
    /// [`safety`]: crate::safety
    unsafe fn call2_unrooted<'target>(
        self,
        _: Global<'target>,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
    ) -> JuliaResultRef<'target, 'data>;

    /// Call a function with three arguments without rooting the result.
    ///
    /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
    /// correctness. More information can be found in the [`safety`] module.
    ///
    /// [`safety`]: crate::safety
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
    /// correctness. More information can be found in the [`safety`] module.
    ///
    /// [`safety`]: crate::safety
    unsafe fn call_unrooted<'target, 'value, V>(
        self,
        _: Global<'target>,
        args: V,
    ) -> JuliaResultRef<'target, 'data>
    where
        V: AsRef<[Value<'value, 'data>]>;
}

/// Provide keyword arguments to a Julia function.
pub trait ProvideKeywords<'value, 'data>: Call<'data> {
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
    /// julia.scope(|global, mut frame| unsafe {
    ///     // The code we evaluate is a simple function definition, which is safe.
    ///     let func = unsafe {
    ///         Value::eval_string(&mut frame, "func(; a=3, b=4, c=5) = a + b + c")?
    ///         .into_jlrs_result()?
    ///     };
    ///
    ///     let a = Value::new(&mut frame, 1isize)?;
    ///     let b = Value::new(&mut frame, 2isize)?;
    ///     let nt = named_tuple!(&mut frame, "a" => a, "b" => b)?;
    ///
    ///     // Call the previously defined function. This function simply sums its three
    ///     // keyword arguments and has no side effects, so it's safe to call.
    ///     let res = unsafe {
    ///         func.provide_keywords(nt)?
    ///             .call0(&mut frame)?
    ///             .into_jlrs_result()?
    ///             .unbox::<isize>()?
    ///     };
    ///
    ///     assert_eq!(res, 8);
    ///
    ///     Ok(())
    /// })
    /// # .unwrap();
    /// # });
    /// # }
    fn provide_keywords(
        self,
        keywords: Value<'value, 'data>,
    ) -> JlrsResult<WithKeywords<'value, 'data>>;
}

impl<'data> Call<'data> for WithKeywords<'_, 'data> {
    unsafe fn call0<'target, S>(self, scope: S) -> JlrsResult<JuliaResult<'target, 'data>>
    where
        S: PartialScope<'target>,
    {
        let func = jl_get_kwsorter(self.func.datatype().unwrap(Private).cast());
        let args = &mut [self.keywords, self.func];
        let n = args.len();

        let res = jl_call(func, args.as_mut_ptr().cast(), n as _);
        let exc = jl_exception_occurred();

        if exc.is_null() {
            scope.call_result(Ok(NonNull::new_unchecked(res)), Private)
        } else {
            scope.call_result(Err(NonNull::new_unchecked(exc)), Private)
        }
    }

    unsafe fn call1<'target, S>(
        self,
        scope: S,
        arg0: Value<'_, 'data>,
    ) -> JlrsResult<JuliaResult<'target, 'data>>
    where
        S: PartialScope<'target>,
    {
        let func = jl_get_kwsorter(self.func.datatype().unwrap(Private).cast());
        let args = &mut [self.keywords, self.func, arg0];
        let n = args.len();

        let res = jl_call(func, args.as_mut_ptr().cast(), n as _);
        let exc = jl_exception_occurred();

        if exc.is_null() {
            scope.call_result(Ok(NonNull::new_unchecked(res)), Private)
        } else {
            scope.call_result(Err(NonNull::new_unchecked(exc)), Private)
        }
    }

    unsafe fn call2<'target, S>(
        self,
        scope: S,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
    ) -> JlrsResult<JuliaResult<'target, 'data>>
    where
        S: PartialScope<'target>,
    {
        let func = jl_get_kwsorter(self.func.datatype().unwrap(Private).cast());
        let args = &mut [self.keywords, self.func, arg0, arg1];
        let n = args.len();

        let res = jl_call(func, args.as_mut_ptr().cast(), n as _);
        let exc = jl_exception_occurred();

        if exc.is_null() {
            scope.call_result(Ok(NonNull::new_unchecked(res)), Private)
        } else {
            scope.call_result(Err(NonNull::new_unchecked(exc)), Private)
        }
    }

    unsafe fn call3<'target, S>(
        self,
        scope: S,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
        arg2: Value<'_, 'data>,
    ) -> JlrsResult<JuliaResult<'target, 'data>>
    where
        S: PartialScope<'target>,
    {
        let func = jl_get_kwsorter(self.func.datatype().unwrap(Private).cast());
        let args = &mut [self.keywords, self.func, arg0, arg1, arg2];
        let n = args.len();

        let res = jl_call(func, args.as_mut_ptr().cast(), n as _);
        let exc = jl_exception_occurred();

        if exc.is_null() {
            scope.call_result(Ok(NonNull::new_unchecked(res)), Private)
        } else {
            scope.call_result(Err(NonNull::new_unchecked(exc)), Private)
        }
    }

    unsafe fn call<'target, 'value, V, S>(
        self,
        scope: S,
        args: V,
    ) -> JlrsResult<JuliaResult<'target, 'data>>
    where
        V: AsRef<[Value<'value, 'data>]>,
        S: PartialScope<'target>,
    {
        let func = jl_get_kwsorter(self.func.datatype().unwrap(Private).cast());
        let args = args.as_ref();
        let mut vals: SmallVec<[Value; MAX_SIZE]> = SmallVec::with_capacity(2 + args.len());
        vals.push(self.keywords);
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
        let args = &mut [self.keywords, self.func];

        let res = jl_call(func, args.as_mut_ptr().cast(), 2);
        let exc = jl_exception_occurred();

        if exc.is_null() {
            Ok(ValueRef::wrap(res))
        } else {
            Err(ValueRef::wrap(exc))
        }
    }

    unsafe fn call1_unrooted<'target>(
        self,
        _: Global<'target>,
        arg0: Value<'_, 'data>,
    ) -> JuliaResultRef<'target, 'data> {
        let func = jl_get_kwsorter(self.func.datatype().unwrap(Private).cast());
        let args = &mut [self.keywords, self.func, arg0];

        let res = jl_call(func, args.as_mut_ptr().cast(), 3);
        let exc = jl_exception_occurred();

        if exc.is_null() {
            Ok(ValueRef::wrap(res))
        } else {
            Err(ValueRef::wrap(exc))
        }
    }

    unsafe fn call2_unrooted<'target>(
        self,
        _: Global<'target>,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
    ) -> JuliaResultRef<'target, 'data> {
        let func = jl_get_kwsorter(self.func.datatype().unwrap(Private).cast());
        let args = &mut [self.keywords, self.func, arg0, arg1];

        let res = jl_call(func, args.as_mut_ptr().cast(), 4);
        let exc = jl_exception_occurred();

        if exc.is_null() {
            Ok(ValueRef::wrap(res))
        } else {
            Err(ValueRef::wrap(exc))
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
        let args = &mut [self.keywords, self.func, arg0, arg1, arg2];

        let res = jl_call(func, args.as_mut_ptr().cast(), 5);
        let exc = jl_exception_occurred();

        if exc.is_null() {
            Ok(ValueRef::wrap(res))
        } else {
            Err(ValueRef::wrap(exc))
        }
    }

    unsafe fn call_unrooted<'target, 'value, V>(
        self,
        _: Global<'target>,
        args: V,
    ) -> JuliaResultRef<'target, 'data>
    where
        V: AsRef<[Value<'value, 'data>]>,
    {
        let func = jl_get_kwsorter(self.func.datatype().unwrap(Private).cast());
        let args = args.as_ref();
        let mut vals: SmallVec<[Value; MAX_SIZE]> = SmallVec::with_capacity(2 + args.len());
        vals.push(self.keywords);
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
            Err(ValueRef::wrap(exc))
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "async")] {
        use async_trait::async_trait;
        use crate::{memory::frame::AsyncGcFrame, wrappers::ptr::{Wrapper, task::Task, module::Module, function::Function}, async_util::{future::JuliaFuture, task::yield_task}};
        /// This trait provides async methods to create and schedule `Task`s that resolve when the
        /// `Task` has completed. Sync methods are also provided which only schedule the `Task`,
        /// those methods should only be used from [`PersistentTask::init`].
        ///
        /// [`PersistentTask::init`]: crate::async_util::task::PersistentTask::init
        #[async_trait(?Send)]
        pub trait CallAsync<'data>: Call<'data> {
            /// Call a function on another thread with the given arguments. This method uses
            /// `Base.Threads.@spawn` to call the given function on another thread but return immediately.
            /// While `await`ing the result the async runtime can work on other tasks, the current task
            /// resumes after the function call on the other thread completes.
            ///
            /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
            /// correctness. More information can be found in the [`safety`] module.
            ///
            /// [`safety`]: crate::safety
            async unsafe fn call_async<'frame, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'frame>,
                args: V,
            ) -> JlrsResult<JuliaResult<'frame, 'data>>
            where
                V: AsRef<[Value<'value, 'data>]>;

            /// Does the same thing as [`CallAsync::call_async`], but the task is returned rather than an
            /// awaitable `Future`. This method should only be called in [`PersistentTask::init`],
            /// otherwise it's not guaranteed this task can make progress.
            ///
            /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
            /// correctness. More information can be found in the [`safety`] module.
            ///
            /// [`safety`]: crate::safety
            /// [`PersistentTask::init`]: crate::async_util::task::PersistentTask::init
            unsafe fn schedule_async<'frame, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'frame>,
                args: V,
            ) -> JlrsResult<JuliaResult<Task<'frame>, 'frame, 'data>>
            where
                V: AsRef<[Value<'value, 'data>]>;

            /// Call a function with the given arguments in an `@async` block. Like `call_async`, the
            /// function is not called on the main thread, but on a separate thread that handles all
            /// tasks created by this method. This method should only be used with functions that do very
            /// little computational work but mostly spend their time waiting on IO.
            ///
            /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
            /// correctness. More information can be found in the [`safety`] module.
            ///
            /// [`safety`]: crate::safety
            async unsafe fn call_async_local<'frame, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'frame>,
                args: V,
            ) -> JlrsResult<JuliaResult<'frame, 'data>>
            where
                V: AsRef<[Value<'value, 'data>]>;

            /// Does the same thing as [`CallAsync::call_async_local`], but the task is returned rather
            /// than an awaitable `Future`. This method should only be called in [`PersistentTask::init`],
            /// otherwise it's not guaranteed this task can make progress.
            ///
            /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
            /// correctness. More information can be found in the [`safety`] module.
            ///
            /// [`safety`]: crate::safety
            /// [`PersistentTask::init`]: crate::async_util::task::PersistentTask::init
            unsafe fn schedule_async_local<'frame, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'frame>,
                args: V,
            ) -> JlrsResult<JuliaResult<Task<'frame>, 'frame, 'data>>
            where
                V: AsRef<[Value<'value, 'data>]>;

            /// Call a function with the given arguments in an `@async` block. The task is scheduled on
            /// the main thread. This method should only be used with functions that must run on the main
            /// thread. The runtime is blocked while this task is active.
            ///
            /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
            /// correctness. More information can be found in the [`safety`] module.
            ///
            /// [`safety`]: crate::safety
            async unsafe fn call_async_main<'frame, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'frame>,
                args: V,
            ) -> JlrsResult<JuliaResult<'frame, 'data>>
            where
                V: AsRef<[Value<'value, 'data>]>;

            /// Does the same thing as [`CallAsync::call_async_main`], but the task is returned rather
            /// than an awaitable `Future`. This method should only be called in [`PersistentTask::init`],
            /// otherwise it's not guaranteed this task can make progress.
            ///
            /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
            /// correctness. More information can be found in the [`safety`] module.
            ///
            /// [`safety`]: crate::safety
            /// [`PersistentTask::init`]: crate::async_util::task::PersistentTask::init
            unsafe fn schedule_async_main<'frame, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'frame>,
                args: V,
            ) -> JlrsResult<JuliaResult<Task<'frame>, 'frame, 'data>>
            where
                V: AsRef<[Value<'value, 'data>]>;
        }

        #[async_trait(?Send)]
        impl<'data> CallAsync<'data> for Value<'_, 'data> {
            async unsafe fn call_async<'frame, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'frame>,
                args: V,
            ) -> JlrsResult<JuliaResult<'frame, 'data>>
            where
                V: AsRef<[Value<'value, 'data>]>,
            {
                Ok(JuliaFuture::new(frame, self, args)?.await)
            }

            unsafe fn schedule_async<'frame, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'frame>,
                args: V,
            ) -> JlrsResult<JuliaResult<Task<'frame>, 'frame, 'data>>
            where
                V: AsRef<[Value<'value, 'data>]>,
            {
                let values = args.as_ref();
                let mut vals: SmallVec<[Value; MAX_SIZE]> = SmallVec::with_capacity(1 + values.len());

                vals.push(self);
                vals.extend_from_slice(values);

                let global = Global::new();
                let task = Module::main(global)
                    .submodule_ref("JlrsMultitask")?
                    .wrapper_unchecked()
                    .function_ref("asynccall")?
                    .wrapper_unchecked()
                    .call(&mut *frame, &mut vals)?;

                yield_task(frame);

                match task {
                    Ok(t) => Ok(Ok(t.cast_unchecked::<Task>())),
                    Err(e) => Ok(Err(e)),
                }
            }

            async unsafe fn call_async_local<'frame, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'frame>,
                args: V,
            ) -> JlrsResult<JuliaResult<'frame, 'data>>
            where
                V: AsRef<[Value<'value, 'data>]>,
            {
                Ok(JuliaFuture::new_local(frame, self, args)?.await)
            }

            unsafe fn schedule_async_local<'frame, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'frame>,
                args: V,
            ) -> JlrsResult<JuliaResult<Task<'frame>, 'frame, 'data>>
            where
                V: AsRef<[Value<'value, 'data>]>,
            {
                let values = args.as_ref();
                let mut vals: SmallVec<[Value; MAX_SIZE]> = SmallVec::with_capacity(1 + values.len());

                vals.push(self);
                vals.extend_from_slice(values);

                let global = Global::new();
                let task = Module::main(global)
                    .submodule_ref("JlrsMultitask")?
                    .wrapper_unchecked()
                    .function_ref("scheduleasynclocal")?
                    .wrapper_unchecked()
                    .call(&mut *frame, &mut vals)?;

                yield_task(frame);

                match task {
                    Ok(t) => Ok(Ok(t.cast_unchecked::<Task>())),
                    Err(e) => Ok(Err(e)),
                }
            }

            async unsafe fn call_async_main<'frame, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'frame>,
                args: V,
            ) -> JlrsResult<JuliaResult<'frame, 'data>>
            where
                V: AsRef<[Value<'value, 'data>]>,
            {
                Ok(JuliaFuture::new_main(frame, self, args)?.await)
            }

            unsafe fn schedule_async_main<'frame, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'frame>,
                args: V,
            ) -> JlrsResult<JuliaResult<Task<'frame>, 'frame, 'data>>
            where
                V: AsRef<[Value<'value, 'data>]>,
            {
                let values = args.as_ref();
                let mut vals: SmallVec<[Value; MAX_SIZE]> = SmallVec::with_capacity(1 + values.len());

                vals.push(self);
                vals.extend_from_slice(values);

                let global = Global::new();
                let task = Module::main(global)
                    .submodule_ref("JlrsMultitask")?
                    .wrapper_unchecked()
                    .function_ref("scheduleasync")?
                    .wrapper_unchecked()
                    .call(&mut *frame, &mut vals)?;

                yield_task(frame);

                match task {
                    Ok(t) => Ok(Ok(t.cast_unchecked::<Task>())),
                    Err(e) => Ok(Err(e)),
                }
            }
        }

        #[async_trait(?Send)]
        impl<'data> CallAsync<'data> for Function<'_, 'data> {
            async unsafe fn call_async<'frame, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'frame>,
                args: V,
            ) -> JlrsResult<JuliaResult<'frame, 'data>>
            where
                V: AsRef<[Value<'value, 'data>]>,
            {
                Ok(JuliaFuture::new(frame, self.as_value(), args)?.await)
            }

            unsafe fn schedule_async<'frame, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'frame>,
                args: V,
            ) -> JlrsResult<JuliaResult<Task<'frame>, 'frame, 'data>>
            where
                V: AsRef<[Value<'value, 'data>]>,
            {
                self.as_value().schedule_async(frame, args)
            }

            async unsafe fn call_async_local<'frame, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'frame>,
                args: V,
            ) -> JlrsResult<JuliaResult<'frame, 'data>>
            where
                V: AsRef<[Value<'value, 'data>]>,
            {
                Ok(JuliaFuture::new_local(frame, self.as_value(), args)?.await)
            }

            unsafe fn schedule_async_local<'frame, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'frame>,
                args: V,
            ) -> JlrsResult<JuliaResult<Task<'frame>, 'frame, 'data>>
            where
                V: AsRef<[Value<'value, 'data>]>,
            {
                self.as_value().schedule_async_local(frame, args)
            }

            async unsafe fn call_async_main<'frame, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'frame>,
                args: V,
            ) -> JlrsResult<JuliaResult<'frame, 'data>>
            where
                V: AsRef<[Value<'value, 'data>]>,
            {
                Ok(JuliaFuture::new_main(frame, self.as_value(), args)?.await)
            }

            unsafe fn schedule_async_main<'frame, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'frame>,
                args: V,
            ) -> JlrsResult<JuliaResult<Task<'frame>, 'frame, 'data>>
            where
                V: AsRef<[Value<'value, 'data>]>,
            {
                self.as_value().schedule_async_main(frame, args)
            }
        }

        #[async_trait(?Send)]
        impl<'data> CallAsync<'data> for WithKeywords<'_, 'data> {
            async unsafe fn call_async<'frame, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'frame>,
                args: V,
            ) -> JlrsResult<JuliaResult<'frame, 'data>>
            where
                V: AsRef<[Value<'value, 'data>]>,
            {
                Ok(JuliaFuture::new_with_keywords(frame, self, args)?.await)
            }

            unsafe fn schedule_async<'frame, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'frame>,
                args: V,
            ) -> JlrsResult<JuliaResult<Task<'frame>, 'frame, 'data>>
            where
                V: AsRef<[Value<'value, 'data>]>,
            {
                let values = args.as_ref();
                let mut vals: SmallVec<[Value; MAX_SIZE]> = SmallVec::with_capacity(1 + values.len());

                vals.push(self.function());
                vals.extend_from_slice(values);

                let global = Global::new();
                let task = Module::main(global)
                    .submodule_ref("JlrsMultitask")?
                    .wrapper_unchecked()
                    .function_ref("asynccall")?
                    .wrapper_unchecked()
                    .provide_keywords(self.keywords())?
                    .call(&mut *frame, &mut vals)?;

                yield_task(frame);

                match task {
                    Ok(t) => Ok(Ok(t.cast_unchecked::<Task>())),
                    Err(e) => Ok(Err(e)),
                }
            }

            async unsafe fn call_async_local<'frame, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'frame>,
                args: V,
            ) -> JlrsResult<JuliaResult<'frame, 'data>>
            where
                V: AsRef<[Value<'value, 'data>]>,
            {
                Ok(JuliaFuture::new_local_with_keywords(frame, self, args)?.await)
            }

            unsafe fn schedule_async_local<'frame, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'frame>,
                args: V,
            ) -> JlrsResult<JuliaResult<Task<'frame>, 'frame, 'data>>
            where
                V: AsRef<[Value<'value, 'data>]>,
            {
                let values = args.as_ref();
                let mut vals: SmallVec<[Value; MAX_SIZE]> = SmallVec::with_capacity(1 + values.len());

                vals.push(self.function());
                vals.extend_from_slice(values);

                let global = Global::new();
                let task = Module::main(global)
                    .submodule_ref("JlrsMultitask")?
                    .wrapper_unchecked()
                    .function_ref("scheduleasynclocal")?
                    .wrapper_unchecked()
                    .provide_keywords(self.keywords())?
                    .call(&mut *frame, &mut vals)?;

                yield_task(frame);

                match task {
                    Ok(t) => Ok(Ok(t.cast_unchecked::<Task>())),
                    Err(e) => Ok(Err(e)),
                }
            }

            async unsafe fn call_async_main<'frame, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'frame>,
                args: V,
            ) -> JlrsResult<JuliaResult<'frame, 'data>>
            where
                V: AsRef<[Value<'value, 'data>]>,
            {
                Ok(JuliaFuture::new_main_with_keywords(frame, self, args)?.await)
            }

            unsafe fn schedule_async_main<'frame, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'frame>,
                args: V,
            ) -> JlrsResult<JuliaResult<Task<'frame>, 'frame, 'data>>
            where
                V: AsRef<[Value<'value, 'data>]>,
            {
                let values = args.as_ref();
                let mut vals: SmallVec<[Value; MAX_SIZE]> = SmallVec::with_capacity(1 + values.len());

                vals.push(self.function());
                vals.extend_from_slice(values);

                let global = Global::new();
                let task = Module::main(global)
                    .submodule_ref("JlrsMultitask")?
                    .wrapper_unchecked()
                    .function_ref("scheduleasync")?
                    .wrapper_unchecked()
                    .provide_keywords(self.keywords())?
                    .call(&mut *frame, &mut vals)?;

                yield_task(frame);

                match task {
                    Ok(t) => Ok(Ok(t.cast_unchecked::<Task>())),
                    Err(e) => Ok(Err(e)),
                }
            }
        }
    }
}

mod private {
    use crate::wrappers::ptr::{function::Function, value::Value};

    #[cfg(all(
        any(not(feature = "lts"), feature = "all-features-override"),
        feature = "internal-types"
    ))]
    use crate::wrappers::ptr::internal::opaque_closure::OpaqueClosure;

    use super::WithKeywords;
    pub trait CallPriv {}
    impl CallPriv for WithKeywords<'_, '_> {}
    impl CallPriv for Function<'_, '_> {}
    #[cfg(all(
        any(not(feature = "lts"), feature = "all-features-override"),
        feature = "internal-types"
    ))]
    impl CallPriv for OpaqueClosure<'_> {}
    impl CallPriv for Value<'_, '_> {}
}
