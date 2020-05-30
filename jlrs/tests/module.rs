use jlrs::prelude::*;
use jlrs::util::JULIA;
use std::borrow::Cow;

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

#[test]
fn function_returns_module() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.frame(1, |global, frame| {
            let base = Module::main(global)
                .submodule("JlrsTests")?
                .function("base")?;
            let base_val = base.call0(frame)?.unwrap();

            assert!(base_val.is::<Module>());
            assert!(base_val.cast::<Module>().is_ok());
            assert!(base_val.cast::<Symbol>().is_err());

            Ok(())
        })
        .unwrap();
    })
}

#[test]
fn use_string_for_access() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.frame(1, |global, _frame| {
            assert!(Module::main(global)
                .submodule("JlrsTests".to_string())
                .is_ok());

            Ok(())
        })
        .unwrap();
    })
}

#[test]
fn use_cow_for_access() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.frame(1, |global, _frame| {
            assert!(Module::main(global)
                .submodule(Cow::from("JlrsTests"))
                .is_ok());

            Ok(())
        })
        .unwrap();
    })
}

struct MyString(String);
impl AsRef<str> for MyString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[test]
fn use_dyn_str_for_access() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.frame(1, |global, _frame| {
            let name = MyString("JlrsTests".to_string());
            assert!(Module::main(global)
                .submodule(&name as &dyn AsRef<str>)
                .is_ok());

            Ok(())
        })
        .unwrap();
    })
}
