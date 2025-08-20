//! Traits to implement async and persistent tasks.

use std::{future::Future, time::Duration};

use crate::{
    call::Call,
    inline_static_ref,
    memory::scope::LocalScopeExt,
    prelude::{AsyncGcFrame, JlrsResult, Target, Value},
};

/// Register a task.
///
/// Some tasks require additional code to be loaded. This trait can be impemented to take care of
/// such initialization.
pub trait Register: 'static + Send {
    /// Perform the operations necessary to create new tasks.
    fn register(frame: AsyncGcFrame) -> impl Future<Output = JlrsResult<()>>;
}

/// An async task.
///
/// Any type that implements this trait can be sent to the async runtime where its `run` method
/// will be called. This trait is implemented for all implementations of
/// `AsyncFnOnce(AsyncGcFrame) -> T` as long as the trait bounds are respected.
pub trait AsyncTask: 'static + Send {
    /// The return type of `run`.
    type Output: 'static + Send;

    /// Run this task.
    fn run(self, frame: AsyncGcFrame) -> impl Future<Output = Self::Output>;
}

impl<A, U> AsyncTask for A
where
    A: AsyncFnOnce(AsyncGcFrame) -> U + Send + 'static,
    U: Send + 'static,
{
    type Output = U;

    fn run(self, frame: AsyncGcFrame) -> impl Future<Output = Self::Output> {
        self(frame)
    }
}

/// Persistent task
///
/// Unlike an [`AsyncTask`] which is executed once, a persistent task is initialized and then
/// provides a handle to call `run`. A persistent task has a state, which is returned by `init`,
/// which is provided every time `run` is called in addition to the input data.
pub trait PersistentTask: 'static + Send {
    /// The type of the result which is returned if `init` completes successfully.
    ///
    /// This data is provided to every call of `run`.
    type State<'state>;

    /// The type of the data that must be provided to call this persistent task.
    type Input: 'static + Send;

    /// The return type of `run`.
    type Output: 'static + Send;

    // The capacity of the channel used to communicate with this task.
    const CHANNEL_CAPACITY: usize = 0;

    /// Initialize the task.
    ///
    /// You can interact with Julia inside this method, the frame is not dropped until the task
    /// itself is dropped. This means that `State` can contain arbitrary Julia data rooted in this
    /// frame. This data is provided to every call to `run`.
    fn init<'task>(
        &mut self,
        frame: AsyncGcFrame<'task>,
    ) -> impl Future<Output = JlrsResult<Self::State<'task>>>;

    /// Run the task.
    ///
    /// This method takes an `AsyncGcFrame`, which lets you interact with Julia.
    /// It's also provided with a mutable reference to its `state` and the `input` provided by the
    /// caller. While the state is mutable, it's not possible to allocate a new Julia value in
    /// `run` and assign it to the state because the frame doesn't live long enough.
    fn run<'frame, 'task: 'frame>(
        &mut self,
        frame: AsyncGcFrame<'frame>,
        state: &mut Self::State<'task>,
        input: Self::Input,
    ) -> impl Future<Output = Self::Output>;

    /// Method that is called when all handles to the task have been dropped.
    ///
    /// This method is called with the same frame as `init`.
    fn exit<'task>(
        &mut self,
        _frame: AsyncGcFrame<'task>,
        _state: &mut Self::State<'task>,
    ) -> impl Future<Output = ()> {
        async {}
    }
}

/// Sleep for `duration`.
///
/// This function calls `Base.sleep`. If `duration` is less than 1ms this function may return
/// immediately.
pub fn sleep<'scope, 'data, Tgt: Target<'scope>>(target: &Tgt, duration: Duration) {
    unsafe {
        let millis = duration.as_millis();
        if millis == 0 {
            return;
        }

        // Is rooted when sleep is called.
        target.with_local_scope::<_, 1>(|target, mut frame| {
            let secs = duration.as_millis() as usize as f64 / 1000.;
            let secs = Value::new(&mut frame, secs);

            let func: Value<'_, '_> =
                inline_static_ref!(SLEEP, Value<'static, 'static>, "Base.sleep", target);
            func.call(target, [secs]).expect("sleep threw an exception");
        })
    }
}
