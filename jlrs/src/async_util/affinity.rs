//! The thread-affinity of a task.
//!
//! The async runtime can use worker threads since Julia 1.9, the tread-affinity of a task must
//! be set through the associated `Affinity` type. This configures whether the task can run on
//! any thread, or only the main or worker threads.

use self::private::AffinityPriv;

pub trait Affinity: AffinityPriv {}

pub trait ToWorker: Affinity {}

pub trait ToMain: Affinity {}

pub trait ToAny: Affinity {}

pub enum DispatchWorker {}
impl Affinity for DispatchWorker {}
impl ToWorker for DispatchWorker {}

pub enum DispatchMain {}
impl Affinity for DispatchMain {}
impl ToMain for DispatchMain {}

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
