use std::{
    any::Any,
    ffi::c_void,
    mem::MaybeUninit,
    panic::{catch_unwind, AssertUnwindSafe},
    ptr::{null_mut, NonNull},
};

use jl_sys::{
    jlrs_catch_t, jlrs_catch_tag_t_JLRS_CATCH_ERR, jlrs_catch_tag_t_JLRS_CATCH_EXCEPTION,
    jlrs_catch_tag_t_JLRS_CATCH_OK, jlrs_catch_tag_t_JLRS_CATCH_PANIC, jlrs_catch_wrapper,
};
use jlrs_macros::julia_version;

#[julia_version(windows_lts = true)]
use crate::{
    call::Call,
    data::managed::{module::Module, value::Value},
    memory::{gc::Gc, target::unrooted::Unrooted},
};
use crate::{
    data::managed::value::ValueRef,
    error::{JlrsResult, JuliaResultRef},
    memory::target::frame::GcFrame,
};

unsafe extern "C" fn trampoline_with_slots<'frame, F, T>(
    func: &mut F,
    frame_slice: &mut GcFrame<'frame>,
    result: &mut MaybeUninit<T>,
) -> jlrs_catch_t
where
    F: FnMut(&mut GcFrame<'frame>, &mut MaybeUninit<T>) -> JlrsResult<()>,
{
    let res = catch_unwind(AssertUnwindSafe(|| func(frame_slice, result)));

    match res {
        Ok(Ok(())) => jlrs_catch_t {
            tag: jlrs_catch_tag_t_JLRS_CATCH_OK,
            error: null_mut(),
        },
        Ok(Err(e)) => jlrs_catch_t {
            tag: jlrs_catch_tag_t_JLRS_CATCH_ERR,
            error: Box::leak(e) as *mut _ as *mut _,
        },
        Err(e) => {
            // extra box because it's a fat pointer
            jlrs_catch_t {
                tag: jlrs_catch_tag_t_JLRS_CATCH_PANIC,
                error: Box::leak(Box::new(e)) as *mut _ as *mut _,
            }
        }
    }
}

fn trampoline_with_slots_for<
    'frame,
    F: FnMut(&mut GcFrame<'frame>, &mut MaybeUninit<T>) -> JlrsResult<()>,
    T,
>(
    _: &mut F,
) -> Option<unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void) -> jlrs_catch_t> {
    unsafe {
        std::mem::transmute::<
            Option<
                unsafe extern "C" fn(
                    &mut F,
                    &mut GcFrame<'frame>,
                    &mut MaybeUninit<T>,
                ) -> jlrs_catch_t,
            >,
            Option<unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void) -> jlrs_catch_t>,
        >(Some(trampoline_with_slots::<F, T>))
    }
}

#[julia_version(windows_lts = false)]
pub(crate) unsafe fn catch_exceptions_with_slots<'frame, 'borrow, 'data, G, T>(
    frame: &'borrow mut GcFrame<'frame>,
    func: &'borrow mut G,
) -> JlrsResult<JuliaResultRef<'frame, 'data, T>>
where
    T: 'frame,
    G: FnMut(&mut GcFrame<'frame>, &mut MaybeUninit<T>) -> JlrsResult<()>,
{
    let trampoline = trampoline_with_slots_for(func);
    let mut result = MaybeUninit::<T>::uninit();

    let res = jlrs_catch_wrapper(
        func as *mut _ as *mut _,
        trampoline,
        (&mut result) as *mut _ as *mut _,
        frame as *mut _ as *mut _,
    );

    match res.tag {
        x if x == jlrs_catch_tag_t_JLRS_CATCH_OK => Ok(Ok(result.assume_init())),
        x if x == jlrs_catch_tag_t_JLRS_CATCH_ERR => Err(Box::from_raw(res.error.cast())),
        x if x == jlrs_catch_tag_t_JLRS_CATCH_EXCEPTION => Ok(Err(ValueRef::wrap(
            NonNull::new_unchecked(res.error.cast()),
        ))),
        x if x == jlrs_catch_tag_t_JLRS_CATCH_PANIC => {
            let err: Box<Box<dyn Any + Send>> = Box::from_raw(res.error.cast());
            std::panic::resume_unwind(err)
        }
        _ => unreachable!(),
    }
}

#[julia_version(windows_lts = true)]
pub(crate) unsafe fn catch_exceptions_with_slots<'frame, 'borrow, 'data, G, T>(
    frame: &'borrow mut GcFrame<'frame>,
    func: &'borrow mut G,
) -> JlrsResult<JuliaResultRef<'frame, 'data, T>>
where
    T: 'frame,
    G: FnMut(&mut MaybeUninit<T>) -> JlrsResult<()>,
{
    let trampoline = trampoline_for(func);
    let mut result = MaybeUninit::<T>::uninit();

    let unrooted = Unrooted::new();
    let enabled = unrooted.gc_is_enabled();
    unrooted.enable_gc(false);

    let caller = Module::package_root_module(&unrooted, "Jlrs")
        .unwrap()
        .global(&unrooted, "call_catch_wrapper")
        .unwrap()
        .as_value();

    let func = Value::new(unrooted, func as *mut _ as *mut c_void).as_value();
    let cw = Value::new(unrooted, jlrs_catch_wrapper as *mut c_void).as_value();
    let trampoline =
        Value::new(unrooted, std::mem::transmute::<_, *mut c_void>(trampoline)).as_value();
    {
        let result_ref = &mut result;
        let result_v = Value::new(unrooted, result_ref as *mut _ as *mut c_void).as_value();
        let frame_slice = Value::new(unrooted, frame as *mut _ as *mut c_void).as_value();
        let res = caller.call(unrooted, [cw, func, trampoline, result_v, frame_slice]);

        unrooted.enable_gc(enabled);

        match res {
            Ok(res) => {
                let res = res.data_ptr().cast::<jlrs_catch_t>().as_ref();
                match res.tag {
                    x if x == jlrs_catch_tag_t_JLRS_CATCH_OK => Ok(Ok(result.assume_init())),
                    x if x == jlrs_catch_tag_t_JLRS_CATCH_ERR => {
                        Err(Box::from_raw(res.error.cast()))
                    }
                    x if x == jlrs_catch_tag_t_JLRS_CATCH_EXCEPTION => Ok(Err(ValueRef::wrap(
                        NonNull::new_unchecked(res.error.cast()),
                    ))),
                    x if x == jlrs_catch_tag_t_JLRS_CATCH_PANIC => {
                        let err: Box<Box<dyn Any + Send>> = Box::from_raw(res.error.cast());
                        std::panic::resume_unwind(err)
                    }
                    _ => unreachable!(),
                }
            }
            Err(err) => Ok(Err(err)),
        }
    }
}

unsafe extern "C" fn trampoline<'frame, F: FnMut(&mut MaybeUninit<T>) -> JlrsResult<()>, T>(
    func: &mut F,
    _: *mut c_void,
    result: &mut MaybeUninit<T>,
) -> jlrs_catch_t {
    let res = catch_unwind(AssertUnwindSafe(|| func(result)));

    match res {
        Ok(Ok(())) => jlrs_catch_t {
            tag: jlrs_catch_tag_t_JLRS_CATCH_OK,
            error: null_mut(),
        },
        Ok(Err(e)) => jlrs_catch_t {
            tag: jlrs_catch_tag_t_JLRS_CATCH_ERR,
            error: Box::leak(e) as *mut _ as *mut _,
        },
        Err(e) => {
            // extra box because it's a fat pointer
            jlrs_catch_t {
                tag: jlrs_catch_tag_t_JLRS_CATCH_PANIC,
                error: Box::leak(Box::new(e)) as *mut _ as *mut _,
            }
        }
    }
}

fn trampoline_for<'frame, F: FnMut(&mut MaybeUninit<T>) -> JlrsResult<()>, T>(
    _: &mut F,
) -> Option<unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void) -> jlrs_catch_t> {
    unsafe {
        std::mem::transmute::<
            Option<unsafe extern "C" fn(&mut F, *mut c_void, &mut MaybeUninit<T>) -> jlrs_catch_t>,
            Option<unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void) -> jlrs_catch_t>,
        >(Some(trampoline::<F, T>))
    }
}

#[julia_version(windows_lts = false)]
pub(crate) unsafe fn catch_exceptions<'frame, 'borrow, 'data, G, T>(
    func: &'borrow mut G,
) -> JlrsResult<JuliaResultRef<'frame, 'data, T>>
where
    T: 'frame,
    G: FnMut(&mut MaybeUninit<T>) -> JlrsResult<()>,
{
    let trampoline = trampoline_for(func);
    let mut result = MaybeUninit::<T>::uninit();

    let res = jlrs_catch_wrapper(
        func as *mut _ as *mut _,
        trampoline,
        (&mut result) as *mut _ as *mut _,
        null_mut(),
    );

    match res.tag {
        x if x == jlrs_catch_tag_t_JLRS_CATCH_OK => Ok(Ok(result.assume_init())),
        x if x == jlrs_catch_tag_t_JLRS_CATCH_ERR => Err(Box::from_raw(res.error.cast())),
        x if x == jlrs_catch_tag_t_JLRS_CATCH_EXCEPTION => Ok(Err(ValueRef::wrap(
            NonNull::new_unchecked(res.error.cast()),
        ))),
        x if x == jlrs_catch_tag_t_JLRS_CATCH_PANIC => {
            let err: Box<Box<dyn Any + Send>> = Box::from_raw(res.error.cast());
            std::panic::resume_unwind(err)
        }
        _ => unreachable!(),
    }
}

#[julia_version(windows_lts = true)]
pub(crate) unsafe fn catch_exceptions<'frame, 'borrow, 'data, G, T>(
    func: &'borrow mut G,
) -> JlrsResult<JuliaResultRef<'frame, 'data, T>>
where
    T: 'frame,
    G: FnMut(&mut MaybeUninit<T>) -> JlrsResult<()>,
{
    let trampoline = trampoline_for(func);
    let mut result = MaybeUninit::<T>::uninit();

    let unrooted = Unrooted::new();
    let enabled = unrooted.gc_is_enabled();
    unrooted.enable_gc(false);

    let caller = Module::package_root_module(&unrooted, "Jlrs")
        .unwrap()
        .global(&unrooted, "call_catch_wrapper")
        .unwrap()
        .as_value();

    let func = Value::new(unrooted, func as *mut _ as *mut c_void).as_value();
    let cw = Value::new(unrooted, jlrs_catch_wrapper as *mut c_void).as_value();
    let trampoline =
        Value::new(unrooted, std::mem::transmute::<_, *mut c_void>(trampoline)).as_value();
    {
        let result_ref = &mut result;
        let result_v = Value::new(unrooted, result_ref as *mut _ as *mut c_void).as_value();
        let frame_slice = Value::new(unrooted, null_mut::<c_void>()).as_value();
        let res = caller.call(unrooted, [cw, func, trampoline, result_v, frame_slice]);

        unrooted.enable_gc(enabled);

        match res {
            Ok(res) => {
                let res = res.data_ptr().cast::<jlrs_catch_t>().as_ref();
                match res.tag {
                    x if x == jlrs_catch_tag_t_JLRS_CATCH_OK => Ok(Ok(result.assume_init())),
                    x if x == jlrs_catch_tag_t_JLRS_CATCH_ERR => {
                        Err(Box::from_raw(res.error.cast()))
                    }
                    x if x == jlrs_catch_tag_t_JLRS_CATCH_EXCEPTION => Ok(Err(ValueRef::wrap(
                        NonNull::new_unchecked(res.error.cast()),
                    ))),
                    x if x == jlrs_catch_tag_t_JLRS_CATCH_PANIC => {
                        let err: Box<Box<dyn Any + Send>> = Box::from_raw(res.error.cast());
                        std::panic::resume_unwind(err)
                    }
                    _ => unreachable!(),
                }
            }
            Err(err) => Ok(Err(err)),
        }
    }
}
