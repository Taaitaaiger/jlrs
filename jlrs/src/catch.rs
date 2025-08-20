//! Try-catch blocks.
//!
//! Many functions in Julia can throw exceptions, jlrs provides checked and unchecked variants of
//! such functions. The checked variant calls the function in a try-catch block and returns a
//! `Result` to indicate whether or not the operation succeeded, while the unchecked variant
//! simply calls the function. If an exception is thrown and it isn't caught the application is
//! aborted. The main disadvantage of the checked variants is that a new try-catch block is
//! created every time the function is called and creating such a block is relatively expensive.
//!
//! Instead of using the checked variants you can create a try-catch block from Rust with
//! [`catch_exceptions`]. This function takes two closures, think of them as the content of the
//! try and catch blocks respectively.
//!
//! Because exceptions work by jumping to the nearest enclosing catch block, you must guarantee
//! that there are no pending drops when an exception is thrown. See this [blog post] for more
//! information.
//!
//! Only local scopes may be created in the try-block, Julia's unwinding mechanism ensures that
//! any scope we jump out of is removed from the GC stack. Dynamic scopes (i.e. scopes that
//! provide a `GcFrame`) depend on `Drop` so jumping out of them is not sound.
//!
//! [blog post]: https://blog.rust-lang.org/inside-rust/2021/01/26/ffi-unwind-longjmp.html#pofs-and-stack-deallocating-functions

use std::{
    any::Any,
    ffi::c_void,
    mem::MaybeUninit,
    panic::{catch_unwind, AssertUnwindSafe},
    ptr::{null_mut, NonNull},
};

use jl_sys::{jl_value_t, jlrs_catch_t, jlrs_catch_tag_t, jlrs_try_catch};

use crate::{
    data::managed::{private::ManagedPriv, Managed},
    memory::target::unrooted::Unrooted,
    prelude::{LocalScope, Value},
    private::Private,
};

/// Call `func`, if an exception is thrown it is caught and `exception_handler` is called. The
/// exception is guaranteed to be rooted inside the exception handler.
///
/// Safety:
///
/// If an exception is thrown, there must be no pending drops. Only local scopes may be created in
/// `func`.
pub unsafe fn catch_exceptions<T, E>(
    func: impl FnOnce() -> T,
    exception_handler: impl for<'exc> FnOnce(Value<'exc, 'static>) -> E,
) -> Result<T, E> {
    let mut func = Some(func);
    let func = &mut func;
    let trampoline = trampoline_for(func).unwrap_unchecked();
    let mut result = MaybeUninit::<T>::uninit();

    let res = jlrs_try_catch(
        func as *mut _ as *mut _,
        trampoline,
        (&mut result) as *mut _ as *mut _,
    );

    match res.tag {
        jlrs_catch_tag_t::Ok => Ok(result.assume_init()),
        jlrs_catch_tag_t::Exception => {
            let ptr = NonNull::new_unchecked(res.error.cast());
            let unrooted = Unrooted::new();
            unrooted.local_scope::<_, 1>(|frame| {
                // Root the exception because we're not in an actual catch block.
                let v = Value::wrap_non_null(ptr, Private).root(frame);
                Err(exception_handler(v))
            })
        }
        jlrs_catch_tag_t::Panic => {
            let err: Box<Box<dyn Any + Send>> = Box::from_raw(res.error.cast());
            std::panic::resume_unwind(err)
        }
    }
}

#[inline]
unsafe extern "C" fn trampoline<'frame, F: FnOnce() -> T, T>(
    func: &mut Option<F>,
    result: &mut MaybeUninit<T>,
) -> jlrs_catch_t {
    let res = catch_unwind(AssertUnwindSafe(|| func.take().unwrap()()));

    match res {
        Ok(v) => {
            result.write(v);
            jlrs_catch_t {
                tag: jlrs_catch_tag_t::Ok,
                error: null_mut(),
            }
        }
        Err(e) => {
            // extra box because it's a fat pointer
            jlrs_catch_t {
                tag: jlrs_catch_tag_t::Panic,
                error: Box::leak(Box::new(e)) as *mut _ as *mut _,
            }
        }
    }
}

#[inline]
fn trampoline_for<'frame, F: FnOnce() -> T, T>(
    _: &mut Option<F>,
) -> Option<unsafe extern "C" fn(*mut c_void, *mut c_void) -> jlrs_catch_t> {
    unsafe {
        std::mem::transmute::<
            Option<unsafe extern "C" fn(&mut Option<F>, &mut MaybeUninit<T>) -> jlrs_catch_t>,
            Option<unsafe extern "C" fn(*mut c_void, *mut c_void) -> jlrs_catch_t>,
        >(Some(trampoline::<F, T>))
    }
}

#[inline]
pub(crate) fn unwrap_exc(exc: Value) -> NonNull<jl_value_t> {
    exc.unwrap_non_null(Private)
}
