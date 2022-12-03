//! Call Julia functions.
//!
//! This module provides the [`Call`], [`CallAsync`] and [`ProvideKeywords`] traits. Their methods
//! can be used to call Julia functions, including inner and outer constructors; schedule a
//! function call as a new Julia task; and provide keyword arguments respectively.

use std::ptr::NonNull;

#[cfg(not(any(feature = "julia-1-10", feature = "julia-1-9")))]
use jl_sys::jl_get_kwsorter;
#[cfg(any(feature = "julia-1-10", feature = "julia-1-9"))]
use jl_sys::jl_kwcall_func;
use jl_sys::{jl_call, jl_exception_occurred};
use smallvec::SmallVec;

#[cfg(not(any(feature = "julia-1-10", feature = "julia-1-9")))]
use crate::data::managed::private::ManagedPriv as _;
use crate::{
    data::managed::{
        array::{tracked::ArrayWrapper, Array},
        value::{Value, ValueResult, MAX_SIZE},
    },
    error::{AccessError, JlrsResult, JuliaResult},
    memory::{context::ledger::Ledger, target::Target},
    private::Private,
};

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
/// All of these methods are unsafe, arbitrary Julia functions can't be checked for correctness.
/// More information can be found in the [`safety`] module.
///
/// [`Function`]: crate::data::managed::function::Function
/// [`OpaqueClosure`]: crate::data::managed::internal::opaque_closure::OpaqueClosure
/// [`safety`]: crate::safety
pub trait Call<'data>: private::CallPriv {
    /// Call a function with no arguments.
    ///
    /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
    /// correctness. More information can be found in the [`safety`] module.
    ///
    /// [`safety`]: crate::safety
    unsafe fn call0<'target, T>(self, target: T) -> ValueResult<'target, 'data, T>
    where
        T: Target<'target>;

    /// Call a function with one argument.
    ///
    /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
    /// correctness. More information can be found in the [`safety`] module. This method doesn't
    /// check if the argument is currently borrowed from Rust.
    ///
    /// [`safety`]: crate::safety
    unsafe fn call1<'target, T>(
        self,
        target: T,
        arg0: Value<'_, 'data>,
    ) -> ValueResult<'target, 'data, T>
    where
        T: Target<'target>;

    /// Call a function with two arguments.
    ///
    /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
    /// correctness. More information can be found in the [`safety`] module. This method doesn't
    /// check if any of the arguments is currently borrowed from Rust.
    ///
    /// [`safety`]: crate::safety
    unsafe fn call2<'target, T>(
        self,
        target: T,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
    ) -> ValueResult<'target, 'data, T>
    where
        T: Target<'target>;

    /// Call a function with three arguments.
    ///
    /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
    /// correctness. More information can be found in the [`safety`] module. This method doesn't
    /// check if any of the arguments is currently borrowed from Rust.
    ///
    /// [`safety`]: crate::safety
    unsafe fn call3<'target, T>(
        self,
        target: T,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
        arg2: Value<'_, 'data>,
    ) -> ValueResult<'target, 'data, T>
    where
        T: Target<'target>;

    /// Call a function with an arbitrary number arguments.
    ///
    /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
    /// correctness. More information can be found in the [`safety`] module. This method doesn't
    /// check if any of the arguments is currently borrowed from Rust.
    ///
    /// [`safety`]: crate::safety
    unsafe fn call<'target, 'value, V, T>(
        self,
        target: T,
        args: V,
    ) -> ValueResult<'target, 'data, T>
    where
        V: AsRef<[Value<'value, 'data>]>,
        T: Target<'target>;

    /// Call a function with an arbitrary number arguments.
    ///
    /// Unlike the other methods of this trait, this method checks if any of the arguments is
    /// currently borrowed from Rust, and returns an `AccessError::BorrowError` if any of the
    /// arguments is.
    ///
    /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
    /// correctness. More information can be found in the [`safety`] module.
    ///
    /// [`safety`]: crate::safety
    unsafe fn call_tracked<'target, 'value, V, T>(
        self,
        target: T,
        args: V,
    ) -> JlrsResult<ValueResult<'target, 'data, T>>
    where
        V: AsRef<[Value<'value, 'data>]>,
        T: Target<'target>,
    {
        let args = args.as_ref();
        let res = args
            .iter()
            .map(|arg| -> JlrsResult<()> {
                if let Ok(arr) = arg.cast::<Array>() {
                    let range = arr.data_range();
                    if Ledger::is_borrowed_any(range) {
                        Err(AccessError::BorrowError)?
                    }
                    Ok(())
                } else {
                    let start = arg.data_ptr().as_ptr() as *mut u8;
                    if Ledger::is_borrowed_any(start..start) {
                        Err(AccessError::BorrowError)?
                    }
                    Ok(())
                }
            })
            .find(|f| f.is_err())
            .map_or_else(
                || Ok(self.call(target, args)),
                |_: _| Err(AccessError::BorrowError),
            )?;

        Ok(res)
    }
}

/// Provide keyword arguments to a Julia function.
// TODO: track array?
pub trait ProvideKeywords<'value, 'data>: Call<'data> {
    /// Provide keyword arguments to the function. The keyword arguments must be a `NamedTuple`.
    ///
    /// Example:
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # use jlrs::util::test::JULIA;
    /// # fn main() {
    /// # JULIA.with(|j| {
    /// # let mut julia = j.borrow_mut();
    /// # let mut frame = StackFrame::new();
    /// # let mut julia = julia.instance(&mut frame);
    /// julia.scope(|mut frame| unsafe {
    ///     // The code we evaluate is a simple function definition, which is safe.
    ///     let func = unsafe {
    ///         Value::eval_string(&mut frame, "func(; a=3, b=4, c=5) = a + b + c")
    ///         .into_jlrs_result()?
    ///     };
    ///
    ///     let a = Value::new(&mut frame, 1isize);
    ///     let b = Value::new(&mut frame, 2isize);
    ///     let nt = named_tuple!(frame.as_extended_target(), "a" => a, "b" => b);
    ///
    ///     // Call the previously defined function. This function simply sums its three
    ///     // keyword arguments and has no side effects, so it's safe to call.
    ///     let res = unsafe {
    ///         func.provide_keywords(nt)?
    ///             .call0(&mut frame)
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
    unsafe fn call0<'target, T>(self, target: T) -> ValueResult<'target, 'data, T>
    where
        T: Target<'target>,
    {
        #[cfg(not(any(feature = "julia-1-10", feature = "julia-1-9")))]
        let func = jl_get_kwsorter(self.func.datatype().unwrap(Private).cast());
        #[cfg(any(feature = "julia-1-10", feature = "julia-1-9"))]
        let func = jl_kwcall_func; // jl_get_kwsorter(self.func.datatype().unwrap(Private).cast());
        let args = &mut [self.keywords, self.func];

        let res = jl_call(func, args.as_mut_ptr().cast(), 2);
        let exc = jl_exception_occurred();

        let res = if exc.is_null() {
            Ok(NonNull::new_unchecked(res))
        } else {
            Err(NonNull::new_unchecked(exc))
        };

        target.result_from_ptr(res, Private)
    }

    unsafe fn call1<'target, T>(
        self,
        target: T,
        arg0: Value<'_, 'data>,
    ) -> ValueResult<'target, 'data, T>
    where
        T: Target<'target>,
    {
        #[cfg(not(any(feature = "julia-1-10", feature = "julia-1-9")))]
        let func = jl_get_kwsorter(self.func.datatype().unwrap(Private).cast());
        #[cfg(any(feature = "julia-1-10", feature = "julia-1-9"))]
        let func = jl_kwcall_func;
        let args = &mut [self.keywords, self.func, arg0];

        let res = jl_call(func, args.as_mut_ptr().cast(), 3);
        let exc = jl_exception_occurred();

        let res = if exc.is_null() {
            Ok(NonNull::new_unchecked(res))
        } else {
            Err(NonNull::new_unchecked(exc))
        };

        target.result_from_ptr(res, Private)
    }

    unsafe fn call2<'target, T>(
        self,
        target: T,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
    ) -> ValueResult<'target, 'data, T>
    where
        T: Target<'target>,
    {
        #[cfg(not(any(feature = "julia-1-10", feature = "julia-1-9")))]
        let func = jl_get_kwsorter(self.func.datatype().unwrap(Private).cast());
        #[cfg(any(feature = "julia-1-10", feature = "julia-1-9"))]
        let func = jl_kwcall_func;
        let args = &mut [self.keywords, self.func, arg0, arg1];

        let res = jl_call(func, args.as_mut_ptr().cast(), 4);
        let exc = jl_exception_occurred();

        let res = if exc.is_null() {
            Ok(NonNull::new_unchecked(res))
        } else {
            Err(NonNull::new_unchecked(exc))
        };

        target.result_from_ptr(res, Private)
    }

    unsafe fn call3<'target, T>(
        self,
        target: T,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
        arg2: Value<'_, 'data>,
    ) -> ValueResult<'target, 'data, T>
    where
        T: Target<'target>,
    {
        #[cfg(not(any(feature = "julia-1-10", feature = "julia-1-9")))]
        let func = jl_get_kwsorter(self.func.datatype().unwrap(Private).cast());
        #[cfg(any(feature = "julia-1-10", feature = "julia-1-9"))]
        let func = jl_kwcall_func;
        let args = &mut [self.keywords, self.func, arg0, arg1, arg2];

        let res = jl_call(func, args.as_mut_ptr().cast(), 5);
        let exc = jl_exception_occurred();

        let res = if exc.is_null() {
            Ok(NonNull::new_unchecked(res))
        } else {
            Err(NonNull::new_unchecked(exc))
        };

        target.result_from_ptr(res, Private)
    }

    unsafe fn call<'target, 'value, V, T>(
        self,
        target: T,
        args: V,
    ) -> ValueResult<'target, 'data, T>
    where
        V: AsRef<[Value<'value, 'data>]>,
        T: Target<'target>,
    {
        #[cfg(not(any(feature = "julia-1-10", feature = "julia-1-9")))]
        let func = jl_get_kwsorter(self.func.datatype().unwrap(Private).cast());
        #[cfg(any(feature = "julia-1-10", feature = "julia-1-9"))]
        let func = jl_kwcall_func;
        let args = args.as_ref();
        let mut vals: SmallVec<[Value; MAX_SIZE]> = SmallVec::with_capacity(2 + args.len());
        vals.push(self.keywords);
        vals.push(self.func);
        vals.extend_from_slice(args);

        let n = vals.len();
        let res = jl_call(func, vals.as_mut_ptr().cast(), n as _);
        let exc = jl_exception_occurred();

        let res = if exc.is_null() {
            Ok(NonNull::new_unchecked(res))
        } else {
            Err(NonNull::new_unchecked(exc))
        };

        target.result_from_ptr(res, Private)
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "async")] {
        use async_trait::async_trait;
        use crate::{
            memory::target::frame::AsyncGcFrame,
            data::managed::{
                Managed,
                task::Task,
                module::Module,
                function::Function
            },
            async_util::{
                future::JuliaFuture,
            }
        };

        /// This trait provides async methods to create and schedule `Task`s that resolve when the
        /// `Task` has completed. Sync methods are also provided which only schedule the `Task`,
        /// those methods should only be used from [`PersistentTask::init`].
        ///
        /// [`PersistentTask::init`]: crate::async_util::task::PersistentTask::init
        #[async_trait(?Send)]
        pub trait CallAsync<'data>: Call<'data> {
            /// Creates and schedules a new task with `Base.Threads.@spawn`, and returns a future
            /// that resolves when this task is finished.
            ///
            /// When the `nightly` feature is enabled, this task is spawned on the `:default`
            /// thread pool.
            ///
            /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
            /// correctness. More information can be found in the [`safety`] module. This method doesn't
            /// check if any of the arguments is currently borrowed from Rust.
            ///
            /// [`safety`]: crate::safety
            async unsafe fn call_async<'target, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'target>,
                args: V,
            ) -> JuliaResult<'target, 'data>
            where
                V: AsRef<[Value<'value, 'data>]>;

            /// Creates and schedules a new task with `Base.Threads.@spawn`, and returns a future
            /// that resolves when this task is finished.
            ///
            /// When the `nightly` feature is enabled, this task is spawned on the `:default`
            /// thread pool.
            ///
            /// This method checks if any of the arguments is currently borrowed from Rust, and
            /// returns an `AccessError::BorrowError` if any of the arguments is.
            ///
            /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
            /// correctness. More information can be found in the [`safety`] module.
            ///
            /// [`safety`]: crate::safety
            async unsafe fn call_async_tracked<'target, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'target>,
                args: V,
            ) -> JlrsResult<JuliaResult<'target, 'data>>
            where
                V: AsRef<[Value<'value, 'data>]>
            {
                let args = args.as_ref();
                let res = args
                    .iter()
                    .map(|arg| -> JlrsResult<()> {
                        if let Ok(arr) = arg.cast::<Array>() {
                            let range = arr.data_range();
                            if Ledger::is_borrowed_any(range) {
                                Err(AccessError::BorrowError)?
                            }
                            Ok(())
                        } else {
                            let start = arg.data_ptr().as_ptr() as *mut u8;
                            if Ledger::is_borrowed_any(start..start) {
                                Err(AccessError::BorrowError)?
                            }
                            Ok(())
                        }
                    })
                    .find(|f| f.is_err())
                    .map_or_else(
                         || Ok(async { self.call_async(frame, args).await }),
                        |_: _| Err(AccessError::BorrowError),
                    )?.await;

                Ok(res)
            }

            /// Does the same thing as [`CallAsync::call_async`], but the task is returned rather than an
            /// awaitable `Future`. This method should only be called in [`PersistentTask::init`],
            /// otherwise it's not guaranteed this task can make progress.
            ///
            /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
            /// correctness. More information can be found in the [`safety`] module. This method doesn't
            /// check if any of the arguments is currently borrowed from Rust.
            ///
            /// [`safety`]: crate::safety
            /// [`PersistentTask::init`]: crate::async_util::task::PersistentTask::init
            unsafe fn schedule_async<'target, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'target>,
                args: V,
            ) -> JuliaResult<Task<'target>, 'target, 'data>
            where
                V: AsRef<[Value<'value, 'data>]>;

            /// Does the same thing as [`CallAsync::call_async`], but the task is returned rather than an
            /// awaitable `Future`. This method should only be called in [`PersistentTask::init`],
            /// otherwise it's not guaranteed this task can make progress.
            ///
            /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
            /// correctness. More information can be found in the [`safety`] module.
            ///
            /// This method checks if any of the arguments is currently borrowed from Rust, and
            /// returns an `AccessError::BorrowError` if any of the arguments is.
            ///
            /// [`safety`]: crate::safety
            /// [`PersistentTask::init`]: crate::async_util::task::PersistentTask::init
            unsafe fn schedule_async_tracked<'target, 'value, V, T>(
                self,
                frame: &mut AsyncGcFrame<'target>,
                args: V,
            ) -> JlrsResult<JuliaResult<Task<'target>, 'target, 'data>>
            where
                V: AsRef<[Value<'value, 'data>]>,
                T: Target<'target>,
            {
                let args = args.as_ref();
                let res = args
                    .iter()
                    .map(|arg| -> JlrsResult<()> {
                        if let Ok(arr) = arg.cast::<Array>() {
                            let range = arr.data_range();
                            if Ledger::is_borrowed_any(range) {
                                Err(AccessError::BorrowError)?
                            }
                            Ok(())
                        } else {
                            let start = arg.data_ptr().as_ptr() as *mut u8;
                            if Ledger::is_borrowed_any(start..start) {
                                Err(AccessError::BorrowError)?
                            }
                            Ok(())
                        }
                    })
                    .find(|f| f.is_err())
                    .map_or_else(
                        || Ok(self.schedule_async(frame, args)),
                       |_: _| Err(AccessError::BorrowError),
                   )?;

                Ok(res)
            }

            /// Call a function on another thread with the given arguments. This method uses
            /// `Base.Threads.@spawn` to call the given function on another thread but return immediately.
            /// While `await`ing the result the async runtime can work on other tasks, the current task
            /// resumes after the function call on the other thread completes.
            ///
            /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
            /// correctness. More information can be found in the [`safety`] module. This method doesn't
            /// check if any of the arguments is currently borrowed from Rust.
            ///
            /// [`safety`]: crate::safety
            #[cfg(any(feature = "julia-1-10", feature = "julia-1-9"))]
            async unsafe fn call_async_interactive<'target, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'target>,
                args: V,
            ) -> JuliaResult<'target, 'data>
            where
                V: AsRef<[Value<'value, 'data>]>;

            /// Call a function on another thread with the given arguments. This method uses
            /// `Base.Threads.@spawn` to call the given function on another thread but return immediately.
            /// While `await`ing the result the async runtime can work on other tasks, the current task
            /// resumes after the function call on the other thread completes.
            ///
            /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
            /// correctness. More information can be found in the [`safety`] module.
            ///
            /// This method checks if any of the arguments is currently borrowed from Rust, and
            /// returns an `AccessError::BorrowError` if any of the arguments is.
            ///
            /// [`safety`]: crate::safety
            #[cfg(any(feature = "julia-1-10", feature = "julia-1-9"))]
            async unsafe fn call_async_interactive_tracked<'target, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'target>,
                args: V,
            ) -> JlrsResult<JuliaResult<'target, 'data>>
            where
                V: AsRef<[Value<'value, 'data>]>
            {
                let args = args.as_ref();
                let res = args
                    .iter()
                    .map(|arg| -> JlrsResult<()> {
                        if let Ok(arr) = arg.cast::<Array>() {
                            let range = arr.data_range();
                            if Ledger::is_borrowed_any(range) {
                                Err(AccessError::BorrowError)?
                            }
                            Ok(())
                        } else {
                            let start = arg.data_ptr().as_ptr() as *mut u8;
                            if Ledger::is_borrowed_any(start..start) {
                                Err(AccessError::BorrowError)?
                            }
                            Ok(())
                        }
                    })
                    .find(|f| f.is_err())
                    .map_or_else(
                         || Ok(async { self.call_async_interactive(frame, args).await }),
                        |_: _| Err(AccessError::BorrowError),
                    )?.await;

                Ok(res)
            }

            /// Does the same thing as [`CallAsync::call_async`], but the task is returned rather than an
            /// awaitable `Future`. This method should only be called in [`PersistentTask::init`],
            /// otherwise it's not guaranteed this task can make progress.
            ///
            /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
            /// correctness. More information can be found in the [`safety`] module. This method doesn't
            /// check if any of the arguments is currently borrowed from Rust.
            ///
            /// [`safety`]: crate::safety
            /// [`PersistentTask::init`]: crate::async_util::task::PersistentTask::init
            #[cfg(any(feature = "julia-1-10", feature = "julia-1-9"))]
            unsafe fn schedule_async_interactive<'target, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'target>,
                args: V,
            ) -> JuliaResult<Task<'target>, 'target, 'data>
            where
                V: AsRef<[Value<'value, 'data>]>;


            /// Does the same thing as [`CallAsync::call_async`], but the task is returned rather than an
            /// awaitable `Future`. This method should only be called in [`PersistentTask::init`],
            /// otherwise it's not guaranteed this task can make progress.
            ///
            /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
            /// correctness. More information can be found in the [`safety`] module.
            ///
            /// This method checks if any of the arguments is currently borrowed from Rust, and
            /// returns an `AccessError::BorrowError` if any of the arguments is.
            ///
            /// [`safety`]: crate::safety
            /// [`PersistentTask::init`]: crate::async_util::task::PersistentTask::init
            #[cfg(any(feature = "julia-1-10", feature = "julia-1-9"))]
            unsafe fn schedule_async_interactive_tracked<'target, 'value, V, T>(
                self,
                frame: &mut AsyncGcFrame<'target>,
                args: V,
            ) -> JlrsResult<JuliaResult<Task<'target>, 'target, 'data>>
            where
                V: AsRef<[Value<'value, 'data>]>,
                T: Target<'target>,
            {
                let args = args.as_ref();
                let res = args
                    .iter()
                    .map(|arg| -> JlrsResult<()> {
                        if let Ok(arr) = arg.cast::<Array>() {
                            let range = arr.data_range();
                            if Ledger::is_borrowed_any(range) {
                                Err(AccessError::BorrowError)?
                            }
                            Ok(())
                        } else {
                            let start = arg.data_ptr().as_ptr() as *mut u8;
                            if Ledger::is_borrowed_any(start..start) {
                                Err(AccessError::BorrowError)?
                            }
                            Ok(())
                        }
                    })
                    .find(|f| f.is_err())
                    .map_or_else(
                        || Ok(self.schedule_async_interactive(frame, args)),
                        |_: _| Err(AccessError::BorrowError),
                    )?;

                Ok(res)
            }

            /// Call a function with the given arguments in an `@async` block. Like `call_async`, the
            /// function is not called on the main thread, but on a separate thread that handles all
            /// tasks created by this method. This method should only be used with functions that do very
            /// little computational work but mostly spend their time waiting on IO.
            ///
            /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
            /// correctness. More information can be found in the [`safety`] module. This method doesn't
            /// check if any of the arguments is currently borrowed from Rust.
            ///
            /// [`safety`]: crate::safety
            async unsafe fn call_async_local<'target, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'target>,
                args: V,
            ) -> JuliaResult<'target, 'data>
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
            /// This method checks if any of the arguments is currently borrowed from Rust, and
            /// returns an `AccessError::BorrowError` if any of the arguments is.
            ///
            /// [`safety`]: crate::safety
            async unsafe fn call_async_local_tracked<'target, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'target>,
                args: V,
            ) -> JlrsResult<JuliaResult<'target, 'data>>
            where
                V: AsRef<[Value<'value, 'data>]>
            {
                let args = args.as_ref();
                let res = args
                    .iter()
                    .map(|arg| -> JlrsResult<()> {
                        if let Ok(arr) = arg.cast::<Array>() {
                            let range = arr.data_range();
                            if Ledger::is_borrowed_any(range) {
                                Err(AccessError::BorrowError)?
                            }
                            Ok(())
                        } else {
                            let start = arg.data_ptr().as_ptr() as *mut u8;
                            if Ledger::is_borrowed_any(start..start) {
                                Err(AccessError::BorrowError)?
                            }
                            Ok(())
                        }
                    })
                    .find(|f| f.is_err())
                    .map_or_else(
                         || Ok(async { self.call_async_local(frame, args).await }),
                        |_: _| Err(AccessError::BorrowError),
                    )?.await;

                Ok(res)
            }

            /// Does the same thing as [`CallAsync::call_async_local`], but the task is returned rather
            /// than an awaitable `Future`. This method should only be called in [`PersistentTask::init`],
            /// otherwise it's not guaranteed this task can make progress.
            ///
            /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
            /// correctness. More information can be found in the [`safety`] module. This method doesn't
            /// check if any of the arguments is currently borrowed from Rust.
            ///
            /// [`safety`]: crate::safety
            /// [`PersistentTask::init`]: crate::async_util::task::PersistentTask::init
            unsafe fn schedule_async_local<'target, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'target>,
                args: V,
            ) -> JuliaResult<Task<'target>, 'target, 'data>
            where
                V: AsRef<[Value<'value, 'data>]>;


            /// Does the same thing as [`CallAsync::call_async_local`], but the task is returned rather
            /// than an awaitable `Future`. This method should only be called in [`PersistentTask::init`],
            /// otherwise it's not guaranteed this task can make progress.
            ///
            /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
            /// correctness. More information can be found in the [`safety`] module.
            ///
            /// This method checks if any of the arguments is currently borrowed from Rust, and
            /// returns an `AccessError::BorrowError` if any of the arguments is.
            ///
            /// [`safety`]: crate::safety
            /// [`PersistentTask::init`]: crate::async_util::task::PersistentTask::init
            unsafe fn schedule_async_local_tracked<'target, 'value, V, T>(
                self,
                frame: &mut AsyncGcFrame<'target>,
                args: V,
            ) -> JlrsResult<JuliaResult<Task<'target>, 'target, 'data>>
            where
                V: AsRef<[Value<'value, 'data>]>,
                T: Target<'target>,
            {
                let args = args.as_ref();
                let res = args
                    .iter()
                    .map(|arg| -> JlrsResult<()> {
                        if let Ok(arr) = arg.cast::<Array>() {
                            let range = arr.data_range();
                            if Ledger::is_borrowed_any(range) {
                                Err(AccessError::BorrowError)?
                            }
                            Ok(())
                        } else {
                            let start = arg.data_ptr().as_ptr() as *mut u8;
                            if Ledger::is_borrowed_any(start..start) {
                                Err(AccessError::BorrowError)?
                            }
                            Ok(())
                        }
                    })
                    .find(|f| f.is_err())
                    .map_or_else(
                        || Ok(self.schedule_async_local(frame, args)),
                        |_: _| Err(AccessError::BorrowError),
                    )?;

                Ok(res)
            }

            /// Call a function with the given arguments in an `@async` block. The task is scheduled on
            /// the main thread. This method should only be used with functions that must run on the main
            /// thread. The runtime is blocked while this task is active.
            ///
            /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
            /// correctness. More information can be found in the [`safety`] module. This method doesn't
            /// check if any of the arguments is currently borrowed from Rust.
            ///
            /// [`safety`]: crate::safety
            async unsafe fn call_async_main<'target, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'target>,
                args: V,
            ) -> JuliaResult<'target, 'data>
            where
                V: AsRef<[Value<'value, 'data>]>;


            /// Call a function with the given arguments in an `@async` block. The task is scheduled on
            /// the main thread. This method should only be used with functions that must run on the main
            /// thread. The runtime is blocked while this task is active.
            ///
            /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
            /// correctness. More information can be found in the [`safety`] module.
            ///
            /// This method checks if any of the arguments is currently borrowed from Rust, and
            /// returns an `AccessError::BorrowError` if any of the arguments is.
            ///
            /// [`safety`]: crate::safety
            async unsafe fn call_async_main_tracked<'target, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'target>,
                args: V,
            ) -> JlrsResult<JuliaResult<'target, 'data>>
            where
                V: AsRef<[Value<'value, 'data>]>
            {
                let args = args.as_ref();
                let res = args
                    .iter()
                    .map(|arg| -> JlrsResult<()> {
                        if let Ok(arr) = arg.cast::<Array>() {
                            let range = arr.data_range();
                            if Ledger::is_borrowed_any(range) {
                                Err(AccessError::BorrowError)?
                            }
                            Ok(())
                        } else {
                            let start = arg.data_ptr().as_ptr() as *mut u8;
                            if Ledger::is_borrowed_any(start..start) {
                                Err(AccessError::BorrowError)?
                            }
                            Ok(())
                        }
                    })
                    .find(|f| f.is_err())
                    .map_or_else(
                         || Ok(async { self.call_async_main(frame, args).await }),
                        |_: _| Err(AccessError::BorrowError),
                    )?.await;

                Ok(res)
            }

            /// Does the same thing as [`CallAsync::call_async_main`], but the task is returned rather
            /// than an awaitable `Future`. This method should only be called in [`PersistentTask::init`],
            /// otherwise it's not guaranteed this task can make progress.
            ///
            /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
            /// correctness. More information can be found in the [`safety`] module. This method doesn't
            /// check if any of the arguments is currently borrowed from Rust.
            ///
            /// [`safety`]: crate::safety
            /// [`PersistentTask::init`]: crate::async_util::task::PersistentTask::init
            unsafe fn schedule_async_main<'target, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'target>,
                args: V,
            ) ->JuliaResult<Task<'target>, 'target, 'data>
            where
                V: AsRef<[Value<'value, 'data>]>;

            /// Does the same thing as [`CallAsync::call_async_main`], but the task is returned rather
            /// than an awaitable `Future`. This method should only be called in [`PersistentTask::init`],
            /// otherwise it's not guaranteed this task can make progress.
            ///
            /// Safety: this method lets you call arbitrary Julia functions which can't be checked for
            /// correctness. More information can be found in the [`safety`] module.
            ///
            /// This method checks if any of the arguments is currently borrowed from Rust, and
            /// returns an `AccessError::BorrowError` if any of the arguments is.
            ///
            /// [`safety`]: crate::safety
            /// [`PersistentTask::init`]: crate::async_util::task::PersistentTask::init
            unsafe fn schedule_async_main_tracked<'target, 'value, V, T>(
                self,
                frame: &mut AsyncGcFrame<'target>,
                args: V,
            ) -> JlrsResult<JuliaResult<Task<'target>, 'target, 'data>>
            where
                V: AsRef<[Value<'value, 'data>]>,
                T: Target<'target>,
            {
                let args = args.as_ref();
                let res = args
                    .iter()
                    .map(|arg| -> JlrsResult<()> {
                        if let Ok(arr) = arg.cast::<Array>() {
                            let range = arr.data_range();
                            if Ledger::is_borrowed_any(range) {
                                Err(AccessError::BorrowError)?
                            }
                            Ok(())
                        } else {
                            let start = arg.data_ptr().as_ptr() as *mut u8;
                            if Ledger::is_borrowed_any(start..start) {
                                Err(AccessError::BorrowError)?
                            }
                            Ok(())
                        }
                    })
                    .find(|f| f.is_err())
                    .map_or_else(
                        || Ok(self.schedule_async_main(frame, args)),
                        |_: _| Err(AccessError::BorrowError),
                    )?;

                Ok(res)
            }
        }

        #[async_trait(?Send)]
        impl<'data> CallAsync<'data> for Value<'_, 'data> {
            async unsafe fn call_async<'target, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'target>,
                args: V,
            ) -> JuliaResult<'target, 'data>
            where
                V: AsRef<[Value<'value, 'data>]>,
            {
                JuliaFuture::new(frame, self, args).await
            }

            #[cfg(any(feature = "julia-1-10", feature = "julia-1-9"))]
            async unsafe fn call_async_interactive<'target, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'target>,
                args: V,
            ) -> JuliaResult<'target, 'data>
            where
                V: AsRef<[Value<'value, 'data>]>
            {
                JuliaFuture::new_interactive(frame, self, args).await
            }

            #[cfg(any(feature = "julia-1-10", feature = "julia-1-9"))]
            unsafe fn schedule_async_interactive<'target, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'target>,
                args: V,
            ) -> JuliaResult<Task<'target>, 'target, 'data>
            where
                V: AsRef<[Value<'value, 'data>]>,
            {
                let values = args.as_ref();
                let mut vals: SmallVec<[Value; MAX_SIZE]> = SmallVec::with_capacity(1 + values.len());

                vals.push(self);
                vals.extend_from_slice(values);

                let task = Module::main(&frame)
                    .submodule(&frame, "JlrsMultitask")
                    .expect("JlrsMultitask not available")
                    .as_managed()
                    .function(&frame, "interactivecall")
                    .expect("interactivecall not available")
                    .as_managed()
                    .call(&mut *frame, &mut vals);

                match task {
                    Ok(t) => Ok(t.cast_unchecked::<Task>()),
                    Err(e) => Err(e),
                }
            }

            unsafe fn schedule_async<'target, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'target>,
                args: V,
            ) -> JuliaResult<Task<'target>, 'target, 'data>
            where
                V: AsRef<[Value<'value, 'data>]>,
            {
                let values = args.as_ref();
                let mut vals: SmallVec<[Value; MAX_SIZE]> = SmallVec::with_capacity(1 + values.len());

                vals.push(self);
                vals.extend_from_slice(values);

                let task = Module::main(&frame)
                    .submodule(&frame, "JlrsMultitask")
                    .expect("JlrsMultitask not available")
                    .as_managed()
                    .function(&frame, "asynccall")
                    .expect("asynccall not available")
                    .as_managed()
                    .call(&mut *frame, &mut vals);

                match task {
                    Ok(t) => Ok(t.cast_unchecked::<Task>()),
                    Err(e) => Err(e),
                }
            }

            async unsafe fn call_async_local<'target, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'target>,
                args: V,
            ) -> JuliaResult<'target, 'data>
            where
                V: AsRef<[Value<'value, 'data>]>,
            {
                JuliaFuture::new_local(frame, self, args).await
            }

            unsafe fn schedule_async_local<'target, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'target>,
                args: V,
            ) -> JuliaResult<Task<'target>, 'target, 'data>
            where
                V: AsRef<[Value<'value, 'data>]>,
            {
                let values = args.as_ref();
                let mut vals: SmallVec<[Value; MAX_SIZE]> = SmallVec::with_capacity(1 + values.len());

                vals.push(self);
                vals.extend_from_slice(values);

                let task = Module::main(&frame)
                    .submodule(&frame, "JlrsMultitask")
                    .expect("JlrsMultitask not available")
                    .as_managed()
                    .function(&frame, "scheduleasynclocal")
                    .expect("scheduleasynclocal not available")
                    .as_managed()
                    .call(&mut *frame, &mut vals);

                match task {
                    Ok(t) => Ok(t.cast_unchecked::<Task>()),
                    Err(e) => Err(e),
                }
            }

            async unsafe fn call_async_main<'target, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'target>,
                args: V,
            ) -> JuliaResult<'target, 'data>
            where
                V: AsRef<[Value<'value, 'data>]>,
            {
                JuliaFuture::new_main(frame, self, args).await
            }

            unsafe fn schedule_async_main<'target, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'target>,
                args: V,
            ) -> JuliaResult<Task<'target>, 'target, 'data>
            where
                V: AsRef<[Value<'value, 'data>]>,
            {
                let values = args.as_ref();
                let mut vals: SmallVec<[Value; MAX_SIZE]> = SmallVec::with_capacity(1 + values.len());

                vals.push(self);
                vals.extend_from_slice(values);

                let task = Module::main(&frame)
                    .submodule(&frame, "JlrsMultitask")
                    .expect("JlrsMultitask not available")
                    .as_managed()
                    .function(&frame, "scheduleasync")
                    .expect("scheduleasync not available")
                    .as_managed()
                    .call(&mut *frame, &mut vals);

                match task {
                    Ok(t) => Ok(t.cast_unchecked::<Task>()),
                    Err(e) => Err(e),
                }
            }
        }

        #[async_trait(?Send)]
        impl<'data> CallAsync<'data> for Function<'_, 'data> {
            async unsafe fn call_async<'target, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'target>,
                args: V,
            ) -> JuliaResult<'target, 'data>
            where
                V: AsRef<[Value<'value, 'data>]>,
            {
                JuliaFuture::new(frame, self.as_value(), args).await
            }

            #[cfg(any(feature = "julia-1-10", feature = "julia-1-9"))]
            async unsafe fn call_async_interactive<'target, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'target>,
                args: V,
            ) -> JuliaResult<'target, 'data>
            where
                V: AsRef<[Value<'value, 'data>]>,
            {
                JuliaFuture::new_interactive(frame, self.as_value(), args).await
            }

            #[cfg(any(feature = "julia-1-10", feature = "julia-1-9"))]
            unsafe fn schedule_async_interactive<'target, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'target>,
                args: V,
            ) -> JuliaResult<Task<'target>, 'target, 'data>
            where
                V: AsRef<[Value<'value, 'data>]>,
            {
                self.as_value().schedule_async_interactive(frame, args)
            }

            unsafe fn schedule_async<'target, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'target>,
                args: V,
            ) -> JuliaResult<Task<'target>, 'target, 'data>
            where
                V: AsRef<[Value<'value, 'data>]>,
            {
                self.as_value().schedule_async(frame, args)
            }

            async unsafe fn call_async_local<'target, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'target>,
                args: V,
            ) -> JuliaResult<'target, 'data>
            where
                V: AsRef<[Value<'value, 'data>]>,
            {
                JuliaFuture::new_local(frame, self.as_value(), args).await
            }

            unsafe fn schedule_async_local<'target, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'target>,
                args: V,
            ) -> JuliaResult<Task<'target>, 'target, 'data>
            where
                V: AsRef<[Value<'value, 'data>]>,
            {
                self.as_value().schedule_async_local(frame, args)
            }

            async unsafe fn call_async_main<'target, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'target>,
                args: V,
            ) -> JuliaResult<'target, 'data>
            where
                V: AsRef<[Value<'value, 'data>]>,
            {
                JuliaFuture::new_main(frame, self.as_value(), args).await
            }

            unsafe fn schedule_async_main<'target, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'target>,
                args: V,
            ) -> JuliaResult<Task<'target>, 'target, 'data>
            where
                V: AsRef<[Value<'value, 'data>]>,
            {
                self.as_value().schedule_async_main(frame, args)
            }
        }

        #[async_trait(?Send)]
        impl<'data> CallAsync<'data> for WithKeywords<'_, 'data> {
            async unsafe fn call_async<'target, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'target>,
                args: V,
            ) -> JuliaResult<'target, 'data>
            where
                V: AsRef<[Value<'value, 'data>]>,
            {
                JuliaFuture::new_with_keywords(frame, self, args).await
            }

            #[cfg(any(feature = "julia-1-10", feature = "julia-1-9"))]
            async unsafe fn call_async_interactive<'target, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'target>,
                args: V,
            ) -> JuliaResult<'target, 'data>
            where
                V: AsRef<[Value<'value, 'data>]>,
            {
                JuliaFuture::new_interactive_with_keywords(frame, self, args).await
            }

            #[cfg(any(feature = "julia-1-10", feature = "julia-1-9"))]
            unsafe fn schedule_async_interactive<'target, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'target>,
                args: V,
            ) -> JuliaResult<Task<'target>, 'target, 'data>
            where
                V: AsRef<[Value<'value, 'data>]>,
            {
                let values = args.as_ref();
                let mut vals: SmallVec<[Value; MAX_SIZE]> = SmallVec::with_capacity(1 + values.len());

                vals.push(self.function());
                vals.extend_from_slice(values);

                let task = Module::main(&frame)
                    .submodule(&frame, "JlrsMultitask")
                    .expect("JlrsMultitask not available")
                    .as_managed()
                    .function(&frame, "interactivecall")
                    .expect("interactivecall not available")
                    .as_managed()
                    .provide_keywords(self.keywords())
                    .expect("Keywords invalid")
                    .call(&mut *frame, &mut vals);

                match task {
                    Ok(t) => Ok(t.cast_unchecked::<Task>()),
                    Err(e) => Err(e),
                }
            }

            unsafe fn schedule_async<'target, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'target>,
                args: V,
            ) -> JuliaResult<Task<'target>, 'target, 'data>
            where
                V: AsRef<[Value<'value, 'data>]>,
            {
                let values = args.as_ref();
                let mut vals: SmallVec<[Value; MAX_SIZE]> = SmallVec::with_capacity(1 + values.len());

                vals.push(self.function());
                vals.extend_from_slice(values);

                let task = Module::main(&frame)
                    .submodule(&frame, "JlrsMultitask")
                    .expect("JlrsMultitask not available")
                    .as_managed()
                    .function(&frame, "asynccall")
                    .expect("asynccall not available")
                    .as_managed()
                    .provide_keywords(self.keywords())
                    .expect("Keywords invalid")
                    .call(&mut *frame, &mut vals);

                match task {
                    Ok(t) => Ok(t.cast_unchecked::<Task>()),
                    Err(e) => Err(e),
                }
            }

            async unsafe fn call_async_local<'target, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'target>,
                args: V,
            ) -> JuliaResult<'target, 'data>
            where
                V: AsRef<[Value<'value, 'data>]>,
            {
                JuliaFuture::new_local_with_keywords(frame, self, args).await
            }

            unsafe fn schedule_async_local<'target, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'target>,
                args: V,
            ) -> JuliaResult<Task<'target>, 'target, 'data>
            where
                V: AsRef<[Value<'value, 'data>]>,
            {
                let values = args.as_ref();
                let mut vals: SmallVec<[Value; MAX_SIZE]> = SmallVec::with_capacity(1 + values.len());

                vals.push(self.function());
                vals.extend_from_slice(values);

                let task = Module::main(&frame)
                    .submodule(&frame, "JlrsMultitask")
                    .expect("JlrsMultitask not available")
                    .as_managed()
                    .function(&frame, "scheduleasynclocal")
                    .expect("scheduleasynclocal not available")
                    .as_managed()
                    .provide_keywords(self.keywords())
                    .expect("Keywords invalid")
                    .call(&mut *frame, &mut vals);

                match task {
                    Ok(t) => Ok(t.cast_unchecked::<Task>()),
                    Err(e) => Err(e),
                }
            }

            async unsafe fn call_async_main<'target, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'target>,
                args: V,
            ) -> JuliaResult<'target, 'data>
            where
                V: AsRef<[Value<'value, 'data>]>,
            {
                JuliaFuture::new_main_with_keywords(frame, self, args).await
            }

            unsafe fn schedule_async_main<'target, 'value, V>(
                self,
                frame: &mut AsyncGcFrame<'target>,
                args: V,
            ) -> JuliaResult<Task<'target>, 'target, 'data>
            where
                V: AsRef<[Value<'value, 'data>]>,
            {
                let values = args.as_ref();
                let mut vals: SmallVec<[Value; MAX_SIZE]> = SmallVec::with_capacity(1 + values.len());

                vals.push(self.function());
                vals.extend_from_slice(values);

                let task = Module::main(&frame)
                    .submodule(&frame, "JlrsMultitask")
                    .expect("JlrsMultitask not available")
                    .as_managed()
                    .function(&frame, "scheduleasync")
                    .expect("scheduleasync not available")
                    .as_managed()
                    .provide_keywords(self.keywords())
                    .expect("Keywords invalid")
                    .call(&mut *frame, &mut vals);

                match task {
                    Ok(t) => Ok(t.cast_unchecked::<Task>()),
                    Err(e) => Err(e),
                }
            }
        }
    }
}

mod private {
    use super::WithKeywords;
    #[cfg(all(not(feature = "julia-1-6"), feature = "internal-types"))]
    use crate::data::managed::internal::opaque_closure::OpaqueClosure;
    use crate::data::managed::{function::Function, value::Value};
    pub trait CallPriv: Sized {}
    impl CallPriv for WithKeywords<'_, '_> {}
    impl CallPriv for Function<'_, '_> {}
    #[cfg(all(not(feature = "julia-1-6"), feature = "internal-types"))]
    impl CallPriv for OpaqueClosure<'_> {}
    impl CallPriv for Value<'_, '_> {}
}
