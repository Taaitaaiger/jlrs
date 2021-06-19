use jlrs::prelude::*;
use jlrs::util::JULIA;

unsafe extern "C" fn uses_null_scope(array: TypedArray<f64>) -> bool {
    let mut ccall = CCall::new();

    let out = ccall.null_scope(|frame| {
        let borrowed = array.inline_data(&mut *frame)?;
        Ok(borrowed[1] == 1.0)
    });

    if let Ok(o) = out {
        o
    } else {
        false
    }
}

#[test]
fn ccall_with_array() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.scope(|global, frame| unsafe {
            let fn_ptr = Value::new(&mut *frame, uses_null_scope as *mut std::ffi::c_void)?;
            let mut arr_data = vec![0.0f64, 1.0f64];
            let arr = Array::from_slice(&mut *frame, &mut arr_data, 2)?;
            let func = Module::main(global)
                .submodule_ref("JlrsTests")?
                .wrapper_unchecked()
                .function_ref("callrustwitharr")?
                .wrapper_unchecked();

            let out = func.call2(&mut *frame, fn_ptr, arr)?.unwrap();
            let ok = out.unbox::<bool>()?.as_bool();
            assert!(ok);
            Ok(())
        })
        .unwrap()
    })
}
