use crate::extensions::multitask::async_task::GeneratorMessage;
use crate::extensions::multitask::result_sender::ResultSender;
use async_std::channel::{Receiver, Sender};
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
impl<T: 'static + Send> ResultSender<T> for Sender<T> {
    async fn send(self: Box<Self>, msg: T) {
        self.as_ref().send(msg).await.ok();
    }
}

pub(crate) fn channel<T>(cap: usize) -> (Sender<T>, Receiver<T>) {
    if cap == 0 {
        async_std::channel::unbounded()
    } else {
        async_std::channel::bounded(cap)
    }
}

pub(crate) fn oneshot_channel<T>() -> (Sender<T>, Receiver<T>) {
    async_std::channel::bounded(1)
}

pub(crate) type HandleSender<GT> = Arc<Sender<GeneratorMessage<GT>>>;
