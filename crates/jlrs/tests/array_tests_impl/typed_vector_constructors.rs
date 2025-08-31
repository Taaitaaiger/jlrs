#[cfg(feature = "local-rt")]
pub(crate) mod tests {
    use jlrs::{
        data::managed::array::{TypedVector, dimensions::Dims},
        prelude::*,
        runtime::handle::with_stack::StackHandle,
    };

    use crate::util::JULIA;

    fn typed_vector_from_bytes(julia: &mut StackHandle) {
        julia.scope(|mut frame| {
            let dt = DataType::uint8_type(&frame).as_value();
            let s = "jlrs";
            let arr = TypedVector::from_bytes(&mut frame, s);
            assert!(arr.is_ok());

            let arr = arr.unwrap();
            assert_eq!(arr.n_dims(), 1);
            assert_eq!(arr.dimensions().size(), s.len());
            assert_eq!(arr.element_type(), dt);
        })
    }

    fn typed_vector_from_bytes_unchecked(julia: &mut StackHandle) {
        julia.scope(|mut frame| {
            let dt = DataType::uint8_type(&frame).as_value();
            let s = "é";
            let arr = unsafe { TypedVector::from_bytes_unchecked(&mut frame, s) };
            assert_eq!(arr.n_dims(), 1);
            assert_eq!(arr.dimensions().size(), s.len());
            assert_eq!(arr.element_type(), dt);
        })
    }

    pub(crate) fn typed_vector_constructors_test() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                typed_vector_from_bytes(&mut stack);
                typed_vector_from_bytes_unchecked(&mut stack);
            });
        });
    }
}
