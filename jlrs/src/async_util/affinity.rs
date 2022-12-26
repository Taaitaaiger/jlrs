//! The thread-affinity of a task.
//!
//! The async runtime can use worker threads since Julia 1.9, the thread-affinity of a task must
//! be set through the associated `Affinity` type. This configures whether the task can run on
//! any thread, only the main thread. or only worker threads if they're used.
//!
//! Three affitinities are available: [`DispatchAny`], [`DispatchMain`], and [`DispatchWorker`].

use self::private::AffinityPriv;

/// The thread-affinity of a task.
pub trait Affinity: AffinityPriv {}

/// Enables dispatching to worker threads.
pub trait ToWorker: Affinity {}

/// Enables dispatching to the main thread.
pub trait ToMain: Affinity {}

/// Enables dispatching to any thread.
pub trait ToAny: Affinity {}

/// Affinity to worker threads.
///
/// A task with this affinity is guaranteed to be handled by a worker thread if worker threads are
/// used. If no worker threads are used the task is dispatched to the main thread.
pub enum DispatchWorker {}
impl Affinity for DispatchWorker {}
impl ToWorker for DispatchWorker {}

/// Affinity to the main thread.
///
/// A task with this affinity is guaranteed to be handled by the main thread.
pub enum DispatchMain {}
impl Affinity for DispatchMain {}
impl ToMain for DispatchMain {}

/// Affinity to any thread.
///
/// A task with this affinity can be handled by either the main thread or a worker thread.
pub enum DispatchAny {}
impl Affinity for DispatchAny {}
impl ToWorker for DispatchAny {}
impl ToMain for DispatchAny {}
impl ToAny for DispatchAny {}

mod private {
    use super::{DispatchAny, DispatchMain, DispatchWorker};

    pub trait AffinityPriv {}

    impl AffinityPriv for DispatchAny {}
    impl AffinityPriv for DispatchMain {}
    impl AffinityPriv for DispatchWorker {}
}
