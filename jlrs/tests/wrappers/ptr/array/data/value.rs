#[cfg(feature = "sync-rt")]
#[cfg(not(feature = "lts"))]
mod tests {
    use crate::util::JULIA;
    use jlrs::prelude::*;
    use jlrs::wrappers::ptr::ModuleRef;

    #[test]
    fn access_value_array_dimensions() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |global, frame| {
                let arr = Array::new_for(&mut *frame, 4, DataType::module_type(global).as_value())?
                    .into_jlrs_result()?;

                {
                    let data = arr.value_data(frame)?;
                    assert_eq!(data.dimensions().as_slice(), &[4]);
                }

                unsafe {
                    let data = arr.value_data_mut(frame)?;
                    assert_eq!(data.dimensions().as_slice(), &[4]);
                }

                unsafe {
                    let data = arr.unrestricted_value_data_mut(frame)?;
                    assert_eq!(data.dimensions().as_slice(), &[4]);
                }

                {
                    let data = arr.wrapper_data::<ModuleRef, _>(frame)?;
                    assert_eq!(data.dimensions().as_slice(), &[4]);
                }

                unsafe {
                    let data = arr.wrapper_data_mut::<ModuleRef, _>(frame)?;
                    assert_eq!(data.dimensions().as_slice(), &[4]);
                }

                unsafe {
                    let data = arr.unrestricted_wrapper_data_mut::<ModuleRef, _>(frame)?;
                    assert_eq!(data.dimensions().as_slice(), &[4]);
                }

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn set_and_get_value_array_data() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |global, frame| {
                let arr = Array::new_for(&mut *frame, 4, DataType::module_type(global).as_value())?
                    .into_jlrs_result()?;

                let module = Module::core(global).as_value();

                unsafe {
                    let mut data = arr.value_data_mut(frame)?;
                    assert!(data[0].is_undefined());
                    assert!(data.set(0, Some(module)).is_ok());
                    assert!(!data[0].is_undefined());
                    assert_eq!(data[0].value_unchecked(), module);
                    assert_eq!(data.get(0).unwrap().value_unchecked(), module);
                }

                unsafe {
                    let data = arr.value_data(frame)?;
                    assert_eq!(data[0].value_unchecked(), module);
                    assert_eq!(data.get(0).unwrap().value_unchecked(), module);
                }

                unsafe {
                    let mut data = arr.unrestricted_value_data_mut(frame)?;
                    assert!(data[1].is_undefined());
                    assert!(data.set(1, Some(module)).is_ok());
                    assert!(!data[1].is_undefined());
                    assert_eq!(data[1].value_unchecked(), module);
                    assert_eq!(data.get(1).unwrap().value_unchecked(), module);
                }

                unsafe {
                    let data = arr.wrapper_data::<ModuleRef, _>(frame)?;
                    assert_eq!(data[1].value_unchecked(), module);
                    assert_eq!(
                        data.get(1).unwrap().wrapper_unchecked(),
                        module.cast::<Module>()?
                    );
                }

                unsafe {
                    let mut data = arr.wrapper_data_mut::<ModuleRef, _>(frame)?;
                    assert!(data[2].is_undefined());
                    assert!(data.set(2, Some(module)).is_ok());
                    assert!(!data[2].is_undefined());
                    assert_eq!(data[2].value_unchecked(), module);
                    assert_eq!(data.get(2).unwrap().value_unchecked(), module);

                    assert!(data.set(2, None).is_ok());
                    assert!(data[2].is_undefined());
                }

                unsafe {
                    let mut data = arr.unrestricted_wrapper_data_mut::<ModuleRef, _>(frame)?;
                    assert!(data[3].is_undefined());
                    assert!(data.set(3, Some(module)).is_ok());
                    assert!(!data[3].is_undefined());
                    assert_eq!(data[3].value_unchecked(), module);
                    assert_eq!(data.get(3).unwrap().value_unchecked(), module);

                    assert!(data.set(3, None).is_ok());
                    assert!(data[3].is_undefined());
                }

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn cannot_set_invalid_type() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |global, frame| unsafe {
                let arr = Array::new_for(&mut *frame, 4, DataType::module_type(global).as_value())?
                    .into_jlrs_result()?;

                let module = Value::nothing(global);

                {
                    let mut data = arr.value_data_mut(frame)?;
                    assert!(data.set(0, Some(module)).is_err());
                }

                {
                    let mut data = arr.unrestricted_value_data_mut(frame)?;
                    assert!(data.set(0, Some(module)).is_err());
                }

                {
                    let mut data = arr.wrapper_data_mut::<ModuleRef, _>(frame)?;
                    assert!(data.set(0, Some(module)).is_err());
                }

                {
                    let mut data = arr.unrestricted_wrapper_data_mut::<ModuleRef, _>(frame)?;
                    assert!(data.set(0, Some(module)).is_err());
                }

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn get_data_as_slice() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |global, frame| unsafe {
                let arr = Array::new_for(&mut *frame, 4, DataType::module_type(global).as_value())?
                    .into_jlrs_result()?;

                {
                    let data = arr.value_data_mut(frame)?;
                    let slice = data.as_slice();
                    assert_eq!(slice.len(), 4)
                }

                {
                    let data = arr.value_data(frame)?;
                    let slice = data.as_slice();
                    assert_eq!(slice.len(), 4)
                }

                {
                    let data = arr.unrestricted_value_data_mut(frame)?;
                    let slice = data.as_slice();
                    assert_eq!(slice.len(), 4)
                }

                {
                    let data = arr.wrapper_data::<ModuleRef, _>(frame)?;
                    let slice = data.as_slice();
                    assert_eq!(slice.len(), 4)
                }

                {
                    let data = arr.wrapper_data_mut::<ModuleRef, _>(frame)?;
                    let slice = data.as_slice();
                    assert_eq!(slice.len(), 4)
                }

                {
                    let data = arr.unrestricted_wrapper_data_mut::<ModuleRef, _>(frame)?;
                    let slice = data.as_slice();
                    assert_eq!(slice.len(), 4)
                }

                Ok(())
            })
            .unwrap();
        })
    }
}
