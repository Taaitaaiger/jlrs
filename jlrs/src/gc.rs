use jl_sys::{jl_gc_collect, jl_gc_collection_t, jl_gc_enable, jl_gc_is_enabled};

use crate::traits::Frame;
use crate::Julia;

#[derive(Debug, Copy, Clone)]
pub enum GcCollection {
    Auto = 0,
    Full = 1,
    Incremental = 2,
}

pub trait Gc: private::Sealed {
    unsafe fn enable_gc(&mut self, on: bool) -> bool {
        jl_gc_enable(on as i32) == 1
    }

    fn gc_is_enabled(&mut self) -> bool {
        unsafe { jl_gc_is_enabled() == 1 }
    }

    unsafe fn gc_collect(&mut self, mode: GcCollection) {
        jl_gc_collect(mode as jl_gc_collection_t)
    }
}

impl Gc for Julia {}
impl<'frame, T: Frame<'frame>> Gc for T {}

mod private {
    use super::{Frame, Julia};
    pub trait Sealed {}
    impl<'a, T: Frame<'a>> Sealed for T {}
    impl Sealed for Julia {}
}
