//! An atomic pointer to an `extern "C" fn`.
//!
//! Some structs defined by the Julia C API contain atomic function pointers. This module defines
//! `AtomicCFnPointer` to ensure these atomic fields are also atomic fields in the generated
//! bindings.

use std::{
    ffi::c_void,
    marker::PhantomData,
    sync::atomic::{AtomicPtr, Ordering},
};

/// An atomic pointer to an `Option<extern "C" fn>`.
#[repr(transparent)]
pub struct AtomicCFnPtr<T: CFnPtr> {
    ptr: AtomicPtr<c_void>,
    _marker: PhantomData<T>,
}

impl<T: CFnPtr> AtomicCFnPtr<T> {
    pub unsafe fn new(func: T) -> Self {
        let ptr: *mut c_void = std::mem::transmute_copy(&func);

        AtomicCFnPtr {
            ptr: AtomicPtr::new(ptr),
            _marker: PhantomData,
        }
    }

    pub fn load(&self, order: Ordering) -> T {
        let ptr = self.ptr.load(order);
        unsafe { std::mem::transmute_copy(&ptr) }
    }
}

pub unsafe trait CFnPtr: private::CFnPtrPriv {}
unsafe impl<T: private::CFnPtrPriv> CFnPtr for T {}

mod private {
    pub unsafe trait CFnPtrPriv {}
    unsafe impl<Arg0, Arg1, Arg2, Ret> CFnPtrPriv
        for Option<unsafe extern "C" fn(Arg0, Arg1, Arg2) -> Ret>
    {
    }
    unsafe impl<Arg0, Arg1, Arg2, Arg3, Ret> CFnPtrPriv
        for Option<unsafe extern "C" fn(Arg0, Arg1, Arg2, Arg3) -> Ret>
    {
    }
}
