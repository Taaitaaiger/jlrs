#[cfg(feature = "local-rt")]
pub(crate) mod tests {
    use jlrs::{
        data::managed::array::{data::accessor::AccessorMut1D, dimensions::Dims, TypedVector},
        prelude::*,
    };

    use crate::util::JULIA;

    fn typed_vector_grow_end() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let data = [1.0f32, 2.0];
                    let mut arr =
                        TypedVector::<f32>::from_slice_cloned(&mut frame, data.as_ref(), 2)
                            .unwrap()
                            .unwrap();

                    let a2 = arr.clone();
                    let dims = a2.dimensions();

                    assert_eq!(arr.length(), 2);
                    assert_eq!(dims.size(), 2);
                    let success = arr.bits_data_mut().grow_end(&frame, 1);
                    assert!(success.is_ok());
                    assert_eq!(arr.length(), 3);
                    assert_eq!(dims.size(), 3);

                    let success = arr.bits_data_mut().grow_end(&frame, 5);
                    assert!(success.is_ok());
                    assert_eq!(arr.length(), 8);
                    assert_eq!(dims.size(), 8);

                    {
                        let accessor = arr.bits_data();
                        assert_eq!(accessor.get_uninit(0).unwrap().assume_init(), 1.0);
                        assert_eq!(accessor.get_uninit(1).unwrap().assume_init(), 2.0);
                    }
                });
            });
        });
    }

    fn typed_vector_del_end() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let data = [1.0f32, 2.0, 3.0, 4.0];
                    let mut arr =
                        TypedVector::<f32>::from_slice_cloned(&mut frame, data.as_ref(), 4)
                            .unwrap()
                            .unwrap();

                    assert_eq!(arr.length(), 4);
                    let success = arr.bits_data_mut().del_end(&frame, 1);
                    assert!(success.is_ok());
                    assert_eq!(arr.length(), 3);

                    let success = arr.bits_data_mut().del_end(&frame, 2);
                    assert!(success.is_ok());
                    assert_eq!(arr.length(), 1);

                    {
                        let accessor = arr.bits_data();
                        assert_eq!(*accessor.get(0).unwrap(), 1.0);
                    }
                });
            });
        });
    }

    pub(crate) fn array_grow_del_tests() {
        typed_vector_grow_end();
        typed_vector_del_end();
    }
}
