//! Try-catch blocks.
//!
//! Many functions in Julia can throw exceptions, jlrs provides checked and unchecked variants of
//! such functions. The checked variant calls the function in a try-catch block and returns a
//! `Result` to indicate whether or not the operation succeeded, while the unchecked variant
//! simply calls the function. If an exception is thrown and it isn't caught the application is
//! aborted. The main disadvantage of the checked variants is that a new try-catch block is
//! created every time the function is called and creating such a block can be relatively
//! expensive.
//!
//! Instead of using the checked variants you can create a try-catch block from Rust with
//! [`catch_exceptions`]. This function takes two closures, the try and catch blocks. Neither of
//! these closures may panic.
//!
//! Exceptions work by jumping to the nearest enclosing catch block, you must guarantee that every
//! frame the exception jumps over is a "POF", or "Plain Old Frame": there are no pending drops or
//! `catch_unwind` calls. See this [blog post] for more information.
//!
//! Only exception-safe local scopes or unsized local scopes should be used in the try-block:
//! Julia's unwinding mechanism will clean those up correctly. It is UB for an exception to jump
//! out of other scopes because the exception will jump over non-POF frames.
//!
//! [blog post]: https://blog.rust-lang.org/inside-rust/2021/01/26/ffi-unwind-longjmp.html#pofs-and-stack-deallocating-functions

use std::{cell::Cell, marker::PhantomData, mem::MaybeUninit, ptr::NonNull};

use jl_sys::{jl_print_backtrace, jl_rethrow, jl_rethrow_other, jl_value_t};
use jlrs_sys::{
    jlrs_catch_tag_t, jlrs_catch_trampoline_t, jlrs_current_exception, jlrs_try_trampoline_t,
};

use crate::{
    data::managed::{private::ManagedPriv, value::Stream},
    prelude::Value,
    private::Private,
};

/// A caught exception
#[derive(Clone, Copy)]
pub struct Exception<'exc, 'data> {
    _exc: PhantomData<Cell<&'exc ()>>,
    _data: PhantomData<&'data ()>,
}

impl<'exc, 'data> Exception<'exc, 'data> {
    /// Get the current exception as a `Value`.
    ///
    /// This value is only guaranteed to be rooted in the exception handler.
    pub fn value(self) -> Value<'exc, 'data> {
        unsafe {
            let exc = jlrs_current_exception();
            assert!(!exc.is_null());
            Value::wrap_non_null(NonNull::new_unchecked(exc), Private)
        }
    }

    /// Rethrow the current exception.
    ///
    /// Safety: this new exception causes execution to jump to the next enclosing try-catch block,
    /// all intermediate frames must be POF.
    pub unsafe fn rethrow(self) {
        unsafe { jl_rethrow() };
    }

    /// Rethrow another value as the current exception.
    ///
    /// Safety: this new exception causes execution to jump to the next enclosing try-catch block,
    /// all intermediate frames must be POF.
    pub unsafe fn rethrow_other(self, exc: Value) {
        unsafe { jl_rethrow_other(exc.unwrap(Private)) };
    }

    /// Prints the current exception and the backtrace to stderr.
    pub fn print_backtrace(self) {
        self.value().show(Stream::Stderr);
        eprintln!();

        unsafe {
            jl_print_backtrace();
        }
        eprintln!();
    }

    // Safety: must only be called inside a catch-block.
    unsafe fn new() -> Self {
        Exception {
            _exc: PhantomData,
            _data: PhantomData,
        }
    }
}

/// Call `func`, if an exception is thrown it is caught and `exception_handler` is called. The
/// exception is guaranteed to be rooted inside the exception handler.
///
/// Safety:
///
/// Neither `func` nor `exception_handler` may panic.
///
/// If an exception is thrown, it may only jump over POFs. Only exception-safe and unsized local
/// scopes may be jumped out of.
pub unsafe fn catch_exceptions<T, E>(
    func: impl FnOnce() -> T,
    exception_handler: impl for<'exc> FnOnce(Exception<'exc, '_>) -> E,
) -> Result<T, E> {
    unsafe {
        let mut func = Some(func);
        let func = &mut func;
        let mut handler = Some(exception_handler);
        let handler = &mut handler;

        let try_trampoline = trampoline_for_try(func).unwrap_unchecked();
        let catch_trampoline = trampoline_for_catch(handler).unwrap_unchecked();

        let mut result = MaybeUninit::<T>::uninit();
        let mut err = MaybeUninit::<E>::uninit();

        let res = jlrs_sys::jlrs_try_catch(
            func as *mut _ as *mut _,
            handler as *mut _ as *mut _,
            try_trampoline,
            catch_trampoline,
            (&mut result) as *mut _ as *mut _,
            (&mut err) as *mut _ as *mut _,
        );

        match res {
            jlrs_catch_tag_t::Ok => Ok(result.assume_init()),
            jlrs_catch_tag_t::Exception => Err(err.assume_init()),
        }
    }
}

#[inline]
unsafe extern "C" fn try_trampoline<F: FnOnce() -> T, T>(
    func: &mut Option<F>,
    result: &mut MaybeUninit<T>,
) {
    let res = func.take().unwrap()();
    result.write(res);
}

#[inline]
unsafe extern "C" fn catch_trampoline<F, E>(func: &mut Option<F>, error: &mut MaybeUninit<E>)
where
    F: for<'scope> FnOnce(Exception<'scope, '_>) -> E,
{
    let res = func.take().unwrap()(unsafe { Exception::new() });
    error.write(res);
}

#[inline]
fn trampoline_for_try<F, T>(_: &mut Option<F>) -> Option<jlrs_try_trampoline_t>
where
    F: FnOnce() -> T,
{
    unsafe {
        std::mem::transmute::<
            Option<unsafe extern "C" fn(&mut Option<F>, &mut MaybeUninit<T>)>,
            Option<jlrs_try_trampoline_t>,
        >(Some(try_trampoline::<F, T>))
    }
}

#[inline]
fn trampoline_for_catch<F: for<'scope> FnOnce(Exception<'scope, '_>) -> E, E>(
    _: &mut Option<F>,
) -> Option<jlrs_catch_trampoline_t> {
    unsafe {
        std::mem::transmute::<
            Option<unsafe extern "C" fn(&mut Option<F>, &mut MaybeUninit<E>)>,
            Option<jlrs_catch_trampoline_t>,
        >(Some(catch_trampoline::<F, E>))
    }
}

#[inline]
pub(crate) fn unwrap_exc<'scope>(exc: Exception<'scope, '_>) -> NonNull<jl_value_t> {
    exc.value().unwrap_non_null(Private)
}
