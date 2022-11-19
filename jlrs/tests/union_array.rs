mod util;

#[cfg(feature = "sync-rt")]
#[cfg(not(feature = "lts"))]
mod tests {
    use jlrs::prelude::*;

    use crate::util::JULIA;

    fn access_union_array_dimensions() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let global = frame.global();
                    let union_ty = DataType::uniontype_type(&frame)
                        .as_value()
                        .apply_type(
                            &mut frame,
                            &mut [
                                DataType::bool_type(&global).as_value(),
                                DataType::nothing_type(&global).as_value(),
                            ],
                        )
                        .into_jlrs_result()?;

                    let mut arr = Array::new_for(frame.as_extended_target(), 4, union_ty)
                        .into_jlrs_result()?;

                    {
                        let data = unsafe { arr.union_data()? };
                        assert_eq!(unsafe { data.dimensions().as_slice() }, &[4]);
                    }

                    unsafe {
                        let data = arr.union_data_mut()?;
                        assert_eq!(data.dimensions().as_slice(), &[4]);
                    }

                    Ok(())
                })
                .unwrap();
        })
    }

    fn set_get_union_array_data() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let global = frame.global();
                    let union_ty = DataType::uniontype_type(&frame)
                        .as_value()
                        .apply_type(
                            &mut frame,
                            &mut [
                                DataType::bool_type(&global).as_value(),
                                DataType::nothing_type(&global).as_value(),
                            ],
                        )
                        .into_jlrs_result()?;

                    let mut arr = Array::new_for(frame.as_extended_target(), 4, union_ty)
                        .into_jlrs_result()?;

                    {
                        let mut data = arr.union_data_mut()?;
                        assert!(data.contains(DataType::bool_type(&frame)));
                        data.set(0, DataType::bool_type(&frame), false)?;
                        assert_eq!(
                            data.element_type(0)?.unwrap(),
                            DataType::bool_type(&frame).as_value()
                        );
                        assert_eq!(data.get::<bool, _>(0)?, false);
                    }

                    {
                        let data = arr.union_data()?;
                        assert!(data.contains(DataType::bool_type(&frame)));
                        assert_eq!(
                            data.element_type(0)?.unwrap(),
                            DataType::bool_type(&frame).as_value()
                        );
                        assert_eq!(data.get::<bool, _>(0)?, false);
                    }

                    Ok(())
                })
                .unwrap();
        })
    }

    fn cannot_get_wrong_type() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let global = frame.global();
                    let union_ty = DataType::uniontype_type(&frame)
                        .as_value()
                        .apply_type(
                            &mut frame,
                            &mut [
                                DataType::bool_type(&global).as_value(),
                                DataType::int32_type(&global).as_value(),
                            ],
                        )
                        .into_jlrs_result()?;

                    let mut arr = Array::new_for(frame.as_extended_target(), 4, union_ty)
                        .into_jlrs_result()?;

                    {
                        let mut data = arr.union_data_mut()?;
                        data.set(0, DataType::bool_type(&frame), false)?;
                        assert!(data.get::<i64, _>(0).is_err());
                        assert!(data.get::<i32, _>(0).is_err());
                    }

                    {
                        let data = arr.union_data()?;
                        assert!(data.get::<i64, _>(0).is_err());
                        assert!(data.get::<i32, _>(0).is_err());
                    }

                    Ok(())
                })
                .unwrap();
        })
    }

    fn cannot_set_wrong_type() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let global = frame.global();
                    let union_ty = DataType::uniontype_type(&frame)
                        .as_value()
                        .apply_type(
                            &mut frame,
                            &mut [
                                DataType::bool_type(&global).as_value(),
                                DataType::int32_type(&global).as_value(),
                            ],
                        )
                        .into_jlrs_result()?;

                    let mut arr = Array::new_for(frame.as_extended_target(), 4, union_ty)
                        .into_jlrs_result()?;

                    unsafe {
                        let mut data = arr.union_data_mut()?;
                        assert!(data.set(0, DataType::bool_type(&frame), 4usize).is_err());
                        assert!(data.set(0, DataType::int32_type(&frame), false).is_err());
                        assert!(data.set(0, DataType::int64_type(&frame), 1i64).is_err());
                    }

                    Ok(())
                })
                .unwrap();
        })
    }

    #[test]
    fn union_array_tests() {
        access_union_array_dimensions();
        set_get_union_array_data();
        cannot_get_wrong_type();
        cannot_set_wrong_type();
    }
}
