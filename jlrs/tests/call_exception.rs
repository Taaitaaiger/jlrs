use jlrs::prelude::*;
mod util;
use util::JULIA;

#[test]
fn call0() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.scope_with_slots(1, |global, frame| unsafe {
            let func = Module::main(global)
                .submodule_ref("JlrsTests")?
                .wrapper_unchecked()
                .function_ref("throws_exception")?
                .wrapper_unchecked();
            func.call0(&mut *frame)?.into_jlrs_result()?;
            Ok(())
        })
        .unwrap_err();
    });
}

#[test]
fn call0_dynamic() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.scope(|global, frame| unsafe {
            let func = Module::main(global)
                .submodule_ref("JlrsTests")?
                .wrapper_unchecked()
                .function_ref("throws_exception")?
                .wrapper_unchecked();
            func.call0(&mut *frame)?.into_jlrs_result()?;
            Ok(())
        })
        .unwrap_err();
    });
}

#[test]
fn call0_nested_as_unrooted() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.scope(|global, frame| {
            frame
                .result_scope(|output, frame| unsafe {
                    let func = Module::main(global)
                        .submodule_ref("JlrsTests")?
                        .wrapper_unchecked()
                        .function_ref("throws_exception")?
                        .wrapper_unchecked();
                    let res = func.call0(&mut *frame)?;

                    let os = output.into_scope(frame);
                    Ok(res.as_unrooted(os))
                })?
                .unwrap_err();

            Ok(())
        })
        .unwrap();
    });
}

#[test]
fn call1() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.scope_with_slots(2, |global, frame| unsafe {
            let angle = Value::new(&mut *frame, std::f32::consts::PI)?;
            let func = Module::main(global)
                .submodule_ref("JlrsTests")?
                .wrapper_unchecked()
                .function_ref("throws_exception")?
                .wrapper_unchecked();
            func.call1(&mut *frame, angle)?.into_jlrs_result()?;
            Ok(())
        })
        .unwrap_err();
    });
}

#[test]
fn call1_dynamic() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.scope(|global, frame| unsafe {
            let angle = Value::new(&mut *frame, std::f32::consts::PI)?;
            let func = Module::main(global)
                .submodule_ref("JlrsTests")?
                .wrapper_unchecked()
                .function_ref("throws_exception")?
                .wrapper_unchecked();
            func.call1(&mut *frame, angle)?.into_jlrs_result()?;
            Ok(())
        })
        .unwrap_err();
    });
}

#[test]
fn call2() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.scope_with_slots(3, |global, frame| unsafe {
            let angle = Value::new(&mut *frame, std::f32::consts::PI)?;
            let func = Module::main(global)
                .submodule_ref("JlrsTests")?
                .wrapper_unchecked()
                .function_ref("throws_exception")?
                .wrapper_unchecked();
            func.call2(&mut *frame, angle, angle)?.into_jlrs_result()?;
            Ok(())
        })
        .unwrap_err();
    });
}

#[test]
fn call2_dynamic() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.scope(|global, frame| unsafe {
            let angle = Value::new(&mut *frame, std::f32::consts::PI)?;
            let func = Module::main(global)
                .submodule_ref("JlrsTests")?
                .wrapper_unchecked()
                .function_ref("throws_exception")?
                .wrapper_unchecked();
            func.call2(&mut *frame, angle, angle)?.into_jlrs_result()?;
            Ok(())
        })
        .unwrap_err();
    });
}

#[test]
fn call3() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.scope_with_slots(4, |global, frame| unsafe {
            let angle = Value::new(&mut *frame, std::f32::consts::PI)?;
            let func = Module::main(global)
                .submodule_ref("JlrsTests")?
                .wrapper_unchecked()
                .function_ref("throws_exception")?
                .wrapper_unchecked();
            func.call3(&mut *frame, angle, angle, angle)?
                .into_jlrs_result()?;
            Ok(())
        })
        .unwrap_err();
    });
}

#[test]
fn call3_dynamic() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.scope(|global, frame| unsafe {
            let angle = Value::new(&mut *frame, std::f32::consts::PI)?;
            let func = Module::main(global)
                .submodule_ref("JlrsTests")?
                .wrapper_unchecked()
                .function_ref("throws_exception")?
                .wrapper_unchecked();
            func.call3(&mut *frame, angle, angle, angle)?
                .into_jlrs_result()?;
            Ok(())
        })
        .unwrap_err();
    });
}

#[test]
fn call() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.scope_with_slots(5, |global, frame| unsafe {
            let angle = Value::new(&mut *frame, std::f32::consts::PI)?;
            let func = Module::main(global)
                .submodule_ref("JlrsTests")?
                .wrapper_unchecked()
                .function_ref("throws_exception")?
                .wrapper_unchecked();
            func.call(&mut *frame, &mut [angle, angle, angle])?
                .into_jlrs_result()?;
            Ok(())
        })
        .unwrap_err();
    });
}

#[test]
fn call_output() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let out = jlrs.scope_with_slots(5, |global, frame| {
            frame
                .result_scope_with_slots(24, |output, frame| unsafe {
                    let func = Module::base(global).function_ref("+")?.wrapper_unchecked();
                    let arg0 = Value::new(&mut *frame, 1u32)?;
                    let arg1 = Value::new(&mut *frame, 2u32)?;
                    let arg2 = Value::new(&mut *frame, 3u32)?;
                    let arg3 = Value::new(&mut *frame, 4u32)?;
                    let output = output.into_scope(frame);
                    func.call(output, &mut [arg0, arg1, arg2, arg3])
                })?
                .unwrap()
                .unbox::<u32>()
        });

        assert_eq!(out.unwrap(), 10);
    });
}

#[test]
fn call_dynamic() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.scope(|global, frame| unsafe {
            let angle = Value::new(&mut *frame, std::f32::consts::PI)?;
            let func = Module::main(global)
                .submodule_ref("JlrsTests")?
                .wrapper_unchecked()
                .function_ref("throws_exception")?
                .wrapper_unchecked();
            func.call(&mut *frame, &mut [angle, angle, angle])?
                .into_jlrs_result()?;
            Ok(())
        })
        .unwrap_err();
    });
}

#[test]
fn call_dynamic_output() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let out = jlrs.scope(|global, frame| {
            frame
                .result_scope_with_slots(24, |output, frame| unsafe {
                    let func = Module::base(global).function_ref("+")?.wrapper_unchecked();
                    let arg0 = Value::new(&mut *frame, 1u32)?;
                    let arg1 = Value::new(&mut *frame, 2u32)?;
                    let arg2 = Value::new(&mut *frame, 3u32)?;
                    let arg3 = Value::new(&mut *frame, 4u32)?;
                    let output = output.into_scope(frame);
                    func.call(output, &mut [arg0, arg1, arg2, arg3])
                })?
                .unwrap()
                .unbox::<u32>()
        });

        assert_eq!(out.unwrap(), 10);
    });
}
