use std::{ffi::c_void, mem, ptr::null_mut};

const MIN_PAGE_SIZE: usize = 512 / mem::size_of::<usize>();

#[derive(Debug)]
pub(crate) struct StackPage {
    raw: Box<[*mut c_void]>,
}

impl StackPage {
    pub(crate) fn new(min_capacity: usize) -> Self {
        let raw = vec![null_mut(); MIN_PAGE_SIZE.max(min_capacity)];
        StackPage {
            raw: raw.into_boxed_slice(),
        }
    }

    pub(crate) fn size(&self) -> usize {
        self.raw.len()
    }
}

impl Default for StackPage {
    fn default() -> Self {
        Self::new(MIN_PAGE_SIZE)
    }
}

impl AsMut<[*mut c_void]> for StackPage {
    fn as_mut(&mut self) -> &mut [*mut c_void] {
        self.raw.as_mut()
    }
}
