#[cfg(feature = "sync-rt")]
#[cfg(not(feature = "lts"))]
mod tests {
    use crate::util::JULIA;
    use jlrs::prelude::*;

    #[test]
    fn access_copied_array_dimensions() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let arr_val = Array::new::<f32, _, _>(frame.as_extended_target(), (1, 2))
                        .into_jlrs_result()?;
                    let arr = arr_val;

                    let data = unsafe { arr.copy_inline_data::<f32>()? };
                    assert_eq!(data.dimensions().as_slice(), &[1, 2]);

                    Ok(())
                })
                .unwrap();
        })
    }
}
