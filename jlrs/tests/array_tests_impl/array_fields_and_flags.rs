#[cfg(feature = "local-rt")]
pub(crate) mod tests {
    use jlrs::{
        data::managed::array::{dimensions::Dims, How},
        prelude::*,
    };

    use crate::util::JULIA;

    fn array_fields_and_flags() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let arr = unsafe { TypedArray::<f32>::new_unchecked(&mut frame, (1, 2)) };

                    assert_eq!(arr.element_size(), 4);
                    assert_eq!(arr.element_type(), DataType::float32_type(&frame));
                    assert!(arr.contains::<f32>());
                    assert_eq!(arr.length(), 2);
                    assert_eq!(arr.how(), How::InlineOrForeign);
                    assert_eq!(arr.n_dims(), 2);
                    assert!(!arr.ptr_array());
                    assert!(!arr.has_ptr());
                    assert_eq!(arr.dimensions().n_elements(0), Some(1));
                    assert_eq!(arr.dimensions().n_elements(1), Some(2));
                    assert_eq!(arr.dimensions().n_elements(2), None);
                    unsafe {
                        assert_eq!(arr.dimensions().n_elements_unchecked(0), 1);
                        assert_eq!(arr.dimensions().n_elements_unchecked(1), 2);
                        assert!(!arr.data_ptr().is_null());
                    }
                    assert!(arr.owner().is_none());

                    Ok(())
                })
                .unwrap();
        });
    }

    pub(crate) fn array_fields_and_flags_tests() {
        array_fields_and_flags();
    }
}
