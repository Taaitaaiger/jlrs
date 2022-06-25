#[cfg(feature = "sync-rt")]
#[cfg(any(not(feature = "lts"), feature = "all-features-override"))]
mod tests {
    use crate::util::JULIA;
    use jlrs::prelude::*;

    #[test]
    fn access_copied_array_dimensions() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope_with_capacity(1, |_, mut frame| {
                let arr_val = Array::new::<f32, _, _, _>(&mut frame, (1, 2))?.into_jlrs_result()?;
                let arr = arr_val;

                let data = arr.copy_inline_data::<f32, _>(&mut frame)?;
                assert_eq!(data.dimensions().as_slice(), &[1, 2]);

                Ok(())
            })
            .unwrap();
        })
    }
}
