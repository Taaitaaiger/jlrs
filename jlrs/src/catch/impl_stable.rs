#[julia_version(windows_lts = false)]
use std::ptr::NonNull;
use std::{
    any::Any,
    ffi::c_void,
    mem::MaybeUninit,
    panic::{catch_unwind, AssertUnwindSafe},
    ptr::null_mut,
};

use jl_sys::{jlrs_catch_t, jlrs_catch_tag_t, jlrs_try_catch};
use jlrs_macros::julia_version;

use crate::prelude::LocalScope;
#[julia_version(windows_lts = true)]
use crate::{
    call::Call,
    prelude::{Target, Value},
};
#[julia_version(windows_lts = false)]
use crate::{
    data::managed::{private::ManagedPriv, Managed},
    memory::target::unrooted::Unrooted,
    prelude::Value,
    private::Private,
};

/// Call `func`, if an exception is thrown it is caught and `exception_handler` is called. The
/// exception is guaranteed to be rooted inside the exception handler.
///
/// Safety:
///
/// If an exception is thrown, there must be no pending drops. Only local scopes may be created in
/// `func`.
#[julia_version(windows_lts = false)]
pub unsafe fn catch_exceptions<F, H, T, E>(func: F, exception_handler: H) -> Result<T, E>
where
    F: FnOnce() -> T,
    H: for<'exc> FnOnce(Value<'exc, 'static>) -> E,
{
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

/// Call `func`, if an exception is thrown it is caught and `exception_handler` is called. The
/// exception is guaranteed to be rooted inside the exception handler.
///
/// Safety:
///
/// If an exception is thrown, there must be no pending drops. Only local scopes may be created in
/// `func`.
#[julia_version(windows_lts = true)]
pub unsafe fn catch_exceptions<F, H, T, E>(func: F, exception_handler: H) -> Result<T, E>
where
    F: FnOnce() -> T,
    H: for<'exc> FnOnce(Value<'exc, 'static>) -> E,
{
    let mut func = Some(func);
    let func = &mut func;
    let trampoline = trampoline_for(func);
    let mut result = MaybeUninit::<T>::uninit();
    let unrooted = crate::memory::target::unrooted::Unrooted::new();

    // The JL_TRY and JL_CATCH macros don't work when Julia 1.6 is used on Windows, so we're
    // going to jump back to Rust code from Julia rather than C.
    let caller = crate::data::managed::module::JlrsCore::call_catch_wrapper(&unrooted);
    let trampoline = std::mem::transmute::<_, *mut c_void>(trampoline);

    let res = unrooted.with_local_scope::<_, _, 4>(|target, mut frame| {
        let result = &mut result;

        let catch_wrapper = Value::new(&mut frame, jlrs_try_catch as *mut c_void);
        let func = Value::new(&mut frame, func as *mut _ as *mut c_void);
        let trampoline = Value::new(&mut frame, trampoline);
        let result = Value::new(&mut frame, result as *mut _ as *mut c_void);

        caller.call(target, [catch_wrapper, func, trampoline, result])
    });

    match res {
        Ok(res) => {
            let res = res.ptr().cast::<jlrs_catch_t>().as_ref();
            match res.tag {
                _tag @ jlrs_catch_tag_t::Ok => Ok(result.assume_init()),
                _tag @ jlrs_catch_tag_t::Panic => {
                    let err: Box<Box<dyn Any + Send>> = Box::from_raw(res.error.cast());
                    std::panic::resume_unwind(err)
                }
                _ => ::std::hint::unreachable_unchecked(),
            }
        }
        Err(exc) => unrooted.local_scope::<_, 1>(|frame| {
            let value = exc.root(frame);
            Err(exception_handler(value))
        }),
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
