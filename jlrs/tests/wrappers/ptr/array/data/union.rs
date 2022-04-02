#[cfg(feature = "sync-rt")]
mod tests {
    use crate::util::JULIA;
    use jlrs::prelude::*;

    #[test]
    fn access_union_array_dimensions() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |global, frame| {
                let union_ty = DataType::uniontype_type(global)
                    .as_value()
                    .apply_type(
                        &mut *frame,
                        &mut [
                            DataType::bool_type(global).as_value(),
                            DataType::nothing_type(global).as_value(),
                        ],
                    )?
                    .into_jlrs_result()?;

                let arr = Array::new_for(&mut *frame, 4, union_ty)?.into_jlrs_result()?;

                {
                    let data = arr.union_data(frame)?;
                    assert_eq!(data.dimensions().as_slice(), &[4]);
                }

                unsafe {
                    let data = arr.union_data_mut(frame)?;
                    assert_eq!(data.dimensions().as_slice(), &[4]);
                }

                unsafe {
                    let data = arr.unrestricted_union_data_mut(frame)?;
                    assert_eq!(data.dimensions().as_slice(), &[4]);
                }

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn set_get_union_array_data() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |global, frame| unsafe {
                let union_ty = DataType::uniontype_type(global)
                    .as_value()
                    .apply_type(
                        &mut *frame,
                        &mut [
                            DataType::bool_type(global).as_value(),
                            DataType::nothing_type(global).as_value(),
                        ],
                    )?
                    .into_jlrs_result()?;

                let arr = Array::new_for(&mut *frame, 4, union_ty)?.into_jlrs_result()?;

                {
                    let mut data = arr.union_data_mut(frame)?;
                    assert!(data.contains(DataType::bool_type(global)));
                    data.set(0, DataType::bool_type(global), false)?;
                    assert_eq!(
                        data.element_type(0)?.unwrap(),
                        DataType::bool_type(global).as_value()
                    );
                    assert_eq!(data.get::<bool, _>(0)?, false);
                }

                {
                    let data = arr.union_data(frame)?;
                    assert!(data.contains(DataType::bool_type(global)));
                    assert_eq!(
                        data.element_type(0)?.unwrap(),
                        DataType::bool_type(global).as_value()
                    );
                    assert_eq!(data.get::<bool, _>(0)?, false);
                }

                {
                    let mut data = arr.unrestricted_union_data_mut(frame)?;
                    assert!(data.contains(DataType::bool_type(global)));
                    data.set(0, DataType::bool_type(global), true)?;
                    assert_eq!(
                        data.element_type(0)?.unwrap(),
                        DataType::bool_type(global).as_value()
                    );
                    assert_eq!(data.get::<bool, _>(0)?, true);
                }

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn cannot_get_wrong_type() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |global, frame| unsafe {
                let union_ty = DataType::uniontype_type(global)
                    .as_value()
                    .apply_type(
                        &mut *frame,
                        &mut [
                            DataType::bool_type(global).as_value(),
                            DataType::int32_type(global).as_value(),
                        ],
                    )?
                    .into_jlrs_result()?;

                let arr = Array::new_for(&mut *frame, 4, union_ty)?.into_jlrs_result()?;

                {
                    let mut data = arr.union_data_mut(frame)?;
                    data.set(0, DataType::bool_type(global), false)?;
                    assert!(data.get::<i64, _>(0).is_err());
                    assert!(data.get::<i32, _>(0).is_err());
                }

                {
                    let data = arr.union_data(frame)?;
                    assert!(data.get::<i64, _>(0).is_err());
                    assert!(data.get::<i32, _>(0).is_err());
                }

                {
                    let data = arr.unrestricted_union_data_mut(frame)?;
                    assert!(data.get::<i64, _>(0).is_err());
                    assert!(data.get::<i32, _>(0).is_err());
                }

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn cannot_set_wrong_type() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |global, frame| {
                let union_ty = DataType::uniontype_type(global)
                    .as_value()
                    .apply_type(
                        &mut *frame,
                        &mut [
                            DataType::bool_type(global).as_value(),
                            DataType::int32_type(global).as_value(),
                        ],
                    )?
                    .into_jlrs_result()?;

                let arr = Array::new_for(&mut *frame, 4, union_ty)?.into_jlrs_result()?;

                unsafe {
                    let mut data = arr.union_data_mut(frame)?;
                    assert!(data.set(0, DataType::bool_type(global), 4usize).is_err());
                    assert!(data.set(0, DataType::int32_type(global), false).is_err());
                    assert!(data.set(0, DataType::int64_type(global), 1i64).is_err());
                }

                unsafe {
                    let mut data = arr.unrestricted_union_data_mut(frame)?;
                    assert!(data.set(0, DataType::bool_type(global), 4usize).is_err());
                    assert!(data.set(0, DataType::int32_type(global), false).is_err());
                    assert!(data.set(0, DataType::int64_type(global), 1i64).is_err());
                }

                Ok(())
            })
            .unwrap();
        })
    }
}
