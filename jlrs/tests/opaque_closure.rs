use jlrs::layout::valid_layout::ValidLayout;
use jlrs::prelude::*;
use jlrs::util::JULIA;
use jlrs::wrappers::ptr::opaque_closure::OpaqueClosure;

#[test]
fn create_opaque_closure() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.scope_with_slots(1, |_, frame| unsafe {
            let closure =
                Value::eval_string(&mut *frame, "Base.Experimental.@opaque (x::Int64) -> 2x")?
                    .into_jlrs_result()?;

            assert!(closure.is::<OpaqueClosure>());
            assert!(OpaqueClosure::valid_layout(
                closure.as_value().datatype().as_value()
            ));
            Ok(())
        })
        .unwrap();
    });
}

#[test]
fn call_opaque_closure() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.scope_with_slots(3, |_, frame| unsafe {
            let closure =
                Value::eval_string(&mut *frame, "Base.Experimental.@opaque (x::Int64) -> 2x")?
                    .into_jlrs_result()?
                    .cast::<OpaqueClosure>()?;

            let arg = Value::new(&mut *frame, 3isize)?;
            let res = closure
                .call1(&mut *frame, arg)?
                .into_jlrs_result()?
                .unbox::<isize>()?;

            assert_eq!(res, 6);
            Ok(())
        })
        .unwrap();
    });
}

#[test]
fn call_opaque_closure_wrong_argtype() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.scope_with_slots(3, |_, frame| unsafe {
            let closure =
                Value::eval_string(&mut *frame, "Base.Experimental.@opaque (x::Int64) -> 2x")?
                    .into_jlrs_result()?
                    .cast::<OpaqueClosure>()?;

            let arg = Value::new(&mut *frame, 3usize)?;
            let res = closure.call1(&mut *frame, arg)?;

            assert!(res.is_err());
            Ok(())
        })
        .unwrap();
    });
}

#[test]
fn call_opaque_closure_wrong_n_args() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.scope_with_slots(3, |_, frame| unsafe {
            let closure =
                Value::eval_string(&mut *frame, "Base.Experimental.@opaque (x::Int64) -> 2x")?
                    .into_jlrs_result()?
                    .cast::<OpaqueClosure>()?;

            let arg = Value::new(&mut *frame, 3isize)?;
            let res = closure.call2(&mut *frame, arg, arg)?;

            assert!(res.is_err());
            Ok(())
        })
        .unwrap();
    });
}

#[test]
fn call_vararg_opaque_closure_2args() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.scope_with_slots(3, |_, frame| unsafe {
            let closure = Value::eval_string(
                &mut *frame,
                "Base.Experimental.@opaque (x::Int64, y::Int64...) -> 2x + sum(y)",
            )?
            .into_jlrs_result()?
            .cast::<OpaqueClosure>()?;

            let arg = Value::new(&mut *frame, 3isize)?;
            let res = closure
                .call2(&mut *frame, arg, arg)?
                .into_jlrs_result()?
                .unbox::<isize>()?;

            assert_eq!(res, 9);
            Ok(())
        })
        .unwrap();
    });
}
