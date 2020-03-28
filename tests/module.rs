/*use jlrs::prelude::*;

#[test]
fn use_core_module() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    jlrs.session(|session| {
        let module = session.core_module();
        let func = module.function("isa").unwrap();
        let val = session.new_primitive(1f64)?;
        let int64 = module.global("Float64").unwrap();
        let out = session.new_unassigned()?;

        let res = session.execute(|exec_ctx| {
            let out = func.call2(exec_ctx, out, val, int64)?;
            exec_ctx.try_unbox::<bool>(&out)
        })?;

        assert_eq!(res, true);
        Ok(())
    })
    .unwrap()
}

#[test]
fn use_core_module_from_context() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    jlrs.session(|session| {
        let val = session.new_primitive(1f64)?;
        let out = session.new_unassigned()?;

        let res = session.execute(|exec_ctx| {
            let module = exec_ctx.core_module();
            let int64 = module.global("Float64").unwrap();
            let func = module.function("isa").unwrap();
            let out = func.call2(exec_ctx, out, val, int64)?;
            exec_ctx.try_unbox::<bool>(&out)
        })?;

        assert_eq!(res, true);
        Ok(())
    })
    .unwrap()
}

#[test]
fn use_base_module() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    jlrs.session(|session| {
        let module = session.base_module();
        let func = module.function("+").unwrap();
        let val = session.new_primitive(1f64)?;
        let pi = module.global("pi").unwrap();
        let out = session.new_unassigned()?;

        let res = session.execute(|exec_ctx| {
            let out = func.call2(exec_ctx, out, val, pi)?;
            exec_ctx.try_unbox::<f64>(&out)
        })?;

        assert_eq!(res, 1.0 + std::f64::consts::PI);
        Ok(())
    })
    .unwrap()
}

#[test]
fn use_base_module_from_context() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    jlrs.session(|session| {
        let out = session.new_unassigned()?;
        let val = session.new_primitive(1f64)?;

        let res = session.execute(|exec_ctx| {
            let module = exec_ctx.base_module();
            let func = module.function("+").unwrap();
            let pi = module.global("pi").unwrap();
            let out = func.call2(exec_ctx, out, val, pi)?;
            exec_ctx.try_unbox::<f64>(&out)
        })?;

        assert_eq!(res, 1.0 + std::f64::consts::PI);
        Ok(())
    })
    .unwrap()
}

#[test]
fn use_main_module() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    jlrs.session(|session| {
        let base_module = session.base_module();
        let getindex = base_module.function("getindex").unwrap();
        let main_module = session.main_module();
        let jlrs_module = main_module.submodule("Jlrs")?;
        let arraydims = jlrs_module.function("arraydims").unwrap();

        let vals = session.new_primitives([1u64, 2u64])?;
        let idx = session.new_primitive(1u64)?;
        let out1 = session.new_unassigned()?;
        let out2 = session.new_unassigned()?;

        let res = session.execute(|exec_ctx| {
            let tuple = arraydims.call_primitives(exec_ctx, out1, vals)?;
            let val = getindex.call2(exec_ctx, out2, tuple, idx)?;
            exec_ctx.try_unbox::<u64>(&val)
        })?;

        assert_eq!(res, 1);
        Ok(())
    })
    .unwrap()
}

#[test]
fn use_main_module_from_context() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    jlrs.session(|session| {
        let vals = session.new_primitives([1u64, 2u64])?;
        let idx = session.new_primitive(1u64)?;
        let out1 = session.new_unassigned()?;
        let out2 = session.new_unassigned()?;

        let res = session.execute(|exec_ctx| {
            let base_module = exec_ctx.base_module();
            let getindex = base_module.function("getindex").unwrap();
            let main_module = exec_ctx.main_module();
            let jlrs_module = main_module.submodule("Jlrs")?;
            let arraydims = jlrs_module.function("arraydims").unwrap();
            let tuple = arraydims.call_primitives(exec_ctx, out1, vals)?;
            let val = getindex.call2(exec_ctx, out2, tuple, idx)?;
            exec_ctx.try_unbox::<u64>(&val)
        })?;

        assert_eq!(res, 1);
        Ok(())
    })
    .unwrap()
}

#[test]
fn error_nonexistent_function() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    jlrs.session(|session| {
        assert!(session.base_module().function("foo").is_err());
        Ok(())
    })
    .unwrap()
}

#[test]
fn error_nonexistent_submodule() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    jlrs.session(|session| {
        assert!(session.base_module().submodule("Foo").is_err());
        Ok(())
    })
    .unwrap()
}
*/