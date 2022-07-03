use crate::{error::JuliaResultRef, wrappers::ptr::value::ValueRef};
use jl_sys::{jlrs_catch_wrapper, jlrs_result_tag_t_JLRS_RESULT_ERR};
use std::{ffi::c_void, mem::MaybeUninit};

unsafe extern "C" fn trampoline<F: FnMut(&mut MaybeUninit<T>) -> (), T>(
    func: &mut F,
    result: &mut MaybeUninit<T>,
) {
    func(result);
}

fn trampoline_for<F: FnMut(&mut MaybeUninit<T>) -> (), T>(
    _: &mut F,
) -> Option<unsafe extern "C" fn(*mut c_void, *mut c_void) -> ()> {
    unsafe {
        std::mem::transmute::<
            Option<unsafe extern "C" fn(&mut F, &mut MaybeUninit<T>) -> ()>,
            Option<unsafe extern "C" fn(*mut c_void, *mut c_void) -> ()>,
        >(Some(trampoline::<F, T>))
    }
}

pub unsafe fn catch_exceptions<F, T>(func: &mut F) -> JuliaResultRef<T>
where
    F: FnMut(&mut MaybeUninit<T>) -> (),
{
    let trampoline = trampoline_for(func);
    let mut result = MaybeUninit::<T>::uninit();
    let mut error = ValueRef::undefined_ref();

    let tag = jlrs_catch_wrapper(
        func as *mut _ as *mut _,
        trampoline,
        (&mut result) as *mut _ as *mut _,
        (&mut error) as *mut _ as *mut _,
    );

    if tag == jlrs_result_tag_t_JLRS_RESULT_ERR {
        Err(error)
    } else {
        Ok(result.assume_init())
    }
}

#[cfg(test)]
#[cfg(feature = "sync-rt")]
mod test {
    use crate::{
        convert::into_jlrs_result::IntoJlrsResult,
        prelude::{Array, Wrapper},
        util,
    };

    use super::*;

    #[test]
    fn test() {
        util::JULIA.with(|julia| {
            let mut julia = julia.borrow_mut();
            julia
                .scope(|_global, mut frame| unsafe {
                    let mut data = vec![1u64, 2u64, 3u64, 4u64];
                    let arr =
                        Array::from_slice(&mut frame, &mut data, (2, 2))?.into_jlrs_result()?;

                    {
                        let arr = arr.as_value().assume_owned().cast_unchecked::<Array>();
                        let mut callback = |result: &mut MaybeUninit<()>| {
                            arr.grow_begin_unchecked(&mut frame, 5);
                            result.write(());
                        };

                        let res = catch_exceptions(&mut callback);
                        assert!(res.is_err());
                        assert!(!res.unwrap_err().is_undefined());
                    }

                    Ok(())
                })
                .ok();
        })
    }
}
