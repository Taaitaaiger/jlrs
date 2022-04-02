//! Non-blocking tasks
//!
//! In addition to blocking tasks, the async runtime supports non-blocking tasks which fall into
//! two categories: tasks that can be called once implement [`AsyncTask`], tasks that can be
//! called multiple times implement [`PersistentTask`].
//!
//! Both of these traits require that you implement one or more async methods. Rather than a
//! mutable reference to a [`GcFrame`] they take a mutable reference to an [`AsyncGcFrame`].
//! This frame type provides the same functionality as `GcFrame`, and can be used in combination
//! with several async methods. Most importantly, the methods of the trait [`CallAsync`] which let
//! you schedule a Julia function call as a new Julia task and await its completion.
//!
//! [`GcFrame`]: crate::memory::frame::GcFrame
//! [`CallAsync`]: crate::multitask::call_async::CallAsync

#[cfg(any(feature = "async-std-rt", feature = "tokio-rt"))]
pub(crate) mod internal;
use super::async_frame::AsyncGcFrame;
use crate::error::JlrsResult;
use crate::memory::global::Global;
use async_trait::async_trait;

/// A task that returns once. In order to schedule the task you must use [`AsyncJulia::task`] or
/// [`AsyncJulia::try_task`].
///
/// [`AsyncJulia::task`]: crate::multitask::runtime::AsyncJulia::task
/// [`AsyncJulia::try_task`]: crate::multitask::runtime::AsyncJulia::try_task
#[async_trait(?Send)]
pub trait AsyncTask: 'static + Send + Sync {
    /// The type of the result which is returned if `run` completes successfully.
    type Output: 'static + Send + Sync;

    /// The minimum capacity of the `AsyncGcFrame` provided to `run`.
    const RUN_SLOTS: usize = 0;

    /// The minimum capacity of the `AsyncGcFrame` provided to `register`.
    const REGISTER_SLOTS: usize = 0;

    /// Register the task. Note that this method is not called automatically, but only if
    /// [`AsyncJulia::register_task`] or [`AsyncJulia::try_register_task`] is used. This method
    /// can be implemented to take care of everything required to execute the task successfully,
    /// like loading packages.
    ///
    /// [`AsyncJulia::register_task`]: crate::multitask::runtime::AsyncJulia::register_task
    /// [`AsyncJulia::try_register_task`]: crate::multitask::runtime::AsyncJulia::try_register_task
    async fn register<'frame>(
        _global: Global<'frame>,
        _frame: &mut AsyncGcFrame<'frame>,
    ) -> JlrsResult<()> {
        Ok(())
    }

    /// Run this task. This method takes a `Global` and a mutable reference to an `AsyncGcFrame`,
    /// which lets you interact with Julia.
    async fn run<'frame>(
        &mut self,
        global: Global<'frame>,
        frame: &mut AsyncGcFrame<'frame>,
    ) -> JlrsResult<Self::Output>;
}

/// A task that can be called multiple times. In order to schedule the task you must use
/// [`AsyncJulia::persistent`] or [`AsyncJulia::try_persistent`].
///
/// [`AsyncJulia::persistent`]: crate::multitask::runtime::AsyncJulia::persistent
/// [`AsyncJulia::try_persistent`]: crate::multitask::runtime::AsyncJulia::try_persistent
#[async_trait(?Send)]
pub trait PersistentTask: 'static + Send + Sync {
    /// The type of the result which is returned if `init` completes successfully. This data is
    /// provided to every call of `run`. Because `init` takes a frame with the `'static` lifetime,
    /// this type can contain Julia data.
    type State: 'static;

    /// The type of the data that must be provided when calling this persistent through its handle.
    type Input: 'static + Send + Sync;

    /// The type of the result which is returned if `run` completes successfully.
    type Output: 'static + Send + Sync;

    /// The capacity of the channel the [`PersistentHandle`] uses to communicate with this
    /// persistent.
    ///
    /// If it's set to 0, the channel is unbounded.
    ///
    /// [`PersistentHandle`]: crate::multitask::runtime::PersistentHandle
    const CHANNEL_CAPACITY: usize = 0;

    /// TThe minimum capacity of the `AsyncGcFrame` provided to `register`.
    const REGISTER_SLOTS: usize = 0;

    /// The minimum capacity of the `AsyncGcFrame` provided to `init`.
    const INIT_SLOTS: usize = 0;

    /// The minimum capacity of the `AsyncGcFrame` provided to `run`.
    const RUN_SLOTS: usize = 0;

    /// The minimum capacity of the `AsyncGcFrame` provided to `exit`.
    const EXIT_SLOTS: usize = 0;

    // NB: `init` and `run` have an explicit 'inner lifetime . If this lifetime is elided
    // `PersistentTask`s can be implemented in bin crates but not in lib crates (rustc 1.54.0)

    /// Register this persistent task. Note that this method is not called automatically, but only
    /// if [`AsyncJulia::register_persistent`] or [`AsyncJulia::try_register_persistent`] is used.
    /// This method can be implemented to take care of everything required to execute the task
    /// successfully, like loading packages.
    ///
    /// [`AsyncJulia::register_persistent`]: crate::multitask::runtime::AsyncJulia::register_persistent
    /// [`AsyncJulia::try_register_persistent`]: crate::multitask::runtime::AsyncJulia::try_register_persistent
    async fn register<'frame>(
        _global: Global<'frame>,
        _frame: &mut AsyncGcFrame<'frame>,
    ) -> JlrsResult<()> {
        Ok(())
    }

    /// Initialize the task. You can interact with Julia inside this method, the frame is
    /// not dropped until the task itself is dropped. This means that `State` can contain
    /// arbitrary Julia data rooted in this frame. This data is provided to every call to `run`.
    async fn init<'inner>(
        &'inner mut self,
        global: Global<'static>,
        frame: &'inner mut AsyncGcFrame<'static>,
    ) -> JlrsResult<Self::State>;

    /// Run the task. This method takes a `Global` and a mutable reference to an
    /// `AsyncGcFrame`, which lets you interact with Julia. It's also provided with a mutable
    /// reference to its `state` and the `input` provided by the caller. While the state is
    /// mutable, it's not possible to allocate a new Julia value in `run` and assign it to the
    /// state because the frame doesn't live long enough.
    async fn run<'inner, 'frame>(
        &'inner mut self,
        global: Global<'frame>,
        frame: &'inner mut AsyncGcFrame<'frame>,
        state: &'inner mut Self::State,
        input: Self::Input,
    ) -> JlrsResult<Self::Output>;

    /// Method that is called when all handles to the task have been dropped. It's called with the
    /// same frame as `init`.
    async fn exit<'inner>(
        &'inner mut self,
        _global: Global<'static>,
        _frame: &'inner mut AsyncGcFrame<'static>,
        _state: &'inner mut Self::State,
    ) {
    }
}
