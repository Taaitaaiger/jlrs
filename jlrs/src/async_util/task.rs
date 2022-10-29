//! Non-blocking tasks.
//!
//! In addition to blocking tasks, the async runtime supports non-blocking tasks: tasks that can
//! be called once implement [`AsyncTask`], tasks that can be called multiple times implement
//! [`PersistentTask`].
//!
//! Both of these traits require that you implement one or more async methods. These methods take
//! an [`AsyncGcFrame`]. This frame type provides the same functionality as `GcFrame`, and can be
//! used in combination with several async methods. Most importantly, the methods of the trait
//! [`CallAsync`] which let you schedule a Julia function call as a new Julia task and await its
//! completion.
//!
//! [`GcFrame`]: crate::memory::frame::GcFrame
//! [`CallAsync`]: crate::call::CallAsync

use std::time::Duration;

use crate::{
    call::Call,
    error::JlrsResult,
    memory::target::{frame::AsyncGcFrame, Target},
    wrappers::ptr::{module::Module, value::Value},
};
use async_trait::async_trait;
use jl_sys::jl_yield;

/// A task that returns once.
///
/// In order to schedule the task you must use [`AsyncJulia::task`] or [`AsyncJulia::try_task`].
///
/// Example:
///
/// ```
/// use jlrs::prelude::*;
///
/// struct AdditionTask {
///     a: u64,
///     b: u32,
/// }
///
/// // Only the runtime thread can call the Julia C API, so the async trait
/// // methods of `AsyncTask` must not return a future that implements `Send`
/// // or `Sync`.
/// #[async_trait(?Send)]
/// impl AsyncTask for AdditionTask {
///     // The type of the result of this task if it succeeds.
///     type Output = u64;
///
///     async fn run<'base>(
///         &mut self,
///         mut frame: AsyncGcFrame<'base>,
///     ) -> JlrsResult<Self::Output> {
///         let a = Value::new(&mut frame, self.a);
///         let b = Value::new(&mut frame, self.b);
///
///         let func = Module::base(&frame).function(&mut frame, "+")?;
///         unsafe { func.call_async(&mut frame, &mut [a, b]) }
///             .await
///             .into_jlrs_result()?
///             .unbox::<u64>()
///     }
/// }
/// ```
///
/// [`AsyncJulia::task`]: crate::runtime::async_rt::AsyncJulia::task
/// [`AsyncJulia::try_task`]: crate::runtime::async_rt::AsyncJulia::try_task
#[async_trait(?Send)]
pub trait AsyncTask: 'static + Send + Sync {
    /// The type of the result which is returned if `run` completes successfully.
    type Output: 'static + Send;

    /// Register the task.
    ///
    /// Note that this method is not called automatically, but only if
    /// [`AsyncJulia::register_task`] or [`AsyncJulia::try_register_task`] is used. This method
    /// can be implemented to take care of everything required to execute the task successfully,
    /// like loading packages.
    ///
    /// [`AsyncJulia::register_task`]: crate::runtime::async_rt::AsyncJulia::register_task
    /// [`AsyncJulia::try_register_task`]: crate::runtime::async_rt::AsyncJulia::try_register_task
    async fn register<'frame>(_frame: AsyncGcFrame<'frame>) -> JlrsResult<()> {
        Ok(())
    }

    /// Run this task.
    ///
    /// See the [trait docs] for an example implementation.
    ///
    /// [trait docs]: AsyncTask
    async fn run<'frame>(&mut self, frame: AsyncGcFrame<'frame>) -> JlrsResult<Self::Output>;
}

/// A task that can be called multiple times.
///
/// In order to schedule the task you must use [`AsyncJulia::persistent`] or
/// [`AsyncJulia::try_persistent`].
///
/// Example:
///
/// ```
/// # #[cfg(not(all(target_os = "windows", feature = "lts")))]
/// # {
/// use jlrs::prelude::*;
///
/// struct AccumulatorTask {
///     n_values: usize
/// }
///
/// struct AccumulatorTaskState {
///     array: TypedArray<'static, 'static, usize>,
///     offset: usize
/// }
///
/// // Only the runtime thread can call the Julia C API, so the async trait
/// // methods of `PersistentTask` must not return a future that implements
/// // `Send` or `Sync`.
/// #[async_trait(?Send)]
/// impl PersistentTask for AccumulatorTask {
///     // The type of the result of the task if it succeeds.
///     type Output = usize;
///     // The type of the task's internal state.
///     type State = AccumulatorTaskState;
///     // The type of the additional data that the task must be called with.
///     type Input = usize;
///
///     // This method is called before the handle is returned. Note that the
///     // lifetime of the frame is `'static`: the frame is not dropped until
///     // the task has completed, so the task's internal state can contain
///     // Julia data rooted in this frame.
///     async fn init(
///         &mut self,
///         mut frame: AsyncGcFrame<'static>,
///     ) -> JlrsResult<Self::State> {
///         // A `Vec` can be moved from Rust to Julia if the element type
///         // implements `IntoJulia`.
///         let data = vec![0usize; self.n_values];
///         let array = TypedArray::from_vec(frame.as_extended_target(), data, self.n_values)?
///             .into_jlrs_result()?;
///
///         Ok(AccumulatorTaskState {
///             array,
///             offset: 0
///         })
///     }
///
///     // Whenever the task is called through its handle this method
///     // is called. Unlike `init`, the frame that this method can use
///     // is dropped after `run` returns.
///     async fn run<'frame>(
///         &mut self,
///         mut frame: AsyncGcFrame<'frame>,
///         state: &mut Self::State,
///         input: Self::Input,
///     ) -> JlrsResult<Self::Output> {
///         {
///             // Array data can be directly accessed from Rust.
///             // The data is tracked first to ensure it's not
///             // already borrowed from Rust.
///             unsafe {
///                 let mut tracked = state.array.track_mut()?;
///                 let mut data = tracked.bits_data_mut()?;
///                 data[state.offset] = input;
///             };
///
///             state.offset += 1;
///             if (state.offset == self.n_values) {
///                 state.offset = 0;
///             }
///         }
///
///         // Return the sum of the contents of `state.array`.
///         unsafe {
///             Module::base(&frame)
///                 .function(&mut frame, "sum")?
///                 .call1(&mut frame, state.array.as_value())
///                 .into_jlrs_result()?
///                 .unbox::<usize>()
///         }
///     }
/// }
/// # }
/// ```
///
/// [`AsyncJulia::persistent`]: crate::runtime::async_rt::AsyncJulia::persistent
/// [`AsyncJulia::try_persistent`]: crate::runtime::async_rt::AsyncJulia::try_persistent
#[async_trait(?Send)]
pub trait PersistentTask: 'static + Send + Sync {
    /// The type of the result which is returned if `init` completes successfully.
    ///
    /// This data is provided to every call of `run`. Because `init` takes a frame with the
    /// `'static` lifetime, this type can contain Julia data.
    type State: 'static;

    /// The type of the data that must be provided when calling this persistent through its
    /// handle.
    type Input: 'static + Send + Sync;

    /// The type of the result which is returned if `run` completes successfully.
    type Output: 'static + Send + Sync;

    // The capacity of the channel used to communicate with this task.
    const CHANNEL_CAPACITY: usize = 0;

    /// Register this persistent task.
    ///
    /// Note that this method is not called automatically, but only if
    /// [`AsyncJulia::register_persistent`] or [`AsyncJulia::try_register_persistent`] is used.
    /// This method can be implemented to take care of everything required to execute the task
    /// successfully, like loading packages.
    ///
    /// [`AsyncJulia::register_persistent`]: crate::runtime::async_rt::AsyncJulia::register_persistent
    /// [`AsyncJulia::try_register_persistent`]: crate::runtime::async_rt::AsyncJulia::try_register_persistent
    async fn register<'frame>(_frame: AsyncGcFrame<'frame>) -> JlrsResult<()> {
        Ok(())
    }

    /// Initialize the task.
    ///
    /// You can interact with Julia inside this method, the frame is not dropped until the task
    /// itself is dropped. This means that `State` can contain arbitrary Julia data rooted in this
    /// frame. This data is provided to every call to `run`.
    async fn init(&mut self, frame: AsyncGcFrame<'static>) -> JlrsResult<Self::State>;

    /// Run the task.
    ///
    /// This method takes an `AsyncGcFrame`, which lets you interact with Julia.
    /// It's also provided with a mutable reference to its `state` and the `input` provided by the
    /// caller. While the state is mutable, it's not possible to allocate a new Julia value in
    /// `run` and assign it to the state because the frame doesn't live long enough.
    ///
    /// See the [trait docs] for an example implementation.
    ///
    /// [trait docs]: PersistentTask
    async fn run<'frame>(
        &mut self,
        frame: AsyncGcFrame<'frame>,
        state: &mut Self::State,
        input: Self::Input,
    ) -> JlrsResult<Self::Output>;

    /// Method that is called when all handles to the task have been dropped.
    ///
    /// This method is called with the same frame as `init`.
    async fn exit(&mut self, _frame: AsyncGcFrame<'static>, _state: &mut Self::State) {}
}

/// Yield the current Julia task.
///
/// Calling this function allows Julia to switch to another Julia task scheduled on the same
/// thread.
pub fn yield_task(_: &mut AsyncGcFrame) {
    // Safety: this function can only be called from a thread known to Julia.
    unsafe {
        jl_yield();
    }
}

/// Sleep for `duration`.
///
/// The function calls `Base.sleep`. If `duration` is less than 1ms this function returns
/// immediately.
pub fn sleep<'scope, 'data, T: Target<'scope, 'data>>(target: T, duration: Duration) {
    unsafe {
        let millis = duration.as_millis();
        if millis == 0 {
            return;
        }

        let global = target.global();
        // Is rooted when sleep is called.
        let secs = duration.as_millis() as usize as f64 / 1000.;
        let secs = Value::new(global, secs).value_unchecked();

        Module::base(&global)
            .global(global, "sleep")
            .expect("sleep not found")
            .value_unchecked()
            .call1(global, secs)
            .expect("sleep threw an exception");
    }
}
