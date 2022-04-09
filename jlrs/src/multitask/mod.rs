//! Run Julia in a separate thread and execute multiple tasks in parallel.
//!
//! While the Julia C API can only be used from a single thread, it is possible to schedule
//! multiple Julia `Task`s to run in parallel. This doesn't work nicely with the sync runtime
//! because this runtime is not aware of any potential tasks running in the background. Two
//! async runtimes are available, one backed by tokio and the other on async-std.
//!
//! In order to use an async runtime, either the `async-std-rt` or `tokio-rt` feature must be
//! enabled. If the first is enabled async-std is used, while the latter makes use of tokio. Julia
//! must be started with three or more threads by setting the `JULIA_NUM_THREADS` environment
//! variable. If you're writing a library that provides implementations of the task traits that
//! are used by the async runtime you must enable the `async` feature and not choose a runtime.
//!
//! The easiest way to interact with the async runtime is sending a blocking task. A blocking task
//! is executed on the main Julia thread as soon as it's received. Any closure that takes a
//! [`Global`] and a mutable reference to a [`GcFrame`], returns a [`JlrsResult`], and implements
//! `Send` and `Sync` is a valid blocking task. This is essentially the same interface as
//! [`Julia::scope`] provides, the main difference is that the requirements are a bit more strict
//! because the closure and return type must implement `Send` and `Sync`. In order to send the
//! result back, a channel must be provided whenever a new task sent to the runtime. The sending
//! half of this channel must implement the [`ResultSender`] trait, by default this trait is
//! implemented for the `Sender`s from crossbeam-channel and those from the chosen backing
//! runtime.
//!
//! Because blocking tasks are essentially equivalent to using the sync runtime, using blocking
//! tasks to schedule new `Task`s doesn't work. It's also not possible to have multiple blocking
//! tasks running at the same time because they block the main thread. In order to write tasks
//! that can be run in parallel two traits are available, [`AsyncTask`] and [`PersistentTask`].
//! More information about them can be found in the documentation for the [`async_task`] module.
//!
//! Examples that show how to use the async runtime and implement tasks can be found in the
//! [crate-level docs] and [`examples`] directory of the repository.
//!
//! [`examples`]: https://github.com/Taaitaaiger/jlrs/tree/master/examples
//! [`Global`]: crate::memory::global::Global
//! [`GcFrame`]: crate::memory::frame::GcFrame
//! [`JlrsResult`]: crate::error::JlrsResult
//! [`Call`]: crate::wrappers::ptr::call::Call
//! [`ResultSender`]: crate::multitask::result_sender::ResultSender
//! [`Julia::scope`]: crate::julia::Julia::scope
//! [`AsyncGcFrame`]: crate::multitask::async_frame::AsyncGcFrame
//! [`CallAsync`]: crate::multitask::call_async::CallAsync
//! [`PersistentTask`]: crate::multitask::async_task::PersistentTask
//! [`AsyncTask`]: crate::multitask::async_task::AsyncTask
//! [`AsyncJulia`]: crate::multitask::runtime::AsyncJulia
//! [`AsyncTask::run`]: crate::multitask::async_task::AsyncTask::run
//! [`Task`]: crate::wrappers::ptr::task::Task
//! [crate-level docs]: crate

use jl_sys::{jl_process_events, jl_yield};

use self::async_frame::AsyncGcFrame;

pub mod async_frame;
pub mod async_task;
pub mod call_async;
pub(crate) mod julia_future;
pub mod mode;
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
