
use std::sync::atomic::AtomicPtr;

use crate::prelude::Wrapper;

pub struct Atomic<'scope, 'data, W: Wrapper<'scope, 'data>> {
    ptr: AtomicPtr<W::Wraps>
}