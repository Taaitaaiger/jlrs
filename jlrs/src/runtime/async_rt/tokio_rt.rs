//! An implementation of [`AsyncRuntime`] for tokio.
//!
//! When tokio is used as a backing runtime, the following implementations of the [`Channel`]
//! trait are provided:
//!
//!   - [`BoundedChannel`]
//!   - [`UnboundedChannel`]
//!
//! The first of these is backed by the mpsc `Sender` and `Receiver` from tokio, the second by the
//! `UnboundedSender` and `UnboundedReceiver` from the same module.
//!
//! All sending halves of channels provided by tokio, that is the oneshot, mpsc, broadcast, and
//! watch `Sender`s, implement the [`OneshotSender`] trait.

use std::{future::Future, num::NonZeroUsize, time::Duration};

use async_trait::async_trait;
use tokio::{
    runtime::Builder,
    sync::mpsc::{
        Receiver as BoundedReceiver, Sender as BoundedSender, UnboundedReceiver, UnboundedSender,
    },
    task::{JoinError, JoinHandle, LocalSet},
};

use crate::{
    async_util::channel::{
        Channel, ChannelReceiver, ChannelSender, OneshotSender, SendError, TrySendError,
    },
    error::{JlrsError, JlrsResult},
    runtime::async_rt::{AsyncRuntime, Message},
};

/// Struct for which [`AsyncRuntime`] is implemented using tokio.
pub struct Tokio;

#[async_trait(?Send)]
impl AsyncRuntime for Tokio {
    type JoinError = JoinError;
    type TaskOutput = Result<(), Self::JoinError>;
    type RuntimeOutput = Result<JlrsResult<()>, Self::JoinError>;
    type JoinHandle = JoinHandle<()>;
    type RuntimeHandle = JoinHandle<JlrsResult<()>>;

    fn spawn_blocking<F>(rt_fn: F) -> Self::RuntimeHandle
    where
        F: FnOnce() -> JlrsResult<()> + Send + 'static,
    {
        tokio::task::spawn_blocking(rt_fn)
    }

    fn block_on<F>(loop_fn: F, worker_id: Option<usize>) -> JlrsResult<()>
    where
        F: Future<Output = JlrsResult<()>>,
    {
        let thread_name = if let Some(id) = worker_id {
            format!("jlrs-tokio-worker-{}", id)
        } else {
            "jlrs-tokio-runtime".into()
        };

        let runtime = Builder::new_current_thread()
            .thread_name(thread_name)
            .enable_time()
            .build()
            .expect("Unable to build tokio runtime");

        let local_set = LocalSet::new();
        local_set.block_on(&runtime, loop_fn)
    }

    fn spawn_local<F>(future: F) -> Self::JoinHandle
    where
        F: Future<Output = ()> + 'static,
    {
        tokio::task::spawn_local(future)
    }

    async fn yield_now() {
        tokio::task::yield_now().await
    }

    async fn timeout<F>(duration: Duration, future: F) -> Option<JlrsResult<Message>>
    where
        F: Future<Output = JlrsResult<Message>>,
    {
        tokio::time::timeout(duration, future).await.ok()
    }
}

impl<M: Send + 'static> Channel<M> for BoundedChannel<M> {
    type Sender = tokio::sync::mpsc::Sender<M>;
    type Receiver = tokio::sync::mpsc::Receiver<M>;

    fn channel(capacity: Option<NonZeroUsize>) -> (Self::Sender, Self::Receiver) {
        tokio::sync::mpsc::channel(capacity.map(|c| c.get()).unwrap_or_default())
    }
}

impl<M: Send + 'static> Channel<M> for UnboundedChannel<M> {
    type Sender = tokio::sync::mpsc::UnboundedSender<M>;
    type Receiver = tokio::sync::mpsc::UnboundedReceiver<M>;

    fn channel(_: Option<NonZeroUsize>) -> (Self::Sender, Self::Receiver) {
        tokio::sync::mpsc::unbounded_channel()
    }
}

#[async_trait]
impl<M: Send + 'static> ChannelSender<M> for tokio::sync::mpsc::Sender<M> {
    async fn send(&self, msg: M) -> Result<(), SendError<M>> {
        Ok((&*self).send(msg).await.map_err(|e| SendError(e.0))?)
    }

    fn try_send(&self, msg: M) -> Result<(), TrySendError<M>> {
        Ok((&*self).try_send(msg).map_err(|e| match e {
            tokio::sync::mpsc::error::TrySendError::Closed(v) => TrySendError::Closed(v),
            tokio::sync::mpsc::error::TrySendError::Full(v) => TrySendError::Full(v),
        })?)
    }
}

#[async_trait]
impl<M: Send + 'static> ChannelReceiver<M> for tokio::sync::mpsc::Receiver<M> {
    async fn recv(&mut self) -> JlrsResult<M> {
        match self.recv().await {
            Some(m) => Ok(m),
            None => JlrsError::exception_error("Channel was closed".into())?,
        }
    }
}

#[async_trait]
impl<M: Send + 'static> ChannelSender<M> for tokio::sync::mpsc::UnboundedSender<M> {
    async fn send(&self, msg: M) -> Result<(), SendError<M>> {
        Ok((&*self).send(msg).map_err(|e| SendError(e.0))?)
    }

    fn try_send(&self, msg: M) -> Result<(), TrySendError<M>> {
        Ok((&*self).send(msg).map_err(|e| TrySendError::Closed(e.0))?)
    }
}

#[async_trait]
impl<M: Send + 'static> ChannelReceiver<M> for tokio::sync::mpsc::UnboundedReceiver<M> {
    async fn recv(&mut self) -> JlrsResult<M> {
        match self.recv().await {
            Some(m) => Ok(m),
            None => JlrsError::exception_error("Channel was closed".into())?,
        }
    }
}

impl<M: Send + 'static> OneshotSender<M> for tokio::sync::oneshot::Sender<M> {
    fn send(self, msg: M) {
        self.send(msg).ok();
    }
}

impl<M: Send + 'static> OneshotSender<M> for tokio::sync::mpsc::Sender<M> {
    fn send(self, msg: M) {
        (&self).blocking_send(msg).ok();
    }
}

impl<M: Send + 'static> OneshotSender<M> for tokio::sync::broadcast::Sender<M> {
    fn send(self, msg: M) {
        (&self).send(msg).ok();
    }
}

impl<M: Send + Sync + 'static> OneshotSender<M> for tokio::sync::watch::Sender<M> {
    fn send(self, msg: M) {
        (&self).send(msg).ok();
    }
}

/// A bounded channel.
pub type BoundedChannel<M> = (BoundedSender<M>, BoundedReceiver<M>);
/// An unbounded channel.
pub type UnboundedChannel<M> = (UnboundedSender<M>, UnboundedReceiver<M>);
