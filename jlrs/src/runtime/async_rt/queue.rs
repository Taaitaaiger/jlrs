use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use deadqueue::resizable::Queue;
use futures::Future;
use futures_concurrency::future::Race;

use crate::error::{JlrsResult, RuntimeError};

struct Queues<T> {
    main_queue: Queue<T>,
    any_queue: Option<Queue<T>>,
    worker_queue: Option<Queue<T>>,
    // there's no method that closes the queue, so the number of senders must be tracked.
    n_senders: AtomicUsize,
}

impl<T> Queues<T> {
    fn new(capacity: usize, has_workers: bool) -> Arc<Self> {
        let (worker_queue, any_queue) = if has_workers {
            (Some(Queue::new(capacity)), Some(Queue::new(capacity)))
        } else {
            (None, None)
        };

        Arc::new(Queues {
            main_queue: Queue::new(capacity),
            any_queue,
            worker_queue,
            n_senders: AtomicUsize::new(1),
        })
    }
}

pub(crate) struct Sender<T> {
    queues: Arc<Queues<T>>,
}

unsafe impl<T: Send> Send for Sender<T> {}
unsafe impl<T: Sync> Sync for Sender<T> {}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        self.queues.n_senders.fetch_sub(1, Ordering::AcqRel);
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        let cloned = self.queues.clone();
        cloned.n_senders.fetch_add(1, Ordering::AcqRel);
        Sender { queues: cloned }
    }
}

impl<T: Send> Sender<T> {
    pub(crate) async fn send(&self, item: T) {
        if let Some(ref q) = self.queues.any_queue {
            q.push(item).await
        } else {
            self.send_main(item).await
        }
    }

    pub(crate) fn try_send_or_return(&self, item: T) -> Option<T> {
        if let Some(ref q) = self.queues.any_queue {
            q.try_push(item).err()?;
        } else {
            self.try_send_worker_or_return(item)?;
        }
        None
    }

    pub(crate) fn resize_queue<'own>(
        &'own self,
        capacity: usize,
    ) -> Option<impl 'own + Future<Output = ()>> {
        self.queues.any_queue.as_ref().map(|q| q.resize(capacity))
    }

    pub(crate) async fn send_main(&self, item: T) {
        self.queues.main_queue.push(item).await
    }

    pub(crate) fn try_send_main_or_return(&self, item: T) -> Option<T> {
        self.queues.main_queue.try_push(item).err()?;
        None
    }

    pub(crate) fn resize_main_queue<'own>(
        &'own self,
        capacity: usize,
    ) -> impl 'own + Future<Output = ()> {
        self.queues.main_queue.resize(capacity)
    }

    pub(crate) async fn send_worker(&self, item: T) {
        if let Some(ref q) = self.queues.worker_queue {
            q.push(item).await
        } else {
            self.send_main(item).await
        }
    }

    pub(crate) fn try_send_worker_or_return(&self, item: T) -> Option<T> {
        if let Some(ref q) = self.queues.worker_queue {
            q.try_push(item).err()?;
        } else {
            self.try_send_worker_or_return(item)?;
        }
        None
    }

    pub(crate) fn resize_worker_queue<'own>(
        &'own self,
        capacity: usize,
    ) -> Option<impl 'own + Future<Output = ()>> {
        self.queues
            .worker_queue
            .as_ref()
            .map(|q| q.resize(capacity))
    }
}

pub(crate) struct Receiver<T> {
    queue: Arc<Queues<T>>,
}

unsafe impl<T: Send> Send for Receiver<T> {}
unsafe impl<T: Sync> Sync for Receiver<T> {}

impl<T> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        Receiver {
            queue: self.queue.clone(),
        }
    }
}

impl<T: Send> Receiver<T> {
    pub(crate) async fn recv_main(&self) -> JlrsResult<T> {
        if self.queue.n_senders.load(Ordering::Acquire) == 0 {
            return match self.try_recv_main() {
                Some(t) => Ok(t),
                None => Err(RuntimeError::ChannelClosed)?,
            };
        }

        if self.queue.any_queue.is_some() {
            Ok((
                self.queue.main_queue.pop(),
                self.queue.any_queue.as_ref().unwrap().pop(),
            )
                .race()
                .await)
        } else {
            Ok(self.queue.main_queue.pop().await)
        }
    }

    fn try_recv_main(&self) -> Option<T> {
        if let Some(popped_main) = self.queue.main_queue.try_pop() {
            return Some(popped_main);
        }

        self.queue.any_queue.as_ref().map(|q| q.try_pop()).flatten()
    }

    #[cfg(any(feature = "julia-1-9", feature = "julia-1-10"))]
    pub(crate) async fn recv_worker(&self) -> JlrsResult<T> {
        if self.queue.n_senders.load(Ordering::Acquire) == 0 {
            return match self.try_recv_worker() {
                Some(t) => Ok(t),
                None => Err(RuntimeError::ChannelClosed)?,
            };
        }

        Ok((
            self.queue.worker_queue.as_ref().unwrap().pop(),
            self.queue.any_queue.as_ref().unwrap().pop(),
        )
            .race()
            .await)
    }

    #[cfg(any(feature = "julia-1-9", feature = "julia-1-10"))]
    fn try_recv_worker(&self) -> Option<T> {
        if let Some(popped_worker) = self.queue.worker_queue.as_ref().unwrap().try_pop() {
            return Some(popped_worker);
        }

        self.queue.any_queue.as_ref().unwrap().try_pop()
    }
}

pub(crate) fn channel<T>(capacity: usize, has_workers: bool) -> (Sender<T>, Receiver<T>) {
    let capacity = if capacity == 0 { 32 } else { capacity };
    let queue = Queues::new(capacity, has_workers);
    let sender = Sender {
        queues: queue.clone(),
    };

    let receiver = Receiver { queue };

    (sender, receiver)
}
