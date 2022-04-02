use crate::multitask::async_task::internal::PersistentMessage;
use crate::multitask::result_sender::ResultSender;
use async_trait::async_trait;
use futures::Future;
use std::sync::Arc;
use tokio::runtime::{Builder, Runtime as TokioRuntime};
use tokio::sync::broadcast::Sender as BroadcastSender;
use tokio::sync::mpsc::{
    channel as mpsc_channel, unbounded_channel, Receiver as MpscReceiver, Sender as MpscSender,
    UnboundedReceiver as UnboundedMpscReceiver, UnboundedSender as UnboundedMpscSender,
};
use tokio::sync::oneshot::{Receiver as OneshotReceiver, Sender as OneshotSender};
use tokio::sync::watch::Sender as WatchSender;
use tokio::task::LocalSet;

#[derive(Debug)]
pub(crate) struct Tokio {
    runtime: TokioRuntime,
    local_set: LocalSet,
}

impl Tokio {
    #[allow(dead_code)]
    pub(crate) fn new() -> Self {
        let runtime = Builder::new_current_thread()
            .thread_name("jlrs-tokio-runtime")
            .enable_time()
            .build()
            .expect("Unable to build tokio runtime");

        Tokio {
            runtime,
            local_set: LocalSet::new(),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn block_on<F, T>(&self, future: F) -> T
    where
        F: Future<Output = T>,
    {
        self.local_set.block_on(&self.runtime, future)
    }
}

#[allow(dead_code)]
pub(crate) enum MaybeUnboundedSender<T> {
    Bounded(MpscSender<T>),
    Unbounded(UnboundedMpscSender<T>),
}

impl<T> MaybeUnboundedSender<T> {
    #[allow(dead_code)]
    pub(crate) async fn send(&self, msg: T) -> Result<(), tokio::sync::mpsc::error::SendError<T>> {
        match self {
            MaybeUnboundedSender::Bounded(b) => match b.send(msg).await {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            },
            MaybeUnboundedSender::Unbounded(ub) => match ub.send(msg) {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            },
        }
    }

    #[allow(dead_code)]
    pub(crate) fn try_send(&self, msg: T) -> Result<(), tokio::sync::mpsc::error::TrySendError<T>> {
        match self {
            MaybeUnboundedSender::Bounded(b) => match b.try_send(msg) {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            },
            MaybeUnboundedSender::Unbounded(ub) => match ub.send(msg) {
                Ok(_) => Ok(()),
                Err(tokio::sync::mpsc::error::SendError(e)) => {
                    Err(tokio::sync::mpsc::error::TrySendError::Closed(e))
                }
            },
        }
    }
}

#[allow(dead_code)]
pub(crate) enum MaybeUnboundedReceiver<T> {
    Bounded(MpscReceiver<T>),
    Unbounded(UnboundedMpscReceiver<T>),
}

impl<T> MaybeUnboundedReceiver<T> {
    #[allow(dead_code)]
    pub(crate) async fn recv(&mut self) -> Option<T> {
        match self {
            MaybeUnboundedReceiver::Bounded(b) => b.recv().await,
            MaybeUnboundedReceiver::Unbounded(ub) => ub.recv().await,
        }
    }
}

#[async_trait]
impl<T: 'static + Send> ResultSender<T> for MpscSender<T> {
    async fn send(self: Box<Self>, msg: T) {
        self.as_ref().send(msg).await.ok();
    }
}

#[async_trait]
impl<T: 'static + Send> ResultSender<T> for UnboundedMpscSender<T> {
    async fn send(self: Box<Self>, msg: T) {
        self.as_ref().send(msg).ok();
    }
}

#[async_trait]
impl<T: 'static + Send> ResultSender<T> for MaybeUnboundedSender<T> {
    async fn send(self: Box<Self>, msg: T) {
        match self.as_ref() {
            MaybeUnboundedSender::Bounded(b) => {
                b.send(msg).await.ok();
            }
            MaybeUnboundedSender::Unbounded(ub) => {
                ub.send(msg).ok();
            }
        }
    }
}

#[async_trait]
impl<T: 'static + Send> ResultSender<T> for OneshotSender<T> {
    async fn send(self: Box<Self>, msg: T) {
        OneshotSender::send(*self, msg).ok();
    }
}

#[async_trait]
impl<T: 'static + Send + Sync> ResultSender<T> for WatchSender<T> {
    async fn send(self: Box<Self>, msg: T) {
        WatchSender::send(&self, msg).ok();
    }
}

#[async_trait]
impl<T: 'static + Send> ResultSender<T> for BroadcastSender<T> {
    async fn send(self: Box<Self>, msg: T) {
        BroadcastSender::send(&self, msg).ok();
    }
}

#[allow(dead_code)]
pub(crate) fn channel<T>(cap: usize) -> (MaybeUnboundedSender<T>, MaybeUnboundedReceiver<T>) {
    if cap == 0 {
        let (s, r) = unbounded_channel();
        (
            MaybeUnboundedSender::Unbounded(s),
            MaybeUnboundedReceiver::Unbounded(r),
        )
    } else {
        let (s, r) = mpsc_channel(cap);
        (
            MaybeUnboundedSender::Bounded(s),
            MaybeUnboundedReceiver::Bounded(r),
        )
    }
}

#[allow(dead_code)]
pub(crate) fn oneshot_channel<T>() -> (OneshotSender<T>, OneshotReceiver<T>) {
    tokio::sync::oneshot::channel()
}

#[allow(dead_code)]
pub(crate) type HandleSender<GT> = Arc<MaybeUnboundedSender<PersistentMessage<GT>>>;
