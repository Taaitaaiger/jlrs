use jlrs::prelude::*;
use jlrs::util::JULIA;

#[test]
fn borrow_array_1d() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        let mut data = vec![1u64, 2, 3, 4];

        let unboxed = jlrs
            .frame_with_slots(1, |_, frame| {
                let array = Value::borrow_array(frame, &mut data, 4)?;
                assert!(array.is_array_of::<u64>());
                array.cast::<Array>()?.copy_inline_data::<u64>()
            })
            .unwrap();

        let (data, dims) = unboxed.splat();
        assert_eq!(dims.n_dimensions(), 1);
        assert_eq!(dims.n_elements(0), 4);
        assert_eq!(data, vec![1, 2, 3, 4]);
    });
}

#[test]
fn borrow_array_1d_dynamic_type() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        struct Foo<'a> {
            slice: &'a mut [u8],
        }

        let foo = Foo {
            slice: &mut [1, 2, 3, 4, 5, 6, 7, 8],
        };

        let unboxed = jlrs
            .frame_with_slots(1, |_, frame| {
                let x = false;

                let array = match x {
                    true => Value::borrow_array(frame, foo.slice, 8)?,
                    false => unsafe {
                        let ptr = foo.slice.as_mut_ptr().cast::<u16>();
                        let slice = std::slice::from_raw_parts_mut(ptr, 4);
                        Value::borrow_array(frame, slice, 4)?
                    },
                };

                assert!(array.is_array_of::<u16>());
                array.cast::<Array>()?.copy_inline_data::<u16>()
            })
            .unwrap();

        let (data, dims) = unboxed.splat();
        assert_eq!(dims.n_dimensions(), 1);
        assert_eq!(dims.n_elements(0), 4);
        assert_eq!(data, vec![513, 1027, 1541, 2055]);
    });
}

#[test]
fn borrow_array_1d_output() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        let mut data = vec![1u64, 2, 3, 4];

        let unboxed = jlrs
            .frame_with_slots(1, |_, frame| {
                let array = frame.value_frame_with_slots(0, |output, frame| {
                    let output = output.into_scope(frame);
                    Value::borrow_array(output, &mut data, 4)
                })?;
                assert!(array.is_array_of::<u64>());
                array.cast::<Array>()?.copy_inline_data::<u64>()
            })
            .unwrap();

        let (data, dims) = unboxed.splat();
        assert_eq!(dims.n_dimensions(), 1);
        assert_eq!(dims.n_elements(0), 4);
        assert_eq!(data, vec![1, 2, 3, 4]);
    });
}

#[test]
fn borrow_array_1d_dynamic() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        let mut data = vec![1u64, 2, 3, 4];

        let unboxed = jlrs
            .frame(|_, frame| {
                let array = Value::borrow_array(frame, &mut data, 4)?;
                array.cast::<Array>()?.copy_inline_data::<u64>()
            })
            .unwrap();

        let (data, dims) = unboxed.splat();
        assert_eq!(dims.n_dimensions(), 1);
        assert_eq!(dims.n_elements(0), 4);
        assert_eq!(data, vec![1, 2, 3, 4]);
    });
}

#[test]
fn borrow_array_2d() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        let mut data = vec![1u64, 2, 3, 4];

        let unboxed = jlrs
            .frame_with_slots(1, |_, frame| {
                let array = Value::borrow_array(frame, &mut data, (2, 2))?;
                array.cast::<Array>()?.copy_inline_data::<u64>()
            })
            .unwrap();

        let (data, dims) = unboxed.splat();
        assert_eq!(dims.n_dimensions(), 2);
        assert_eq!(dims.n_elements(0), 2);
        assert_eq!(dims.n_elements(1), 2);
        assert_eq!(data, vec![1, 2, 3, 4]);
    });
}

#[test]
fn borrow_array_2d_dynamic() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        let mut data = vec![1u64, 2, 3, 4];

        let unboxed = jlrs
            .frame(|_, frame| {
                let array = Value::borrow_array(frame, &mut data, (2, 2))?;
                array.cast::<Array>()?.copy_inline_data::<u64>()
            })
            .unwrap();
        let (data, dims) = unboxed.splat();
        assert_eq!(dims.n_dimensions(), 2);
        assert_eq!(dims.n_elements(0), 2);
        assert_eq!(dims.n_elements(1), 2);
        assert_eq!(data, vec![1, 2, 3, 4]);
    });
}
