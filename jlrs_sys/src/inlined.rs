// If LTO is not enabled accessing arrays is very slow, so we're going to optimize
// the common case a little.

use std::ptr::NonNull;

use jl_sys::jl_array_t;

#[inline]
pub const unsafe fn jlrs_array_data_fast(a: *mut jl_array_t) -> *mut std::ffi::c_void {
    unsafe { a.cast::<*mut std::ffi::c_void>().read() }
}

#[cfg(julia_1_10)]
#[inline]
pub const unsafe fn jlrs_array_dims_ptr(a: *mut jl_array_t) -> *mut usize {
    #[repr(C)]
    struct RawArray {
        data: *mut std::ffi::c_void,
        length: usize,
        flags: u16,
        elsize: u16,
        offset: u32,
        nrows: usize,
    }

    const OFFSET: usize = std::mem::offset_of!(RawArray, nrows);
    unsafe { (a as *mut u8).add(OFFSET) as *mut usize }
}

#[cfg(not(julia_1_10))]
#[inline]
pub const unsafe fn jlrs_array_dims_ptr(a: *mut jl_array_t) -> *mut usize {
    unsafe {
        #[repr(C)]
        struct GenericMemoryRef {
            ptr_or_offset: *mut std::ffi::c_void,
            mem: *mut std::ffi::c_void,
        }

        #[repr(C)]
        struct RawArray {
            ref_inner: GenericMemoryRef,
        }

        const OFFSET: usize = std::mem::size_of::<RawArray>();
        (a as *mut u8).add(OFFSET) as *mut usize
    }
}

#[cfg(not(julia_1_10))]
#[inline]
pub const unsafe fn jlrs_array_mem(a: *mut jl_array_t) -> *mut crate::types::jl_value_t {
    unsafe {
        #[repr(C)]
        struct GenericMemoryRef {
            ptr_or_offset: *mut std::ffi::c_void,
            mem: *mut std::ffi::c_void,
        }

        #[repr(C)]
        struct RawArray {
            ref_inner: GenericMemoryRef,
        }

        NonNull::new_unchecked(a as *mut RawArray)
            .read()
            .ref_inner
            .mem as _
    }
}

#[inline]
pub const unsafe fn jlrs_array_ndims_fast(a: *mut jl_array_t) -> usize {
    unsafe {
        #[repr(C)]
        struct RawDataType {
            name: *mut std::ffi::c_void,
            super_ty: *mut std::ffi::c_void,
            parameters: *mut std::ffi::c_void,
        }

        #[repr(C)]
        union Header {
            header: usize,
            next: *mut TaggedValue,
            ty: *mut RawDataType,
            bits: usize,
        }

        #[repr(C)]
        struct TaggedValue {
            header: Header,
        }

        let a = a as *mut u8;
        let tagged = a
            .sub(std::mem::size_of::<TaggedValue>())
            .cast::<TaggedValue>();
        let header = NonNull::new_unchecked(tagged).read().header.header;
        let dt = (header & !15) as *mut RawDataType;
        let params = NonNull::new_unchecked(dt).read().parameters as *mut *mut usize;
        params.add(2).read().read()
    }
}
