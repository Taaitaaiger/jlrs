mod util;

#[cfg(feature = "local-rt")]
mod tests {
    use jlrs::prelude::*;

    use crate::util::JULIA;

    #[test]
    fn access_copied_array_dimensions() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|mut frame| {
                        let arr = TypedArray::<f32>::new(&mut frame, (1, 2)).into_jlrs_result()?;

                        let data = unsafe { arr.bits_data().to_copied_array() };
                        assert_eq!(data.dimensions().as_slice(), &[1, 2]);

                        Ok(())
                    })
                    .unwrap();
            })
        })
    }
}
