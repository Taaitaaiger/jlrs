mod util;
#[cfg(all(not(feature = "lts"), feature = "sync-rt", feature = "internal-types"))]
mod not_lts {
    use super::util::JULIA;
    use jlrs::prelude::*;
    use jlrs::wrappers::ptr::internal::vararg::Vararg;
    use jlrs::{layout::valid_layout::ValidLayout, wrappers::ptr::internal::vararg::VarargRef};

    #[test]
    fn access_vararg() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let vararg =
                        Value::eval_string(&mut frame, "Vararg{Int32}").into_jlrs_result()?;

                    assert!(vararg.is::<Vararg>());
                    assert!(VarargRef::valid_layout(
                        vararg.as_value().datatype().as_value()
                    ));

                    let vararg = vararg.cast::<Vararg>()?;
                    assert_eq!(
                        vararg.t().unwrap().value().cast::<DataType>()?,
                        DataType::int32_type(&frame)
                    );
                    assert!(vararg.n().is_none());
                    Ok(())
                })
                .unwrap();
        });
    }

    #[test]
    fn create_emtpy_vararg() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let vararg_type = DataType::vararg_type(&frame);
                    let instance = vararg_type
                        .instantiate(&mut frame, [])?
                        .into_jlrs_result()?;

                    assert!(instance.is::<Vararg>());

                    let vararg = instance.cast::<Vararg>()?;
                    assert!(vararg.t().is_none());
                    assert!(vararg.n().is_none());
                    Ok(())
                })
                .unwrap();
        });
    }

    #[test]
    fn create_typed_vararg() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let vararg_type = DataType::vararg_type(&frame);
                    let args = [DataType::int32_type(&frame).as_value()];
                    let instance = vararg_type
                        .instantiate(&mut frame, args)?
                        .into_jlrs_result()?;

                    assert!(instance.is::<Vararg>());

                    let vararg = instance.cast::<Vararg>()?;
                    assert_eq!(
                        vararg.t().unwrap().value().cast::<DataType>()?,
                        DataType::int32_type(&frame)
                    );
                    assert!(vararg.n().is_none());
                    Ok(())
                })
                .unwrap();
        });
    }

    #[test]
    fn create_typed_and_sized_vararg() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let vararg_type = DataType::vararg_type(&frame);
                    let n = Value::new(&mut frame, 3isize);
                    let args = [DataType::int32_type(&frame).as_value(), n];
                    let instance = vararg_type
                        .instantiate(&mut frame, args)?
                        .into_jlrs_result()?;

                    assert!(instance.is::<Vararg>());

                    let vararg = instance.cast::<Vararg>()?;
                    assert_eq!(
                        vararg.t().unwrap().value().cast::<DataType>()?,
                        DataType::int32_type(&frame)
                    );
                    assert_eq!(vararg.n().unwrap().value().unbox::<isize>()?, 3);
                    Ok(())
                })
                .unwrap();
        });
    }
}
