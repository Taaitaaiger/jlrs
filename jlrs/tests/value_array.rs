mod util;

#[cfg(feature = "sync-rt")]
#[cfg(not(feature = "julia-1-6"))]
mod tests {
    use jlrs::prelude::*;

    use crate::util::JULIA;

    fn access_value_array_dimensions() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let global = frame.unrooted();
                    let mut arr =
                        Array::new_for(&mut frame, 4, DataType::module_type(&global).as_value())
                            .into_jlrs_result()?;

                    {
                        let data = unsafe { arr.value_data()? };
                        assert_eq!(unsafe { data.dimensions().as_slice() }, &[4]);
                    }

                    unsafe {
                        let data = arr.value_data_mut()?;
                        assert_eq!(data.dimensions().as_slice(), &[4]);
                    }

                    {
                        let data = unsafe { arr.managed_data::<ModuleRef>()? };
                        assert_eq!(unsafe { data.dimensions().as_slice() }, &[4]);
                    }

                    unsafe {
                        let data = arr.managed_data_mut::<ModuleRef>()?;
                        assert_eq!(data.dimensions().as_slice(), &[4]);
                    }

                    Ok(())
                })
                .unwrap();
        })
    }

    fn set_and_get_value_array_data() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let global = frame.unrooted();
                    let mut arr =
                        Array::new_for(&mut frame, 4, DataType::module_type(&global).as_value())
                            .into_jlrs_result()?;

                    let module = Module::core(&frame).as_value();

                    unsafe {
                        let mut data = arr.value_data_mut()?;
                        assert!(data[0].is_none());
                        assert!(data.set(0, Some(module)).is_ok());
                        assert!(data.set(1, Some(module)).is_ok());
                        assert!(!data[0].is_none());
                        assert_eq!(data[0].unwrap().as_value(), module);
                        assert_eq!(data.get(&mut frame, 0).unwrap(), module);
                    }

                    unsafe {
                        let data = arr.value_data()?;
                        assert_eq!(data[0].unwrap().as_value(), module);
                        assert_eq!(data.get(&mut frame, 0).unwrap(), module);
                    }

                    unsafe {
                        let data = arr.managed_data::<ModuleRef>()?;
                        assert_eq!(data[1].unwrap().as_value(), module);
                        assert_eq!(data.get(&mut frame, 1).unwrap(), module.cast::<Module>()?);
                    }

                    unsafe {
                        let mut data = arr.managed_data_mut::<ModuleRef>()?;
                        assert!(data[2].is_none());
                        assert!(data.set(2, Some(module)).is_ok());
                        assert!(!data[2].is_none());
                        assert_eq!(data[2].unwrap().as_value(), module);
                        assert_eq!(data.get(&frame, 2).unwrap().as_value(), module);

                        assert!(data.set(2, None).is_ok());
                        assert!(data[2].is_none());
                    }

                    Ok(())
                })
                .unwrap();
        })
    }

    fn cannot_set_invalid_type() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let global = frame.unrooted();
                    let mut arr =
                        Array::new_for(&mut frame, 4, DataType::module_type(&global).as_value())
                            .into_jlrs_result()?;

                    let module = Value::nothing(&frame);

                    {
                        let mut data = unsafe { arr.value_data_mut()? };
                        assert!(data.set(0, Some(module)).is_err());
                    }

                    {
                        let mut data = unsafe { arr.managed_data_mut::<ModuleRef>()? };
                        assert!(data.set(0, Some(module)).is_err());
                    }

                    Ok(())
                })
                .unwrap();
        })
    }

    fn get_data_as_slice() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let global = frame.unrooted();
                    let mut arr =
                        Array::new_for(&mut frame, 4, DataType::module_type(&global).as_value())
                            .into_jlrs_result()?;

                    {
                        let data = unsafe { arr.value_data_mut()? };
                        let slice = data.as_slice();
                        assert_eq!(slice.len(), 4)
                    }

                    {
                        let data = unsafe { arr.value_data()? };
                        let slice = data.as_slice();
                        assert_eq!(slice.len(), 4)
                    }

                    {
                        let data = unsafe { arr.managed_data::<ModuleRef>()? };
                        let slice = data.as_slice();
                        assert_eq!(slice.len(), 4)
                    }

                    {
                        let data = unsafe { arr.managed_data_mut::<ModuleRef>()? };
                        let slice = data.as_slice();
                        assert_eq!(slice.len(), 4)
                    }

                    Ok(())
                })
                .unwrap();
        })
    }

    #[test]
    fn value_array_tests() {
        access_value_array_dimensions();
        set_and_get_value_array_data();
        cannot_set_invalid_type();
        get_data_as_slice();
    }
}
