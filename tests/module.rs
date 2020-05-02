use jlrs::prelude::*;
mod util;
use util::JULIA;

#[test]
fn core_module() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.frame(0, |global, _| {
            let module = Module::core(global);
            let func = module.function("isa");
            let int64 = module.global("Float64");
            assert!(func.is_ok());
            assert!(int64.is_ok());
            Ok(())
        })
        .unwrap()
    });
}

#[test]
fn core_module_dynamic() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.dynamic_frame(|global, _| {
            let module = Module::core(global);
            let func = module.function("isa");
            assert!(func.is_ok());
            Ok(())
        })
        .unwrap()
    });
}

#[test]
fn base_module() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.frame(0, |global, _| {
            let module = Module::base(global);
            let func = module.function("+");
            let int64 = module.global("pi");
            assert!(func.is_ok());
            assert!(int64.is_ok());
            Ok(())
        })
        .unwrap()
    });
}

#[test]
fn base_module_dynamic() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.dynamic_frame(|global, _| {
            let module = Module::base(global);
            let func = module.function("+");
            assert!(func.is_ok());
            Ok(())
        })
        .unwrap()
    });
}

#[test]
fn main_module() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.frame(0, |global, _| {
            let main_module = Module::main(global);
            let jlrs_module = main_module.submodule("Jlrs");
            assert!(jlrs_module.is_ok());
            let func = jlrs_module.unwrap().function("attachstacktrace");
            assert!(func.is_ok());
            Ok(())
        })
        .unwrap()
    });
}

#[test]
fn main_module_dynamic() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.dynamic_frame(|global, _| {
            let main_module = Module::main(global);
            let jlrs_module = main_module.submodule("Jlrs");
            assert!(jlrs_module.is_ok());
            let func = jlrs_module.unwrap().function("attachstacktrace");
            assert!(func.is_ok());
            Ok(())
        })
        .unwrap()
    });
}

#[test]
fn error_nonexistent_function() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.frame(0, |global, _| {
            assert!(Module::base(global).function("foo").is_err());
            Ok(())
        })
        .unwrap()
    });
}

#[test]
fn error_nonexistent_function_dynamic() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.dynamic_frame(|global, _| {
            assert!(Module::base(global).function("foo").is_err());
            Ok(())
        })
        .unwrap()
    });
}

#[test]
fn error_nonexistent_submodule() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.frame(0, |global, _| {
            assert!(Module::base(global).submodule("foo").is_err());
            Ok(())
        })
        .unwrap()
    });
}

#[test]
fn error_nonexistent_submodule_dynamic() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.dynamic_frame(|global, _| {
            assert!(Module::base(global).submodule("foo").is_err());
            Ok(())
        })
        .unwrap()
    });
}
