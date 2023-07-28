mod util;

#[cfg(feature = "sync-rt")]
#[cfg(not(feature = "julia-1-6"))]
mod tests {
    use jlrs::prelude::*;

    use crate::util::JULIA;

    #[test]
    fn access_copied_array_dimensions() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let arr_val = Array::new::<f32, _, _>(&mut frame, (1, 2)).into_jlrs_result()?;
                    let arr = arr_val;

                    let data = unsafe { arr.copy_inline_data::<f32>()? };
                    assert_eq!(data.dimensions().as_slice(), &[1, 2]);

                    Ok(())
                })
                .unwrap();
        })
    }
}
