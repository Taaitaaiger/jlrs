use super::frame::PAGE_SIZE;
use std::{ffi::c_void, ptr::null_mut};

pub(crate) struct Stack {
    raw: Box<[*mut c_void]>,
}

impl Stack {
    pub(crate) fn new() -> Self {
        let raw = vec![null_mut(); PAGE_SIZE];
        Stack {
            raw: raw.into_boxed_slice(),
        }
    }
}

impl AsMut<[*mut c_void]> for Stack {
    fn as_mut(&mut self) -> &mut [*mut c_void] {
        self.raw.as_mut()
    }
}
