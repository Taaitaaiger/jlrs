use std::time::Duration;

use async_trait::async_trait;

use crate::{
    call::Call,
    inline_static_ref,
    prelude::{AsyncGcFrame, JlrsResult, Target, Value},
};

#[async_trait(?Send)]
pub trait Register: 'static + Send {
    async fn register<'frame>(frame: AsyncGcFrame<'frame>) -> JlrsResult<()>;
}

/// Async task
///
/// Any type that implements this trait can be sent to the async runtime where its `run` method
/// will be called. Note that [`async_trait`] is used, all implementers must be marked with
/// `#[async_trait(?Send)]`.
///
/// When the `async-closure` feature has been enabled, this trait is implemented for all
/// `A: AsyncFnMut(AsyncGcFrame) -> T`.
/// 
/// [`async_trait`]: async_trait::async_trait
#[async_trait(?Send)]
pub trait AsyncTask: 'static + Send {
    /// The return type of `run`.
    type Output: 'static + Send;

    /// Run this task.
    async fn run<'frame>(&mut self, frame: AsyncGcFrame<'frame>) -> Self::Output;
}

/// Persistent task
///
/// Unlike an [`AsyncTask`], which is executed once, a persistent task is initialized and then
/// provides a handle to call `run`. A persistent task has a state, which is returned by `init`,
/// which is provided every time `run` is called in addition to the input data.
#[async_trait(?Send)]
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
    async fn init<'frame>(
        &mut self,
        frame: AsyncGcFrame<'frame>,
    ) -> JlrsResult<Self::State<'frame>>;

    /// Run the task.
    ///
    /// This method takes an `AsyncGcFrame`, which lets you interact with Julia.
    /// It's also provided with a mutable reference to its `state` and the `input` provided by the
    /// caller. While the state is mutable, it's not possible to allocate a new Julia value in
    /// `run` and assign it to the state because the frame doesn't live long enough.
    async fn run<'frame, 'state: 'frame>(
        &mut self,
        frame: AsyncGcFrame<'frame>,
        state: &mut Self::State<'state>,
        input: Self::Input,
    ) -> Self::Output;

    /// Method that is called when all handles to the task have been dropped.
    ///
    /// This method is called with the same frame as `init`.
    async fn exit<'frame>(
        &mut self,
        _frame: AsyncGcFrame<'frame>,
        _state: &mut Self::State<'frame>,
    ) {
    }
}

/// Sleep for `duration`.
///
/// The function calls `Base.sleep`. If `duration` is less than 1ms this function returns
/// immediately.
pub fn sleep<'scope, 'data, Tgt: Target<'scope>>(target: &Tgt, duration: Duration) {
    unsafe {
        let millis = duration.as_millis();
        if millis == 0 {
            return;
        }

        let func = inline_static_ref!(FOO, Value<'static, 'static>, "Base.sleep", target);

        // Is rooted when sleep is called.
        target.with_local_scope::<_, _, 1>(|target, mut frame| {
            let secs = duration.as_millis() as usize as f64 / 1000.;
            let secs = Value::new(&mut frame, secs);

            func.call1(target, secs).expect("sleep threw an exception");
        })
    }
}

#[cfg(feature = "async-closure")]
#[async_trait(?Send)]
impl<A, U> AsyncTask for A
where
    for<'scope> A: AsyncFnMut(AsyncGcFrame<'scope>) -> U + Send + 'static,
    U: Send + 'static,
{
    type Output = U;

    async fn run<'frame>(&mut self, frame: AsyncGcFrame<'frame>) -> Self::Output {
        self(frame).await
    }
}
