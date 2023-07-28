#[julia_version(windows_lts = false)]
use std::ptr::NonNull;
use std::{
    any::Any,
    ffi::c_void,
    hint::unreachable_unchecked,
    mem::MaybeUninit,
    panic::{catch_unwind, AssertUnwindSafe},
    ptr::null_mut,
};

#[julia_version(windows_lts = false)]
use jl_sys::jlrs_catch_tag_t_JLRS_CATCH_EXCEPTION;
use jl_sys::{
    jlrs_catch_t, jlrs_catch_tag_t_JLRS_CATCH_OK, jlrs_catch_tag_t_JLRS_CATCH_PANIC,
    jlrs_catch_wrapper,
};
use jlrs_macros::julia_version;

#[julia_version(windows_lts = true)]
use crate::{
    call::Call,
    data::managed::module::JlrsCore,
    prelude::{Target, Value},
};
#[julia_version(windows_lts = false)]
use crate::{
    data::managed::{Managed, private::ManagedPriv},
    memory::target::unrooted::Unrooted,
    prelude::{Target, Value},
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
pub unsafe fn catch_exceptions<G, H, T, E>(mut func: G, exception_handler: H) -> Result<T, E>
where
    G: FnMut() -> T,
    H: for<'exc> FnOnce(Value<'exc, 'static>) -> E,
{
    let func = &mut func;
    let trampoline = trampoline_for(func);
    let mut result = MaybeUninit::<T>::uninit();

    let res = jlrs_catch_wrapper(
        func as *mut _ as *mut _,
        trampoline,
        (&mut result) as *mut _ as *mut _,
    );

    match res.tag {
        x if x == jlrs_catch_tag_t_JLRS_CATCH_OK => Ok(result.assume_init()),
        x if x == jlrs_catch_tag_t_JLRS_CATCH_EXCEPTION => {
            let ptr = NonNull::new_unchecked(res.error.cast());
            let unrooted = Unrooted::new();
            unrooted
                .local_scope::<_, _, 1>(|frame| {
                    // Root the exception because we're not in an actual catch block.
                    let v = Value::wrap_non_null(ptr, Private).root(frame);
                    Ok(Err(exception_handler(v)))
                })
                .unwrap_unchecked()
        }
        x if x == jlrs_catch_tag_t_JLRS_CATCH_PANIC => {
            let err: Box<Box<dyn Any + Send>> = Box::from_raw(res.error.cast());
            std::panic::resume_unwind(err)
        }
        _ => unreachable_unchecked(),
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
pub unsafe fn catch_exceptions<G, H, T, E>(mut func: G, exception_handler: H) -> Result<T, E>
where
    G: FnMut() -> T,
    H: for<'exc> FnOnce(Value<'exc, 'static>) -> E,
{
    let func = &mut func;
    let trampoline = trampoline_for(func);
    let mut result = MaybeUninit::<T>::uninit();
    let unrooted = crate::memory::target::unrooted::Unrooted::new();

    // The JL_TRY and JL_CATCH macros don't work when Julia 1.6 is used on Windows, so we're
    // going to jump back to Rust code from Julia rather than C.
    let caller = JlrsCore::call_catch_wrapper(&unrooted);
    let trampoline = std::mem::transmute::<_, *mut c_void>(trampoline);

    let res = unrooted
        .with_local_scope::<_, _, 4>(|target, mut frame| {
            let result = &mut result;

            let catch_wrapper = Value::new(&mut frame, jlrs_catch_wrapper as *mut c_void);
            let func = Value::new(&mut frame, func as *mut _ as *mut c_void);
            let trampoline = Value::new(&mut frame, trampoline);
            let result = Value::new(&mut frame, result as *mut _ as *mut c_void);

            Ok(caller.call(target, [catch_wrapper, func, trampoline, result]))
        })
        .unwrap_unchecked();

    match res {
        Ok(res) => {
            let res = res.ptr().cast::<jlrs_catch_t>().as_ref();
            match res.tag {
                x if x == jlrs_catch_tag_t_JLRS_CATCH_OK => Ok(result.assume_init()),
                x if x == jlrs_catch_tag_t_JLRS_CATCH_PANIC => {
                    let err: Box<Box<dyn Any + Send>> = Box::from_raw(res.error.cast());
                    std::panic::resume_unwind(err)
                }
                _ => unreachable_unchecked(),
            }
        }
        Err(exc) => unrooted
            .local_scope::<_, _, 1>(|frame| {
                let value = exc.root(frame);
                Ok(Err(exception_handler(value)))
            })
            .unwrap_unchecked(),
    }
}

#[inline]
unsafe extern "C" fn trampoline<'frame, F: FnMut() -> T, T>(
    func: &mut F,
    result: &mut MaybeUninit<T>,
) -> jlrs_catch_t {
    let res = catch_unwind(AssertUnwindSafe(|| func()));

    match res {
        Ok(v) => {
            result.write(v);
            jlrs_catch_t {
                tag: jlrs_catch_tag_t_JLRS_CATCH_OK,
                error: null_mut(),
            }
        }
        Err(e) => {
            // extra box because it's a fat pointer
            jlrs_catch_t {
                tag: jlrs_catch_tag_t_JLRS_CATCH_PANIC,
                error: Box::leak(Box::new(e)) as *mut _ as *mut _,
            }
        }
    }
}

#[inline]
fn trampoline_for<'frame, F: FnMut() -> T, T>(
    _: &mut F,
) -> Option<unsafe extern "C" fn(*mut c_void, *mut c_void) -> jlrs_catch_t> {
    unsafe {
        std::mem::transmute::<
            Option<unsafe extern "C" fn(&mut F, &mut MaybeUninit<T>) -> jlrs_catch_t>,
            Option<unsafe extern "C" fn(*mut c_void, *mut c_void) -> jlrs_catch_t>,
        >(Some(trampoline::<F, T>))
    }
}
