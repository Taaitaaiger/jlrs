//! Non-blocking tasks.
//!
//! In addition to blocking tasks, the async runtime supports non-blocking tasks: tasks that can
//! be called once implement [`AsyncTask`], tasks that can be called multiple times implement
//! [`PersistentTask`].
//!
//! Both of these traits require that you implement one or more async methods. These methods take
//! a mutable reference to an [`AsyncGcFrame`]. This frame type provides the same functionality as
//! `GcFrame`, and can be used in combination with several async methods. Most importantly, the
//! methods of the trait [`CallAsync`] which let you schedule a Julia function call as a new Julia
//! task and await its completion.
//!
//! [`GcFrame`]: crate::memory::frame::GcFrame
//! [`CallAsync`]: crate::call::CallAsync

use crate::error::JlrsResult;
use crate::memory::frame::AsyncGcFrame;
use crate::memory::global::Global;
use async_trait::async_trait;
use jl_sys::{jl_process_events, jl_yield};

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
///         global: Global<'base>,
///         frame: &mut AsyncGcFrame<'base>,
///     ) -> JlrsResult<Self::Output> {
///         let a = Value::new(&mut *frame, self.a)?;
///         let b = Value::new(&mut *frame, self.b)?;
///             
///         let func = Module::base(global).function(&mut *frame, "+")?;
///         unsafe { func.call_async(&mut *frame, &mut [a, b]) }
///             .await?
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
    type Output: 'static + Send + Sync;

    /// The minimum capacity of the `AsyncGcFrame` provided to `run`.
    const RUN_CAPACITY: usize = 0;

    /// The minimum capacity of the `AsyncGcFrame` provided to `register`.
    const REGISTER_CAPACITY: usize = 0;

    /// Register the task.
    ///
    /// Note that this method is not called automatically, but only if
    /// [`AsyncJulia::register_task`] or [`AsyncJulia::try_register_task`] is used. This method
    /// can be implemented to take care of everything required to execute the task successfully,
    /// like loading packages.
    ///
    /// [`AsyncJulia::register_task`]: crate::runtime::async_rt::AsyncJulia::register_task
    /// [`AsyncJulia::try_register_task`]: crate::runtime::async_rt::AsyncJulia::try_register_task
    async fn register<'frame>(
        _global: Global<'frame>,
        _frame: &mut AsyncGcFrame<'frame>,
    ) -> JlrsResult<()> {
        Ok(())
    }

    /// Run this task.
    ///
    /// This method takes a `Global` and a mutable reference to an `AsyncGcFrame`, which lets you
    /// interact with Julia.
    async fn run<'frame>(
        &mut self,
        global: Global<'frame>,
        frame: &mut AsyncGcFrame<'frame>,
    ) -> JlrsResult<Self::Output>;
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
///         _global: Global<'static>,
///         frame: &mut AsyncGcFrame<'static>,
///     ) -> JlrsResult<Self::State> {
///         // A `Vec` can be moved from Rust to Julia if the element type
///         // implements `IntoJulia`.
///         let data = vec![0usize; self.n_values];
///         let array = TypedArray::from_vec(&mut *frame, data, self.n_values)?
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
///         global: Global<'frame>,
///         frame: &mut AsyncGcFrame<'frame>,
///         state: &mut Self::State,
///         input: Self::Input,
///     ) -> JlrsResult<Self::Output> {
///         {
///             // Array data can be directly accessed from Rust.
///             // TypedArray::bits_data_mut can be used if the type
///             // of the elements is concrete and immutable.
///             // This is safe because this is the only active reference to
///             // the array.
///             let mut data = unsafe { state.array.bits_data_mut(frame)? };
///             data[state.offset] = input;
///
///             state.offset += 1;
///             if (state.offset == self.n_values) {
///                 state.offset = 0;
///             }
///         }
///
///         // Return the sum of the contents of `state.array`.
///         unsafe {
///             Module::base(global)
///                 .function(&mut *frame, "sum")?
///                 .call1(&mut *frame, state.array.as_value())?
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

    /// The capacity of the channel the [`PersistentHandle`] uses to communicate with this
    /// persistent.
    ///
    /// If it's set to 0, the channel is unbounded.
    ///
    /// [`PersistentHandle`]: crate::runtime::async_rt::PersistentHandle
    const CHANNEL_CAPACITY: usize = 0;

    /// TThe minimum capacity of the `AsyncGcFrame` provided to `register`.
    const REGISTER_CAPACITY: usize = 0;

    /// The minimum capacity of the `AsyncGcFrame` provided to `init`.
    const INIT_CAPACITY: usize = 0;

    /// The minimum capacity of the `AsyncGcFrame` provided to `run`.
    const RUN_CAPACITY: usize = 0;

    /// Register this persistent task.
    ///
    /// Note that this method is not called automatically, but only if
    /// [`AsyncJulia::register_persistent`] or [`AsyncJulia::try_register_persistent`] is used.
    /// This method can be implemented to take care of everything required to execute the task
    /// successfully, like loading packages.
    ///
    /// [`AsyncJulia::register_persistent`]: crate::runtime::async_rt::AsyncJulia::register_persistent
    /// [`AsyncJulia::try_register_persistent`]: crate::runtime::async_rt::AsyncJulia::try_register_persistent
    async fn register<'frame>(
        _global: Global<'frame>,
        _frame: &mut AsyncGcFrame<'frame>,
    ) -> JlrsResult<()> {
        Ok(())
    }

    /// Initialize the task.
    ///
    /// You can interact with Julia inside this method, the frame is not dropped until the task
    /// itself is dropped. This means that `State` can contain arbitrary Julia data rooted in this
    /// frame. This data is provided to every call to `run`.
    async fn init(
        &mut self,
        global: Global<'static>,
        frame: &mut AsyncGcFrame<'static>,
    ) -> JlrsResult<Self::State>;

    /// Run the task.
    ///
    /// This method takes a `Global` and a mutable reference to an `AsyncGcFrame`, which lets you
    /// interact with Julia. It's also provided with a mutable reference to its `state` and the
    /// `input` provided by the caller. While the state is mutable, it's not possible to allocate
    /// a new Julia value in `run` and assign it to the state because the frame doesn't live long
    /// enough.
    async fn run<'frame>(
        &mut self,
        global: Global<'frame>,
        frame: &mut AsyncGcFrame<'frame>,
        state: &mut Self::State,
        input: Self::Input,
    ) -> JlrsResult<Self::Output>;

    /// Method that is called when all handles to the task have been dropped.
    ///
    /// This method is called with the same frame as `init`.
    async fn exit(
        &mut self,
        _global: Global<'static>,
        _frame: &mut AsyncGcFrame<'static>,
        _state: &mut Self::State,
    ) {
    }
}

/// Yield the root task.
pub fn yield_task(_: &mut AsyncGcFrame) {
    unsafe {
        jl_process_events();
        jl_yield();
    }
}
