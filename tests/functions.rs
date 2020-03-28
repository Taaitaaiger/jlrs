use jlrs::prelude::*;

#[test]
fn call0() {
    let mut jlrs = unsafe { Runtime::testing_instance() };
    
    jlrs.frame(1, |frame| {
        let func = Module::base(frame).function("vect")?;
        func.call0(frame)?;
        Ok(())
    })
    .unwrap();
}

#[test]
fn call0_dynamic() {
    let mut jlrs = unsafe { Runtime::testing_instance() };
    
    jlrs.dynamic_frame(|frame| {
        let func = Module::base(frame).function("vect")?;
        func.call0(frame)?;
        Ok(())
    })
    .unwrap();
}

#[test]
fn call1() {
    let mut jlrs = unsafe { Runtime::testing_instance() };
    
    let out = jlrs.frame(2, |frame| {
        let func = Module::base(frame).function("cos")?;
        let angle = Value::new(frame, std::f32::consts::PI)?;
        let out = func.call1(frame, angle)?;
        out.try_unbox::<f32>()
    });
    
    assert_eq!(out.unwrap(), -1.);
}

#[test]
fn call1_dynamic() {
    let mut jlrs = unsafe { Runtime::testing_instance() };
    
    let out = jlrs.dynamic_frame(|frame| {
        let func = Module::base(frame).function("cos")?;
        let angle = Value::new(frame, std::f32::consts::PI)?;
        let out = func.call1(frame, angle)?;
        out.try_unbox::<f32>()
    });
    
    assert_eq!(out.unwrap(), -1.);
}

#[test]
fn call2() {
    let mut jlrs = unsafe { Runtime::testing_instance() };
    
    let out = jlrs.frame(3, |frame| {
        let func = Module::base(frame).function("+")?;
        let arg0 = Value::new(frame, 1u32)?;
        let arg1 = Value::new(frame, 2u32)?;
        let out = func.call2(frame, arg0, arg1)?;
        out.try_unbox::<u32>()
    });
    
    assert_eq!(out.unwrap(), 3);
}

#[test]
fn call2_dynamic() {
    let mut jlrs = unsafe { Runtime::testing_instance() };
    
    let out = jlrs.dynamic_frame(|frame| {
        let func = Module::base(frame).function("+")?;
        let arg0 = Value::new(frame, 1u32)?;
        let arg1 = Value::new(frame, 2u32)?;
        let out = func.call2(frame, arg0, arg1)?;
        out.try_unbox::<u32>()
    });
    
    assert_eq!(out.unwrap(), 3);
}

#[test]
fn call3() {
    let mut jlrs = unsafe { Runtime::testing_instance() };
    
    let out = jlrs.frame(4, |frame| {
        let func = Module::base(frame).function("+")?;
        let arg0 = Value::new(frame, 1u32)?;
        let arg1 = Value::new(frame, 2u32)?;
        let arg2 = Value::new(frame, 3u32)?;
        let out = func.call3(frame, arg0, arg1, arg2)?;
        out.try_unbox::<u32>()
    });
    
    assert_eq!(out.unwrap(), 6);
}

#[test]
fn call3_dynamic() {
    let mut jlrs = unsafe { Runtime::testing_instance() };
    
    let out = jlrs.dynamic_frame(|frame| {
        let func = Module::base(frame).function("+")?;
        let arg0 = Value::new(frame, 1u32)?;
        let arg1 = Value::new(frame, 2u32)?;
        let arg2 = Value::new(frame, 3u32)?;
        let out = func.call3(frame, arg0, arg1, arg2)?;
        out.try_unbox::<u32>()
    });
    
    assert_eq!(out.unwrap(), 6);
}

#[test]
fn call() {
    let mut jlrs = unsafe { Runtime::testing_instance() };
    
    let out = jlrs.frame(5, |frame| {
        let func = Module::base(frame).function("+")?;
        let arg0 = Value::new(frame, 1u32)?;
        let arg1 = Value::new(frame, 2u32)?;
        let arg2 = Value::new(frame, 3u32)?;
        let arg3 = Value::new(frame, 4u32)?;
        let out = func.call(frame, [arg0, arg1, arg2, arg3])?;
        out.try_unbox::<u32>()
    });
    
    assert_eq!(out.unwrap(), 10);
}

#[test]
fn call_dynamic() {
    let mut jlrs = unsafe { Runtime::testing_instance() };
    
    let out = jlrs.dynamic_frame(|frame| {
        let func = Module::base(frame).function("+")?;
        let arg0 = Value::new(frame, 1u32)?;
        let arg1 = Value::new(frame, 2u32)?;
        let arg2 = Value::new(frame, 3u32)?;
        let arg3 = Value::new(frame, 4u32)?;
        let out = func.call(frame, [arg0, arg1, arg2, arg3])?;
        out.try_unbox::<u32>()
    });
    
    assert_eq!(out.unwrap(), 10);
}
