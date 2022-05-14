//! An implementation of [`AsyncRuntime`] for tokio.
//!
//! When tokio is used as a backing runtime, the following implementations of the [`Channel`]
//! trait are provided:
//!
//!   - [`BoundedChannel`]
//!   - [`UnboundedChannel`]
//!   - [`MaybeUnboundedChannel`]
//!
//! The first of these is backed by the mpsc `Sender` and `Receiver` from tokio, the second by the
//! `UnboundedSender` and `UnboundedReceiver` from the same module. The last one is backed by the
//! `MaybeUnboundedSender` and `MaybeUnboundedReceiver` defined in this module.
//!
//! All sending halves of channels provided by tokio, that is the oneshot, mpsc, broadcast, and
//! watch `Sender`s, implement the [`OneshotSender`] trait.

use crate::{
    async_util::channel::{
        Channel, ChannelReceiver, ChannelSender, OneshotSender, SendError, TrySendError,
    },
    error::{JlrsError, JlrsResult},
    runtime::async_rt::{AsyncRuntime, Message},
};
use async_trait::async_trait;
use std::{future::Future, num::NonZeroUsize, time::Duration};
use tokio::{
    runtime::Builder,
    sync::mpsc::{
        Receiver as BoundedReceiver, Sender as BoundedSender, UnboundedReceiver, UnboundedSender,
    },
    task::{JoinError, JoinHandle, LocalSet},
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
        F: FnOnce() -> JlrsResult<()> + Send + Sync + 'static,
    {
        tokio::task::spawn_blocking(rt_fn)
    }

    fn block_on<F>(loop_fn: F) -> JlrsResult<()>
    where
        F: Future<Output = JlrsResult<()>>,
    {
        let runtime = Builder::new_current_thread()
            .thread_name("jlrs-tokio-runtime")
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

    async fn timeout<F>(duration: Duration, future: F) -> Option<JlrsResult<Message>>
    where
        F: Future<Output = JlrsResult<Message>>,
    {
        tokio::time::timeout(duration, future).await.ok()
    }
}

/// Either [`tokio::sync::mpsc::Sender`] or [`tokio::sync::mpsc::UnboundedSender`].
pub enum MaybeUnboundedSender<M> {
    Bounded(tokio::sync::mpsc::Sender<M>),
    Unbounded(tokio::sync::mpsc::UnboundedSender<M>),
}

/// Either [`tokio::sync::mpsc::Receiver`] or [`tokio::sync::mpsc::UnboundedReceiver`].
pub enum MaybeUnboundedReceiver<M> {
    Bounded(tokio::sync::mpsc::Receiver<M>),
    Unbounded(tokio::sync::mpsc::UnboundedReceiver<M>),
}

/// Create a new channel that is unbounded if the capacity is `None` and bounded otherwise.
pub fn maybe_unbounded_channel<M: Send + Sync + 'static>(
    capacity: Option<NonZeroUsize>,
) -> (MaybeUnboundedSender<M>, MaybeUnboundedReceiver<M>) {
    match capacity {
        Some(n) => {
            let (s, r) = tokio::sync::mpsc::channel(n.get());
            (
                MaybeUnboundedSender::Bounded(s),
                MaybeUnboundedReceiver::Bounded(r),
            )
        }
        None => {
            let (s, r) = tokio::sync::mpsc::unbounded_channel();
            (
                MaybeUnboundedSender::Unbounded(s),
                MaybeUnboundedReceiver::Unbounded(r),
            )
        }
    }
}

impl<M: Send + Sync + 'static> Channel<M> for BoundedChannel<M> {
    type Sender = tokio::sync::mpsc::Sender<M>;
    type Receiver = tokio::sync::mpsc::Receiver<M>;

    fn channel(capacity: Option<NonZeroUsize>) -> (Self::Sender, Self::Receiver) {
        tokio::sync::mpsc::channel(capacity.map(|c| c.get()).unwrap_or_default())
    }
}

impl<M: Send + Sync + 'static> Channel<M> for UnboundedChannel<M> {
    type Sender = tokio::sync::mpsc::UnboundedSender<M>;
    type Receiver = tokio::sync::mpsc::UnboundedReceiver<M>;

    fn channel(_: Option<NonZeroUsize>) -> (Self::Sender, Self::Receiver) {
        tokio::sync::mpsc::unbounded_channel()
    }
}

impl<M: Send + Sync + 'static> Channel<M> for MaybeUnboundedChannel<M> {
    type Sender = MaybeUnboundedSender<M>;
    type Receiver = MaybeUnboundedReceiver<M>;

    fn channel(capacity: Option<NonZeroUsize>) -> (Self::Sender, Self::Receiver) {
        maybe_unbounded_channel(capacity)
    }
}

#[async_trait]
impl<M: Send + Sync + 'static> ChannelSender<M> for tokio::sync::mpsc::Sender<M> {
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
impl<M: Send + Sync + 'static> ChannelReceiver<M> for tokio::sync::mpsc::Receiver<M> {
    async fn recv(&mut self) -> JlrsResult<M> {
        match self.recv().await {
            Some(m) => Ok(m),
            None => Err(JlrsError::Exception {
                msg: String::from("Channel was closed"),
            })?,
        }
    }
}

#[async_trait]
impl<M: Send + Sync + 'static> ChannelSender<M> for tokio::sync::mpsc::UnboundedSender<M> {
    async fn send(&self, msg: M) -> Result<(), SendError<M>> {
        Ok((&*self).send(msg).map_err(|e| SendError(e.0))?)
    }

    fn try_send(&self, msg: M) -> Result<(), TrySendError<M>> {
        Ok((&*self).send(msg).map_err(|e| TrySendError::Closed(e.0))?)
    }
}

#[async_trait]
impl<M: Send + Sync + 'static> ChannelReceiver<M> for tokio::sync::mpsc::UnboundedReceiver<M> {
    async fn recv(&mut self) -> JlrsResult<M> {
        match self.recv().await {
            Some(m) => Ok(m),
            None => Err(JlrsError::Exception {
                msg: String::from("Channel was closed"),
            })?,
        }
    }
}

#[async_trait]
impl<M: Send + Sync + 'static> ChannelSender<M> for MaybeUnboundedSender<M> {
    async fn send(&self, msg: M) -> Result<(), SendError<M>> {
        match self {
            MaybeUnboundedSender::Bounded(ref s) => ChannelSender::send(s, msg).await,
            MaybeUnboundedSender::Unbounded(ref s) => ChannelSender::send(s, msg).await,
        }
    }

    fn try_send(&self, msg: M) -> Result<(), TrySendError<M>> {
        match self {
            MaybeUnboundedSender::Bounded(ref s) => ChannelSender::try_send(s, msg),
            MaybeUnboundedSender::Unbounded(ref s) => ChannelSender::try_send(s, msg),
        }
    }
}

#[async_trait]
impl<M: Send + Sync + 'static> ChannelReceiver<M> for MaybeUnboundedReceiver<M> {
    async fn recv(&mut self) -> JlrsResult<M> {
        match self {
            MaybeUnboundedReceiver::Bounded(ref mut r) => ChannelReceiver::recv(r).await,
            MaybeUnboundedReceiver::Unbounded(ref mut r) => ChannelReceiver::recv(r).await,
        }
    }
}

#[async_trait]
impl<M: Send + Sync + 'static> OneshotSender<M> for tokio::sync::oneshot::Sender<M> {
    async fn send(self: Box<Self>, msg: M) {
        (*self).send(msg).ok();
    }
}

#[async_trait]
impl<M: Send + Sync + 'static> OneshotSender<M> for tokio::sync::mpsc::Sender<M> {
    async fn send(self: Box<Self>, msg: M) {
        (&*self).send(msg).await.ok();
    }
}

#[async_trait]
impl<M: Send + Sync + 'static> OneshotSender<M> for tokio::sync::broadcast::Sender<M> {
    async fn send(self: Box<Self>, msg: M) {
        (&*self).send(msg).ok();
    }
}

#[async_trait]
impl<M: Send + Sync + 'static> OneshotSender<M> for tokio::sync::watch::Sender<M> {
    async fn send(self: Box<Self>, msg: M) {
        (&*self).send(msg).ok();
    }
}

/// A bounded channel.
pub type BoundedChannel<M> = (BoundedSender<M>, BoundedReceiver<M>);
/// An unbounded channel.
pub type UnboundedChannel<M> = (UnboundedSender<M>, UnboundedReceiver<M>);
/// A channel that is either bounded or unbounded.
pub type MaybeUnboundedChannel<M> = (MaybeUnboundedSender<M>, MaybeUnboundedReceiver<M>);
