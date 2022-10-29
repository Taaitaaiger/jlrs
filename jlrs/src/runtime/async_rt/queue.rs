use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use deadqueue::resizable::Queue;
use futures::Future;

use crate::error::{JlrsResult, RuntimeError};

struct AsyncQueue<T> {
    queue: Queue<T>,
    // there's no method that closes the queue, so the number of senders must be tracked.
    n_senders: AtomicUsize,
}

impl<T> AsyncQueue<T> {
    fn new(capacity: usize) -> Arc<Self> {
        Arc::new(AsyncQueue {
            queue: Queue::new(capacity),
            n_senders: AtomicUsize::new(1),
        })
    }
}

pub(crate) struct Sender<T> {
    queue: Arc<AsyncQueue<T>>,
}

unsafe impl<T: Send> Send for Sender<T> {}
unsafe impl<T: Sync> Sync for Sender<T> {}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        self.queue.n_senders.fetch_sub(1, Ordering::AcqRel);
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        let cloned = self.queue.clone();
        cloned.n_senders.fetch_add(1, Ordering::AcqRel);
        Sender { queue: cloned }
    }
}

impl<T: Send> Sender<T> {
    pub(crate) async fn send(&self, item: T) {
        self.queue.queue.push(item).await
    }

    pub(crate) fn try_send(&self, item: T) -> JlrsResult<()> {
        self.queue
            .queue
            .try_push(item)
            .map_err(|_| RuntimeError::ChannelFull)?;

        Ok(())
    }

    pub(crate) fn resize_queue<'own>(
        &'own self,
        capacity: usize,
    ) -> impl 'own + Future<Output = ()> {
        self.queue.queue.resize(capacity)
    }
}

pub(crate) struct Receiver<T> {
    queue: Arc<AsyncQueue<T>>,
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
    pub(crate) async fn recv(&self) -> JlrsResult<T> {
        if self.queue.n_senders.load(Ordering::Acquire) == 0 {
            return match self.try_recv() {
                Some(t) => Ok(t),
                None => Err(RuntimeError::ChannelClosed)?,
            };
        }

        Ok(self.queue.queue.pop().await)
    }

    fn try_recv(&self) -> Option<T> {
        self.queue.queue.try_pop()
    }
}

pub(crate) fn channel<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
    let capacity = if capacity == 0 { 32 } else { capacity };
    let queue = AsyncQueue::new(capacity);
    let sender = Sender {
        queue: queue.clone(),
    };

    let receiver = Receiver { queue };

    (sender, receiver)
}
