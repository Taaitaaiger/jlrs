#[cfg(feature = "local-rt")]
pub(crate) mod tests {
    use jlrs::{
        data::managed::array::{dimensions::Dims, How, TypedVector},
        prelude::*,
    };

    use crate::util::JULIA;

    fn typed_vector_fields() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let arr = unsafe { TypedVector::<f32>::new_unchecked(&mut frame, 4) };

                    assert_eq!(arr.element_size(), 4);
                    assert_eq!(arr.element_type(), DataType::float32_type(&frame));
                    assert!(arr.contains::<f32>());
                    assert_eq!(arr.length(), 4);
                    assert_eq!(arr.how(), How::InlineOrForeign);
                    assert_eq!(arr.n_dims(), 1);
                    assert!(!arr.ptr_array());
                    assert!(!arr.has_ptr());
                    assert_eq!(arr.dimensions().n_elements(0), Some(4));
                    assert_eq!(arr.dimensions().n_elements(1), None);
                    unsafe {
                        assert_eq!(arr.dimensions().n_elements_unchecked(0), 4);
                        assert!(!arr.data_ptr().is_null());
                    }
                    assert!(arr.owner().is_none());

                    Ok(())
                })
                .unwrap();
        });
    }

    pub(crate) fn typed_vector_fields_tests() {
        typed_vector_fields();
    }
}
