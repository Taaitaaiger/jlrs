mod util;

#[cfg(feature = "sync-rt")]
mod tests {
    use jlrs::prelude::*;

    use crate::util::JULIA;

    fn access_inline_array_dimensions() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let data = vec![1.0f32, 2.0f32];
                    let arr_val = Array::from_vec(frame.as_extended_target(), data, (1, 2))?
                        .into_jlrs_result()?;
                    let arr = arr_val;

                    let data = unsafe { arr.inline_data::<f32>()? };
                    assert_eq!(unsafe { data.dimensions().as_slice() }, &[1, 2]);

                    Ok(())
                })
                .unwrap();
        })
    }

    fn clone_inline_array() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let data = vec![1.0f32, 2.0f32];
                    let arr_val = Array::from_vec(frame.as_extended_target(), data, (1, 2))?
                        .into_jlrs_result()?;
                    let arr = arr_val;

                    let data = unsafe { arr.inline_data::<f32>()? };
                    let cloned_data = data.clone();
                    assert_eq!(data[(0, 0)], cloned_data[(0, 0)]);
                    assert_eq!(data[(0, 1)], cloned_data[(0, 1)]);

                    Ok(())
                })
                .unwrap();
        })
    }

    fn access_inline_data() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let data = vec![1.0f32, 2.0f32];
                    let arr_val = Array::from_vec(frame.as_extended_target(), data, (1, 2))?
                        .into_jlrs_result()?;
                    let arr = arr_val;

                    let data = unsafe { arr.inline_data::<f32>()? };
                    assert_eq!(data[(0, 1)], *data.get((0, 1)).unwrap());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn access_inline_data_as_slice() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let data = vec![1.0f32, 2.0f32];
                    let arr_val = Array::from_vec(frame.as_extended_target(), data, (1, 2))?
                        .into_jlrs_result()?;
                    let arr = arr_val;

                    let data = unsafe { arr.inline_data::<f32>()? };
                    let slice = data.as_slice();
                    assert_eq!(slice, &[1.0f32, 2.0f32]);

                    Ok(())
                })
                .unwrap();
        })
    }

    fn convert_inline_data_to_slice() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let data = vec![1.0f32, 2.0f32];
                    let arr_val = Array::from_vec(frame.as_extended_target(), data, (1, 2))?
                        .into_jlrs_result()?;
                    let arr = arr_val;

                    let data = unsafe { arr.inline_data::<f32>()? };
                    let slice = data.into_slice();
                    assert_eq!(slice, &[1.0f32, 2.0f32]);

                    Ok(())
                })
                .unwrap();
        })
    }

    fn access_inline_array_mut_dimensions() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let data = vec![1.0f32, 2.0f32];
                    let arr_val = Array::from_vec(frame.as_extended_target(), data, (1, 2))?
                        .into_jlrs_result()?;
                    let mut arr = arr_val;

                    let data = arr.bits_data_mut::<f32>()?;
                    assert_eq!(data.dimensions().as_slice(), &[1, 2]);
                    Ok(())
                })
                .unwrap();
        })
    }

    fn access_bits_data_mut() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let data = vec![1.0f32, 2.0f32];
                    let arr_val = Array::from_vec(frame.as_extended_target(), data, (1, 2))?
                        .into_jlrs_result()?;
                    let mut arr = arr_val;

                    let mut data = unsafe { arr.bits_data_mut::<f32>()? };
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

    fn access_inline_mut_data_as_slice() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let data = vec![1.0f32, 2.0f32];
                    let arr_val = Array::from_vec(frame.as_extended_target(), data, (1, 2))?
                        .into_jlrs_result()?;
                    let mut arr = arr_val;

                    let data = unsafe { arr.bits_data_mut::<f32>()? };
                    let slice = data.as_slice();
                    assert_eq!(slice, &[1.0f32, 2.0f32]);

                    Ok(())
                })
                .unwrap();
        })
    }

    fn access_inline_mut_data_as_mut_slice() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let data = vec![1.0f32, 2.0f32];
                    let arr_val = Array::from_vec(frame.as_extended_target(), data, (1, 2))?
                        .into_jlrs_result()?;
                    let mut arr = arr_val;

                    let mut data = unsafe { arr.bits_data_mut::<f32>()? };
                    let slice = data.as_mut_slice();
                    slice[0] = 3.0;
                    assert_eq!(slice, &[3.0f32, 2.0f32]);

                    Ok(())
                })
                .unwrap();
        })
    }

    fn convert_inline_mut_data_to_slice() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let data = vec![1.0f32, 2.0f32];
                    let arr_val = Array::from_vec(frame.as_extended_target(), data, (1, 2))?
                        .into_jlrs_result()?;
                    let mut arr = arr_val;

                    let data = unsafe { arr.bits_data_mut::<f32>()? };
                    let slice = data.into_slice();
                    assert_eq!(slice, &[1.0f32, 2.0f32]);

                    Ok(())
                })
                .unwrap();
        })
    }

    fn convert_inline_mut_data_to_mut_slice() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let data = vec![1.0f32, 2.0f32];
                    let arr_val =
                        Array::from_vec_unchecked(frame.as_extended_target(), data, (1, 2))?;
                    let mut arr = arr_val;

                    let data = arr.bits_data_mut::<f32>()?;
                    let slice = data.into_mut_slice();
                    slice[0] = 3.0;
                    assert_eq!(slice, &[3.0f32, 2.0f32]);

                    Ok(())
                })
                .unwrap();
        })
    }

    #[test]
    fn inline_array_tests() {
        access_inline_array_dimensions();
        clone_inline_array();
        access_inline_data();
        access_inline_data_as_slice();
        convert_inline_data_to_slice();
        access_inline_array_mut_dimensions();
        access_bits_data_mut();
        access_inline_mut_data_as_slice();
        access_inline_mut_data_as_mut_slice();
        convert_inline_mut_data_to_slice();
        convert_inline_mut_data_to_mut_slice();
    }
}
