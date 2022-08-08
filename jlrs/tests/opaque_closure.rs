mod util;
#[cfg(all(not(feature = "lts"), feature = "sync-rt", feature = "internal-types"))]
mod not_lts {
    use super::util::JULIA;
    use jlrs::prelude::*;
    use jlrs::wrappers::ptr::internal::opaque_closure::OpaqueClosure;
    use jlrs::{
        layout::valid_layout::ValidLayout,
        wrappers::ptr::internal::opaque_closure::OpaqueClosureRef,
    };

    #[test]
    fn create_opaque_closure() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |_, mut frame| unsafe {
                let closure =
                    Value::eval_string(&mut frame, "Base.Experimental.@opaque (x::Int64) -> 2x")?
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

    #[test]
    fn call_opaque_closure() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(3, |_, mut frame| unsafe {
                let closure =
                    Value::eval_string(&mut frame, "Base.Experimental.@opaque (x::Int64) -> 2x")?
                        .into_jlrs_result()?
                        .cast::<OpaqueClosure>()?;

                let arg = Value::new(&mut frame, 3i64)?;
                let res = closure
                    .call1(&mut frame, arg)?
                    .into_jlrs_result()?
                    .unbox::<i64>()?;

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

            jlrs.scope_with_capacity(3, |_, mut frame| unsafe {
                let closure =
                    Value::eval_string(&mut frame, "Base.Experimental.@opaque (x::Int64) -> 2x")?
                        .into_jlrs_result()?
                        .cast::<OpaqueClosure>()?;

                let arg = Value::new(&mut frame, 3usize)?;
                let res = closure.call1(&mut frame, arg)?;

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

            jlrs.scope_with_capacity(3, |_, mut frame| unsafe {
                let closure =
                    Value::eval_string(&mut frame, "Base.Experimental.@opaque (x::Int64) -> 2x")?
                        .into_jlrs_result()?
                        .cast::<OpaqueClosure>()?;

                let arg = Value::new(&mut frame, 3i64)?;
                let res = closure.call2(&mut frame, arg, arg)?;

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

            jlrs.scope_with_capacity(3, |_, mut frame| unsafe {
                let closure = Value::eval_string(
                    &mut frame,
                    "Base.Experimental.@opaque (x::Int64, y::Int64...) -> 2x + sum(y)",
                )?
                .into_jlrs_result()?
                .cast::<OpaqueClosure>()?;

                let arg = Value::new(&mut frame, 3i64)?;
                let res = closure
                    .call2(&mut frame, arg, arg)?
                    .into_jlrs_result()?
                    .unbox::<i64>()?;

                assert_eq!(res, 9);
                Ok(())
            })
            .unwrap();
        });
    }
}
