use jlrs::prelude::*;
use jlrs::util::JULIA;

#[test]
fn borrow_array_1d() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        let mut data = vec![1u64, 2, 3, 4];

        let unboxed = jlrs
            .frame(1, |_, frame| {
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
fn borrow_array_1d_output() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        let mut data = vec![1u64, 2, 3, 4];

        let unboxed = jlrs
            .frame(1, |_, frame| {
                let output = frame.output()?;
                let array = Value::borrow_array_output(frame, output, &mut data, 4)?;
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
            .dynamic_frame(|_, frame| {
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
            .frame(1, |_, frame| {
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
            .dynamic_frame(|_, frame| {
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
