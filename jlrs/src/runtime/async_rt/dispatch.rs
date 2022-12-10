//! Dispatch a task to the async runtime.

use std::{fmt::Debug, marker::PhantomData};

use crate::{
    async_util::affinity::{Affinity, ToAny, ToMain, ToWorker},
    runtime::async_rt::{queue::Sender, Message},
};

/// Dispatches a task to the aasync runtime.
pub struct Dispatch<'a, D> {
    msg: Message,
    sender: &'a Sender<Message>,
    _dispatch: PhantomData<D>,
}

impl<'a, D> Debug for Dispatch<'a, D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Dispatch").finish()
    }
}

impl<'a, D: Affinity> Dispatch<'a, D> {
    pub(super) fn new(sender: &'a Sender<Message>, msg: Message) -> Self {
        Dispatch {
            msg,
            sender,
            _dispatch: PhantomData,
        }
    }
}

impl<'a, D: ToAny> Dispatch<'a, D> {
    /// Dispatch the task to any thread.
    ///
    /// The dispatched task can be handled by either the main thread or any of the worker threads.
    /// This method doesn't resolve until the task has been successfully dispatched.
    pub async fn dispatch_any(self) {
        self.sender.send(self.msg).await
    }

    /// Try to dispatch the task to any thread.
    ///
    /// The dispatched task can be handled by either the main thread or any of the worker threads.
    /// If the backing queue is full, the dispatcher is returned to allow retrying.
    pub fn try_dispatch_any(self) -> Result<(), Self> {
        if let Some(msg) = self.sender.try_send(self.msg) {
            Err(Dispatch {
                msg,
                sender: self.sender,
                _dispatch: PhantomData,
            })
        } else {
            Ok(())
        }
    }
}

impl<'a, D: ToMain> Dispatch<'a, D> {
    /// Dispatch the task to the main thread.
    ///
    /// The dispatched task is guaranteed to be handled by the main thread. This method doesn't
    /// resolve until the task has been successfully dispatched.
    pub async fn dispatch_main(self) {
        self.sender.send_main(self.msg).await
    }

    /// Try to dispatch the task to the main thread.
    ///
    /// The dispatched task is guaranteed to be handled by the main thread. If the backing queue
    /// is full, the dispatcher is returned to allow retrying.
    pub fn try_dispatch_main(self) -> Result<(), Self> {
        if let Some(msg) = self.sender.try_send_main(self.msg) {
            Err(Dispatch {
                msg,
                sender: self.sender,
                _dispatch: PhantomData,
            })
        } else {
            Ok(())
        }
    }
}

impl<'a, D: ToWorker> Dispatch<'a, D> {
    /// Dispatch the task to a worker thread.
    ///
    /// The dispatched task is guaranteed to be handled by a worker thread if they're used,
    /// otherwise it's handled by the main thread. This method doesn't resolve until the task has
    /// been successfully dispatched.
    pub async fn dispatch_worker(self) {
        self.sender.send_worker(self.msg).await
    }

    /// Try to dispatch the task to a worker thread.
    ///
    /// The dispatched task is guaranteed to be handled by a worker thread if they're used,
    /// otherwise it's handled by the main thread.  If the backing queue is full, the dispatcher
    /// is returned to allow retrying.
    pub fn try_dispatch_worker(self) -> Result<(), Self> {
        if let Some(msg) = self.sender.try_send_worker(self.msg) {
            Err(Dispatch {
                msg,
                sender: self.sender,
                _dispatch: PhantomData,
            })
        } else {
            Ok(())
        }
    }
}
