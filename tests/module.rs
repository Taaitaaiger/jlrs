use jlrs::prelude::*;

#[test]
fn core_module() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    jlrs.frame(0, |frame| {
        let module = Module::core(frame);
        let func = module.function("isa");
        let int64 = module.global("Float64");
        assert!(func.is_ok());
        assert!(int64.is_ok());
        Ok(())
    })
    .unwrap()
}

#[test]
fn core_module_dynamic() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    jlrs.dynamic_frame(|frame| {
        let module = Module::core(frame);
        let func = module.function("isa");
        assert!(func.is_ok());
        Ok(())
    })
    .unwrap()
}

#[test]
fn base_module() {
let mut jlrs = unsafe { Runtime::testing_instance() };

    jlrs.frame(0, |frame| {
        let module = Module::base(frame);
        let func = module.function("+");
        let int64 = module.global("pi");
        assert!(func.is_ok());
        assert!(int64.is_ok());
        Ok(())
    })
    .unwrap()
}

#[test]
fn base_module_dynamic() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    jlrs.dynamic_frame(|frame| {
        let module = Module::base(frame);
        let func = module.function("+");
        assert!(func.is_ok());
        Ok(())
    })
    .unwrap()
}

#[test]
fn main_module() {
let mut jlrs = unsafe { Runtime::testing_instance() };

    jlrs.frame(0, |frame| {
        let main_module = Module::main(frame);
        let jlrs_module = main_module.submodule("Jlrs");
        assert!(jlrs_module.is_ok());
        let func = jlrs_module.unwrap().function("arraydims");
        assert!(func.is_ok());
        Ok(())
    })
    .unwrap()
}

#[test]
fn main_module_dynamic() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    jlrs.dynamic_frame(|frame| {
        let main_module = Module::main(frame);
        let jlrs_module = main_module.submodule("Jlrs");
        assert!(jlrs_module.is_ok());
        let func = jlrs_module.unwrap().function("arraydims");
        assert!(func.is_ok());
        Ok(())
    })
    .unwrap()
}

#[test]
fn error_nonexistent_function() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    jlrs.frame(0, |frame| {
        assert!(Module::base(frame).function("foo").is_err());
        Ok(())
    })
    .unwrap()
}

#[test]
fn error_nonexistent_function_dynamic() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    jlrs.dynamic_frame(|frame| {
        assert!(Module::base(frame).function("foo").is_err());
        Ok(())
    })
    .unwrap()
}

#[test]
fn error_nonexistent_submodule() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    jlrs.frame(0, |frame| {
        assert!(Module::base(frame).submodule("foo").is_err());
        Ok(())
    })
    .unwrap()
}

#[test]
fn error_nonexistent_submodule_dynamic() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    jlrs.dynamic_frame(|frame| {
        assert!(Module::base(frame).submodule("foo").is_err());
        Ok(())
    })
    .unwrap()
}
