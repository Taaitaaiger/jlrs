//! Returns the result of a completed task.
//!
//! In order to return the results of tasks channels must be used. The sending half must implement
//! [`ResultSender`]. By default this trait is implemented for the `Sender`s from
//! `crossbeam_channel` and the chosed backing runtime. If the `async-std-rt` feature is enabled,
//! this trait is implemented for its `Sender`. If `tokio-rt` is enabled, it's implemented for all
//! `Sender`s in its `sync` module.

use async_trait::async_trait;

/// Trait implemented by the sending halves of channels.
#[async_trait]
pub trait ResultSender<T: 'static + Send>: Send + Sync + 'static {
    /// Send `msg`.
    async fn send(self: Box<Self>, msg: T);
}

#[async_trait]
impl<T: 'static + Send> ResultSender<T> for crossbeam_channel::Sender<T> {
    async fn send(self: Box<Self>, msg: T) {
        crossbeam_channel::Sender::send(&self, msg).ok();
    }
}
