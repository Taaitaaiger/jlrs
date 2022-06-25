#[cfg(feature = "sync-rt")]
#[cfg(not(all(target_os = "windows", feature = "lts")))]
mod tests {
    use crate::util::JULIA;
    use jlrs::prelude::*;

    #[test]
    fn access_inline_array_dimensions() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |_, mut frame| {
                let data = vec![1.0f32, 2.0f32];
                let arr_val = Array::from_vec(&mut frame, data, (1, 2))?.into_jlrs_result()?;
                let arr = arr_val;

                let data = arr.inline_data::<f32, _>(&frame)?;
                assert_eq!(unsafe { data.dimensions().as_slice() }, &[1, 2]);

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn clone_inline_array() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |_, mut frame| {
                let data = vec![1.0f32, 2.0f32];
                let arr_val = Array::from_vec(&mut frame, data, (1, 2))?.into_jlrs_result()?;
                let arr = arr_val;

                let data = arr.inline_data::<f32, _>(&frame)?;
                let cloned_data = data.clone();
                assert_eq!(data[(0, 0)], cloned_data[(0, 0)]);
                assert_eq!(data[(0, 1)], cloned_data[(0, 1)]);

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn access_inline_data() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |_, mut frame| {
                let data = vec![1.0f32, 2.0f32];
                let arr_val = Array::from_vec(&mut frame, data, (1, 2))?.into_jlrs_result()?;
                let arr = arr_val;

                let data = arr.inline_data::<f32, _>(&frame)?;
                assert_eq!(data[(0, 1)], *data.get((0, 1)).unwrap());

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn access_inline_data_as_slice() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |_, mut frame| {
                let data = vec![1.0f32, 2.0f32];
                let arr_val = Array::from_vec(&mut frame, data, (1, 2))?.into_jlrs_result()?;
                let arr = arr_val;

                let data = arr.inline_data::<f32, _>(&frame)?;
                let slice = data.as_slice();
                assert_eq!(slice, &[1.0f32, 2.0f32]);

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn convert_inline_data_to_slice() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |_, mut frame| {
                let data = vec![1.0f32, 2.0f32];
                let arr_val = Array::from_vec(&mut frame, data, (1, 2))?.into_jlrs_result()?;
                let arr = arr_val;

                let data = arr.inline_data::<f32, _>(&frame)?;
                let slice = data.into_slice();
                assert_eq!(slice, &[1.0f32, 2.0f32]);

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn access_inline_array_mut_dimensions() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |_, mut frame| unsafe {
                let data = vec![1.0f32, 2.0f32];
                let arr_val = Array::from_vec(&mut frame, data, (1, 2))?.into_jlrs_result()?;
                let arr = arr_val;

                let data = arr.bits_data_mut::<f32, _>(&mut frame)?;
                assert_eq!(data.dimensions().as_slice(), &[1, 2]);
                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn access_bits_data_mut() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |_, mut frame| unsafe {
                let data = vec![1.0f32, 2.0f32];
                let arr_val = Array::from_vec(&mut frame, data, (1, 2))?.into_jlrs_result()?;
                let arr = arr_val;

                let mut data = arr.bits_data_mut::<f32, _>(&mut frame)?;
                data[(0, 0)] = 3.0;
                assert_eq!(data[(0, 0)], 3.0f32);
                assert_eq!(data[(0, 0)], *data.get((0, 0)).unwrap());

                *data.get_mut((0, 0)).unwrap() = 5.0;
                assert_eq!(data[(0, 0)], 5.0f32);
                assert_eq!(data[(0, 0)], *data.get((0, 0)).unwrap());

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn access_inline_mut_data_as_slice() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |_, mut frame| unsafe {
                let data = vec![1.0f32, 2.0f32];
                let arr_val = Array::from_vec(&mut frame, data, (1, 2))?.into_jlrs_result()?;
                let arr = arr_val;

                let data = arr.bits_data_mut::<f32, _>(&mut frame)?;
                let slice = data.as_slice();
                assert_eq!(slice, &[1.0f32, 2.0f32]);

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn access_inline_mut_data_as_mut_slice() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |_, mut frame| unsafe {
                let data = vec![1.0f32, 2.0f32];
                let arr_val = Array::from_vec(&mut frame, data, (1, 2))?.into_jlrs_result()?;
                let arr = arr_val;

                let mut data = arr.bits_data_mut::<f32, _>(&mut frame)?;
                let slice = data.as_mut_slice();
                slice[0] = 3.0;
                assert_eq!(slice, &[3.0f32, 2.0f32]);

                Ok(())
            })
            .unwrap();
        })
    }
    #[test]
    fn convert_inline_mut_data_to_slice() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |_, mut frame| unsafe {
                let data = vec![1.0f32, 2.0f32];
                let arr_val = Array::from_vec(&mut frame, data, (1, 2))?.into_jlrs_result()?;
                let arr = arr_val;

                let data = arr.bits_data_mut::<f32, _>(&mut frame)?;
                let slice = data.into_slice();
                assert_eq!(slice, &[1.0f32, 2.0f32]);

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn convert_inline_mut_data_to_mut_slice() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |_, mut frame| unsafe {
                let data = vec![1.0f32, 2.0f32];
                let arr_val = Array::from_vec_unchecked(&mut frame, data, (1, 2))?;
                let arr = arr_val;

                let data = arr.bits_data_mut::<f32, _>(&mut frame)?;
                let slice = data.into_mut_slice();
                slice[0] = 3.0;
                assert_eq!(slice, &[3.0f32, 2.0f32]);

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn access_unrestricted_inline_array_mut_dimensions() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |_, mut frame| {
                let data = vec![1.0f32, 2.0f32];
                let arr_val = Array::from_vec(&mut frame, data, (1, 2))?.into_jlrs_result()?;
                let arr = arr_val;

                unsafe {
                    let data = arr.unrestricted_bits_data_mut::<f32, _>(&frame)?;
                    assert_eq!(data.dimensions().as_slice(), &[1, 2]);
                }

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn access_unrestricted_bits_data_mut() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |_, mut frame| unsafe {
                let data = vec![1.0f32, 2.0f32];
                let arr_val = Array::from_vec_unchecked(&mut frame, data, (1, 2))?;
                let arr = arr_val;

                let mut data = arr.unrestricted_bits_data_mut::<f32, _>(&frame)?;
                data[(0, 0)] = 3.0;
                assert_eq!(data[(0, 0)], 3.0f32);
                assert_eq!(data[(0, 0)], *data.get((0, 0)).unwrap());

                *data.get_mut((0, 0)).unwrap() = 5.0;
                assert_eq!(data[(0, 0)], 5.0f32);
                assert_eq!(data[(0, 0)], *data.get((0, 0)).unwrap());

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn access_unrestricted_inline_mut_data_as_slice() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |_, mut frame| unsafe {
                let data = vec![1.0f32, 2.0f32];
                let arr_val = Array::from_vec(&mut frame, data, (1, 2))?.into_jlrs_result()?;
                let arr = arr_val;

                let data = arr.unrestricted_bits_data_mut::<f32, _>(&frame)?;
                let slice = data.as_slice();
                assert_eq!(slice, &[1.0f32, 2.0f32]);

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn access_unrestricted_inline_mut_data_as_mut_slice() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |_, mut frame| unsafe {
                let data = vec![1.0f32, 2.0f32];
                let arr_val = Array::from_vec(&mut frame, data, (1, 2))?.into_jlrs_result()?;
                let arr = arr_val;

                let mut data = arr.unrestricted_bits_data_mut::<f32, _>(&frame)?;
                let slice = data.as_mut_slice();
                slice[0] = 3.0;
                assert_eq!(slice, &[3.0f32, 2.0f32]);

                Ok(())
            })
            .unwrap();
        })
    }
    #[test]
    fn convert_unrestricted_inline_mut_data_to_slice() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |_, mut frame| unsafe {
                let data = vec![1.0f32, 2.0f32];
                let arr_val = Array::from_vec(&mut frame, data, (1, 2))?.into_jlrs_result()?;
                let arr = arr_val;

                let data = arr.unrestricted_bits_data_mut::<f32, _>(&frame)?;
                let slice = data.into_slice();
                assert_eq!(slice, &[1.0f32, 2.0f32]);

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn convert_unrestricted_inline_mut_data_to_mut_slice() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |_, mut frame| unsafe {
                let data = vec![1.0f32, 2.0f32];
                let arr_val = Array::from_vec(&mut frame, data, (1, 2))?.into_jlrs_result()?;
                let arr = arr_val;

                let data = arr.unrestricted_bits_data_mut::<f32, _>(&frame)?;
                let slice = data.into_mut_slice();
                slice[0] = 3.0;
                assert_eq!(slice, &[3.0f32, 2.0f32]);

                Ok(())
            })
            .unwrap();
        })
    }
}
