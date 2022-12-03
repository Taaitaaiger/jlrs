//! An implementation of `AsyncRuntime` for async-std.
//!
//! When async-std is used as a backing runtime, the [`AsyncStdChannel`] type provides an
//! implementation of the [`Channel`] trait. This channel is backed by the `Sender` and
//! `Receiver` from async-std. The sending half of this channel also implements
//! [`OneshotSender`].

use std::{future::Future, num::NonZeroUsize, time::Duration};

use async_std::{
    channel::{bounded, unbounded, Receiver, Sender},
    task::JoinHandle,
};
use async_trait::async_trait;

use crate::{
    async_util::channel::{
        Channel, ChannelReceiver, ChannelSender, OneshotSender, SendError, TrySendError,
    },
    error::{JlrsError, JlrsResult},
    runtime::async_rt::{AsyncRuntime, Message},
};

/// Struct for which [`AsyncRuntime`] is implemented using async-std.
pub struct AsyncStd;

#[async_trait(?Send)]
impl AsyncRuntime for AsyncStd {
    type JoinError = ();
    type TaskOutput = ();
    type RuntimeOutput = JlrsResult<()>;
    type JoinHandle = JoinHandle<()>;
    type RuntimeHandle = JoinHandle<JlrsResult<()>>;

    fn spawn_blocking<F>(rt_fn: F) -> Self::RuntimeHandle
    where
        F: FnOnce() -> JlrsResult<()> + Send + 'static,
    {
        async_std::task::spawn_blocking(rt_fn)
    }

    fn block_on<F>(loop_fn: F, _: Option<usize>) -> JlrsResult<()>
    where
        F: Future<Output = JlrsResult<()>>,
    {
        async_std::task::block_on(loop_fn)
    }

    async fn yield_now() {
        async_std::task::yield_now().await
    }

    fn spawn_local<F>(future: F) -> Self::JoinHandle
    where
        F: Future<Output = ()> + 'static,
    {
        async_std::task::spawn_local(future)
    }

    async fn timeout<F>(duration: Duration, future: F) -> Option<JlrsResult<Message>>
    where
        F: Future<Output = JlrsResult<Message>>,
    {
        async_std::future::timeout(duration, future).await.ok()
    }
}

impl<M: Send + Sync + 'static> Channel<M> for (Sender<M>, Receiver<M>) {
    type Sender = Sender<M>;
    type Receiver = Receiver<M>;

    fn channel(capacity: Option<NonZeroUsize>) -> (Self::Sender, Self::Receiver) {
        match capacity {
            Some(n) => bounded(n.get()),
            _ => unbounded(),
        }
    }
}

#[async_trait]
impl<M: Send + Sync + 'static> ChannelSender<M> for Sender<M> {
    async fn send(&self, msg: M) -> Result<(), SendError<M>> {
        Ok((&*self).send(msg).await.map_err(|e| SendError(e.0))?)
    }

    fn try_send(&self, msg: M) -> Result<(), TrySendError<M>> {
        Ok((&*self).try_send(msg).map_err(|e| match e {
            async_std::channel::TrySendError::Closed(v) => TrySendError::Closed(v),
            async_std::channel::TrySendError::Full(v) => TrySendError::Full(v),
        })?)
    }
}

#[async_trait]
impl<M: Send + Sync + 'static> ChannelReceiver<M> for Receiver<M> {
    async fn recv(&mut self) -> JlrsResult<M> {
        match (&*self).recv().await {
            Ok(m) => Ok(m),
            Err(_) => JlrsError::exception_error("Channel was closed".into())?,
        }
    }
}

impl<M: Send + Sync + 'static> OneshotSender<M> for Sender<M> {
    fn send(self, msg: M) {
        (&self).send_blocking(msg).ok();
    }
}

/// A channel that uses the [`Sender`] and [`Receiver`] from async-std.
pub type AsyncStdChannel<M> = (Sender<M>, Receiver<M>);
