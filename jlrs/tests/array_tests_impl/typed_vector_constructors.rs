#[cfg(feature = "local-rt")]
pub(crate) mod tests {
    use jlrs::{
        data::managed::array::{dimensions::Dims, TypedVector},
        prelude::*,
    };

    use crate::util::JULIA;

    fn typed_vector_from_bytes(julia: &mut Julia) {
        julia
            .returning::<JlrsResult<_>>()
            .scope(|mut frame| {
                let dt = DataType::uint8_type(&frame).as_value();
                let s = "jlrs";
                let arr = TypedVector::from_bytes(&mut frame, s);
                assert!(arr.is_ok());

                let arr = arr.unwrap();
                assert_eq!(arr.n_dims(), 1);
                assert_eq!(arr.dimensions().size(), s.len());
                assert_eq!(arr.element_type(), dt);
                Ok(())
            })
            .unwrap();
    }

    fn typed_vector_from_bytes_unchecked(julia: &mut Julia) {
        julia
            .returning::<JlrsResult<_>>()
            .scope(|mut frame| {
                let dt = DataType::uint8_type(&frame).as_value();
                let s = "é";
                let arr = unsafe { TypedVector::from_bytes_unchecked(&mut frame, s) };
                assert_eq!(arr.n_dims(), 1);
                assert_eq!(arr.dimensions().size(), s.len());
                assert_eq!(arr.element_type(), dt);
                Ok(())
            })
            .unwrap();
    }

    pub(crate) fn typed_vector_constructors_test() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let mut inst = jlrs.instance(&mut frame);
            typed_vector_from_bytes(&mut inst);
            typed_vector_from_bytes_unchecked(&mut inst);
        });
    }
}
