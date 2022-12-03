mod util;
#[cfg(all(
    not(feature = "julia-1-6"),
    feature = "sync-rt",
    feature = "internal-types"
))]
mod not_lts {
    use jlrs::{
        data::managed::internal::opaque_closure::{OpaqueClosure, OpaqueClosureRef},
        layout::valid_layout::ValidLayout,
        prelude::*,
    };

    use super::util::JULIA;

    fn create_opaque_closure() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let closure = Value::eval_string(
                        &mut frame,
                        "Base.Experimental.@opaque (x::Int64) -> 2x",
                    )
                    .into_jlrs_result()?;

                    assert!(closure.is::<OpaqueClosure>());
                    assert!(OpaqueClosureRef::valid_layout(
                        closure.as_value().datatype().as_value()
                    ));
                    Ok(())
                })
                .unwrap();
        });
    }

    fn call_opaque_closure() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let closure = Value::eval_string(
                        &mut frame,
                        "Base.Experimental.@opaque (x::Int64) -> 2x",
                    )
                    .into_jlrs_result()?
                    .cast::<OpaqueClosure>()?;

                    let arg = Value::new(&mut frame, 3i64);
                    let res = closure
                        .call1(&mut frame, arg)
                        .into_jlrs_result()?
                        .unbox::<i64>()?;

                    assert_eq!(res, 6);
                    Ok(())
                })
                .unwrap();
        });
    }

    fn call_opaque_closure_wrong_argtype() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let closure = Value::eval_string(
                        &mut frame,
                        "Base.Experimental.@opaque (x::Int64) -> 2x",
                    )
                    .into_jlrs_result()?
                    .cast::<OpaqueClosure>()?;

                    let arg = Value::new(&mut frame, 3usize);
                    let res = closure.call1(&mut frame, arg);

                    assert!(res.is_err());
                    Ok(())
                })
                .unwrap();
        });
    }

    fn call_opaque_closure_wrong_n_args() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let closure = Value::eval_string(
                        &mut frame,
                        "Base.Experimental.@opaque (x::Int64) -> 2x",
                    )
                    .into_jlrs_result()?
                    .cast::<OpaqueClosure>()?;

                    let arg = Value::new(&mut frame, 3i64);
                    let res = closure.call2(&mut frame, arg, arg);

                    assert!(res.is_err());
                    Ok(())
                })
                .unwrap();
        });
    }

    fn call_vararg_opaque_closure_2args() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let closure = Value::eval_string(
                        &mut frame,
                        "Base.Experimental.@opaque (x::Int64, y::Int64...) -> 2x + sum(y)",
                    )
                    .into_jlrs_result()?
                    .cast::<OpaqueClosure>()?;

                    let arg = Value::new(&mut frame, 3i64);
                    let res = closure
                        .call2(&mut frame, arg, arg)
                        .into_jlrs_result()?
                        .unbox::<i64>()?;

                    assert_eq!(res, 9);
                    Ok(())
                })
                .unwrap();
        });
    }

    #[test]
    fn opaque_closure_tests() {
        create_opaque_closure();
        call_opaque_closure();
        call_opaque_closure_wrong_argtype();
        call_opaque_closure_wrong_n_args();
        call_vararg_opaque_closure_2args();
    }
}
