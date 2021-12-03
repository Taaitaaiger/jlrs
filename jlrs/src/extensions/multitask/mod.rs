//! Run Julia in a separate thread and execute tasks in parallel.
//!
//! While the Julia C API can only be used from a single thread, it is possible to schedule
//! multiple Julia `Task`s to run in parallel. This doesn't work nicely with the sync runtime
//! because the garbage collector is unable to run and events aren't handled. The async runtimes
//! initialize Julia on a new thread and return a handle, [`AsyncJulia`], that can be cloned and
//! shared across threads; async events in Julia are handled periodically, which also allows the
//! garbage collector to run.
//!
//! In order to use an async runtime, either the `async-std-rt` or `tokio-rt` feature must be
//! enabled. If the first is enabled async-std is used, while the latter makes use of tokio. Julia
//! must be started with three or more threads by setting the `JULIA_NUM_THREADS` environment
//! variable. If this environment variable is not set it's set to `auto`, which starts Julia with
//! as many threads as the CPU supports.
//!
//! The easiest way to interact with the async runtime is sending a blocking task. A blocking task
//! is executed on the main thread as soon as it's received. Any closure that takes a [`Global`]
//! and a mutable reference to a [`GcFrame`], returns a [`JlrsResult`], and implements `Send` and
//! `Sync` is a valid blocking task. This is essentially the same interface as [`Julia::scope`]
//! provides, the main difference is that the requirements are a bit more strict because the
//! closure and return type must implement `Send` and `Sync`. In order to send the result back, a
//! channel must be provided whenever a new task sent to the runtime. The sending half of this
//! channel must implement the [`ResultSender`] trait, by default this trait is implemented for
//! the `Sender`s from crossbeam-channel and those from the chosen backing runtime.
//!
//! Because blocking tasks are essentially equivalent to using [`Julia::scope`], using blocking
//! threads to schedule new `Task`s doesn't work. It's also not possible to have multiple blocking
//! tasks running at the same time because they block the main thread.
//!
//! In order to write a non-blocking task the [`AsyncTask`] trait must be implemented. This is an
//! async trait with two async methods, `register` and `run`. Only `run` has to be implemented, it
//! takes a mutable reference to an [`AsyncGcFrame`] rather than a `GcFrame`, the major difference
//! between these two frame types is that `AsyncGcFrame` can be used to call the methods provided
//! by the [`CallAsync`] trait.
//!
//! The `CallAsync` trait extends [`Call`]. Its methods let you schedule a Julia function call as
//! a new `Task`, either by using `Base.Thread.@spawn` or `@async` internally. Note that tasks are
//! never scheduled on the main thread, even if `@async` is used this happens on another thread to
//! ensure the main thread isn't blocked. A sync and async variation of each method is available,
//! the async method resolves when the function call has completed. While it's `await`ed the async
//! runtime can handle other tasks. The sync variants simply schedule the function call and return
//! the `Task`.
//!
//! Like a blocking task, an `AsyncTask` runs once and eventually sends back its result through a
//! provided channel. In many cases it's more useful to set up some initial state and interact
//! with this task. For this purpose the [`PersistentTask`] trait can be implemented. It has three
//! async methods: `register`, `init`, and `run`; both `init` and `run` must be implemented. When
//! a `PersistentTask` starts executing `init` is called, which returns the initial state of the
//! persistent. Because the frame provided to this method isn't dropped after it has completed, the
//! initial state can contain Julia data rooted in that frame. After `init` has completed a handle
//! to the task is returned. This handle can be used to call the task's `run` method.
//! This method is similar to [`AsyncTask::run`], except that it's also provided with a mutable
//! reference to the task's state and the additional data that must be provided when calling
//! the task using its handle.
//!
//! Examples that show how to use the async runtime and implement tasks can be found in the
//! [crate-level docs] and [`examples`] directory of the repository.
//!
//! [`examples`]: https://github.com/Taaitaaiger/jlrs/tree/master/examples
//! [`Julia`]: crate::julia::Julia
//! [`Julia::scope`]: crate::julia::Julia::scope
//! [`AsyncGcFrame`]: crate::extensions::multitask::async_frame::AsyncGcFrame
//! [`CallAsync`]: crate::extensions::multitask::call_async::CallAsync
//! [`PersistentTask`]: crate::extensions::multitask::async_task::PersistentTask
//! [`AsyncTask`]: crate::extensions::multitask::async_task::AsyncTask
//! [`AsyncTask::run`]: crate::extensions::multitask::async_task::AsyncTask::run
//! [`Task`]: crate::wrappers::ptr::task::Task
//! [crate-level docs]: crate

use jl_sys::{jl_process_events, jl_yield};

use self::async_frame::AsyncGcFrame;

pub mod async_frame;
pub mod async_task;
pub mod call_async;
pub(crate) mod julia_future;
pub mod mode;
pub(crate) mod output_result_ext;
pub mod result_sender;

#[cfg(any(feature = "async-std-rt", feature = "tokio-rt"))]
pub mod runtime;

/// Yield the current task.
pub fn yield_task(_: &mut AsyncGcFrame) {
    unsafe {
        jl_process_events();
        jl_yield();
    }
}
