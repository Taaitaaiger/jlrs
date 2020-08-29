use jlrs::prelude::*;
use jlrs::util::JULIA;

unsafe extern "C" fn uses_null_frame(array: TypedArray<f64>) -> bool {
    let mut ccall = CCall::new(0);

    let out = ccall.null_frame(|frame| {
        let borrowed = array.inline_data(frame)?;
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

        jlrs.dynamic_frame(|global, frame| {
            let fn_ptr = Value::new(frame, uses_null_frame as *mut std::ffi::c_void)?;
            let mut arr_data = vec![0.0f64, 1.0f64];
            let arr = Value::borrow_array(frame, &mut arr_data, 2)?;
            let func = Module::main(global)
                .submodule("JlrsTests")?
                .function("callrustwitharr")?;

            let out = func.call2(frame, fn_ptr, arr)?.unwrap();
            let ok = out.cast::<bool>()?;
            assert!(ok);
            Ok(())
        })
        .unwrap()
    })
}
