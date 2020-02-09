use jlrs::prelude::*;

#[test]
fn call0() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    jlrs.session(|session| {
        let module = session.base_module();
        let func = module.function("vect").unwrap();
        let out = session.new_unassigned()?;

        session.execute(|exec_ctx| func.call0(exec_ctx, out))?;

        Ok(())
    })
    .unwrap();
}

#[test]
fn call1() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    jlrs.session(|session| {
        let module = session.base_module();
        let func = module.function("cos").unwrap();
        let p1 = session.new_primitive(std::f32::consts::PI)?;
        let out = session.new_unassigned()?;

        let res = session.execute(|exec_ctx| {
            let out = func.call1(exec_ctx, out, p1)?;
            exec_ctx.try_unbox::<f32>(&out)
        })?;

        assert_eq!(res, -1.);
        Ok(())
    })
    .unwrap()
}

#[test]
fn call2() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    jlrs.session(|session| {
        let module = session.base_module();
        let func = module.function("+").unwrap();
        let p1 = session.new_primitive(1u8)?;
        let p2 = session.new_primitive(2u8)?;
        let out = session.new_unassigned()?;

        let res = session.execute(|exec_ctx| {
            let out = func.call2(exec_ctx, out, p1, p2)?;
            exec_ctx.try_unbox::<u8>(&out)
        })?;

        assert_eq!(res, 3);
        Ok(())
    })
    .unwrap()
}

#[test]
fn call3() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    jlrs.session(|session| {
        let module = session.base_module();
        let func = module.function("+").unwrap();
        let p1 = session.new_primitive(1u8)?;
        let p2 = session.new_primitive(2u8)?;
        let p3 = session.new_primitive(3u8)?;
        let out = session.new_unassigned()?;

        let res = session.execute(|exec_ctx| {
            let out = func.call3(exec_ctx, out, p1, p2, p3)?;
            exec_ctx.try_unbox::<u8>(&out)
        })?;

        assert_eq!(res, 6);
        Ok(())
    })
    .unwrap()
}

#[test]
fn call_primitives() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    jlrs.session(|session| {
        let module = session.base_module();
        let func = module.function("+").unwrap();
        let ps = session.new_primitives([1u8, 2u8, 3u8, 4u8])?;
        let out = session.new_unassigned()?;

        let res = session.execute(|exec_ctx| {
            let out = func.call_primitives(exec_ctx, out, ps)?;
            exec_ctx.try_unbox::<u8>(&out)
        })?;

        assert_eq!(res, 10);
        Ok(())
    })
    .unwrap()
}

#[test]
fn call_dyn() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    jlrs.session(|session| {
        let module = session.base_module();
        let func = module.function("+").unwrap();
        let p1 = session.new_primitive(1u8)?;
        let p2 = session.new_primitive(2u16)?;
        let p3 = session.new_primitive(3u32)?;
        let out = session.new_unassigned()?;

        let res = session.execute(|exec_ctx| {
            let out = func.call_dyn(exec_ctx, out, [p1.as_dyn(), p2.as_dyn(), p3.as_dyn()])?;
            exec_ctx.try_unbox::<u32>(&out)
        })?;

        assert_eq!(res, 6);
        Ok(())
    })
    .unwrap()
}

#[test]
fn call() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    let res = jlrs
        .session(|session| {
            let func = session.base_module().function("+")?;
            let p1 = session.new_primitive(1u8)?;
            let p2 = session.new_primitive(2u8)?;
            let p3 = session.new_primitive(3u8)?;
            let p4 = session.new_primitive(4u8)?;
            let out = session.new_unassigned()?;

            session.execute(|exec_ctx| {
                let out = func.call(exec_ctx, out, [p1, p2, p3, p4])?;
                exec_ctx.try_unbox::<u8>(&out)
            })
        })
        .unwrap();
    assert_eq!(res, 10);
}
