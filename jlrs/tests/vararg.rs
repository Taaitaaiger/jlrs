mod util;
#[cfg(all(not(feature = "lts"), feature = "sync-rt", feature = "internal-types"))]
mod not_lts {
    use super::util::JULIA;
    use jlrs::prelude::*;
    use jlrs::wrappers::ptr::internal::vararg::Vararg;
    use jlrs::{layout::valid_layout::ValidLayout, wrappers::ptr::VarargRef};

    #[test]
    fn access_vararg() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |global, mut frame| unsafe {
                let vararg = Value::eval_string(&mut frame, "Vararg{Int32}")?.into_jlrs_result()?;

                assert!(vararg.is::<Vararg>());
                assert!(VarargRef::valid_layout(
                    vararg.as_value().datatype().as_value()
                ));

                let vararg = vararg.cast::<Vararg>()?;
                assert_eq!(
                    vararg.t().value_unchecked().cast::<DataType>()?,
                    DataType::int32_type(global)
                );
                assert!(vararg.n().is_undefined());
                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn create_emtpy_vararg() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |global, mut frame| {
                let vararg_type = DataType::vararg_type(global);
                let instance = vararg_type
                    .instantiate(&mut frame, [])?
                    .into_jlrs_result()?;

                assert!(instance.is::<Vararg>());

                let vararg = instance.cast::<Vararg>()?;
                assert!(vararg.t().is_undefined());
                assert!(vararg.n().is_undefined());
                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn create_typed_vararg() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |global, mut frame| unsafe {
                let vararg_type = DataType::vararg_type(global);
                let instance = vararg_type
                    .instantiate(&mut frame, [DataType::int32_type(global).as_value()])?
                    .into_jlrs_result()?;

                assert!(instance.is::<Vararg>());

                let vararg = instance.cast::<Vararg>()?;
                assert_eq!(
                    vararg.t().value_unchecked().cast::<DataType>()?,
                    DataType::int32_type(global)
                );
                assert!(vararg.n().is_undefined());
                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn create_typed_and_sized_vararg() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(2, |global, mut frame| unsafe {
                let vararg_type = DataType::vararg_type(global);
                let n = Value::new(&mut frame, 3isize)?;
                let instance = vararg_type
                    .instantiate(&mut frame, [DataType::int32_type(global).as_value(), n])?
                    .into_jlrs_result()?;

                assert!(instance.is::<Vararg>());

                let vararg = instance.cast::<Vararg>()?;
                assert_eq!(
                    vararg.t().value_unchecked().cast::<DataType>()?,
                    DataType::int32_type(global)
                );
                assert_eq!(vararg.n().value_unchecked().unbox::<isize>()?, 3);
                Ok(())
            })
            .unwrap();
        });
    }
}
