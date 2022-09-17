use crate::{
    memory::context::ContextFrame,
    runtime::{builder::RuntimeBuilder, sync_rt::Julia},
};
use std::{
    cell::{Cell, RefCell},
    ffi::c_void,
    ptr::null_mut,
};

pub fn static_context_frame() -> &'static ContextFrame {
    CONTEXT_FRAME.as_context_frame()
}

#[repr(C)]
struct StaticFrame([Cell<*mut c_void>; 3]);
unsafe impl Sync for StaticFrame {}

impl StaticFrame {
    fn as_context_frame(&'static self) -> &'static ContextFrame {
        unsafe { std::mem::transmute(self) }
    }
}

static CONTEXT_FRAME: StaticFrame = StaticFrame([
    Cell::new(4 as _),
    Cell::new(null_mut()),
    Cell::new(null_mut()),
]);

thread_local! {
    #[doc(hidden)]
    pub static JULIA: RefCell<Julia<'static>> = {
        let context_frame = static_context_frame();
        RefCell::new(unsafe {RuntimeBuilder::new().start(context_frame).unwrap() })
    }
}
