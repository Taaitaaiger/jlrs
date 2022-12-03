mod util;
#[cfg(all(
    not(feature = "julia-1-6"),
    feature = "sync-rt",
    feature = "internal-types"
))]
mod not_lts {
    use jlrs::{
        layout::valid_layout::ValidLayout,
        prelude::*,
        wrappers::ptr::internal::vararg::{Vararg, VarargRef},
    };

    use super::util::JULIA;

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
                        vararg.t(&frame).unwrap().value().cast::<DataType>()?,
                        DataType::int32_type(&frame)
                    );
                    assert!(vararg.n(&frame).is_none());
                    Ok(())
                })
                .unwrap();
        });
    }

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
                    assert!(vararg.t(&frame).is_none());
                    assert!(vararg.n(&frame).is_none());
                    Ok(())
                })
                .unwrap();
        });
    }

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
                        vararg.t(&frame).unwrap().value().cast::<DataType>()?,
                        DataType::int32_type(&frame)
                    );
                    assert!(vararg.n(&frame).is_none());
                    Ok(())
                })
                .unwrap();
        });
    }

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
                        vararg.t(&frame).unwrap().value().cast::<DataType>()?,
                        DataType::int32_type(&frame)
                    );
                    assert_eq!(vararg.n(&frame).unwrap().value().unbox::<isize>()?, 3);
                    Ok(())
                })
                .unwrap();
        });
    }

    #[test]
    fn vararg_tests() {
        access_vararg();
        create_emtpy_vararg();
        create_typed_vararg();
        create_typed_and_sized_vararg();
    }
}
