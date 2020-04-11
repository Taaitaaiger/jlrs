use jlrs::prelude::*;

#[test]
fn move_array_1d() {
    let mut jlrs = unsafe { Julia::testing_instance() };

    let unboxed = jlrs
        .frame(1, |frame| {
            let data = vec![1.0f32, 2., 3.];
            let array = Value::move_array(frame, data, 3)?;
            array.try_unbox::<Array<f32>>()
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 1);
    assert_eq!(dims.n_elements(0), 3);
    assert_eq!(data, vec![1., 2., 3.]);
}

#[test]
fn move_array_1d_nested() {
    let mut jlrs = unsafe { Julia::testing_instance() };

    let unboxed = jlrs
        .frame(0, |frame| {
            frame.frame(1, |frame| {
                let data = vec![1.0f64, 2., 3.];
                let array = Value::move_array(frame, data, 3)?;
                array.try_unbox::<Array<f64>>()
            })
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 1);
    assert_eq!(dims.n_elements(0), 3);
    assert_eq!(data, vec![1., 2., 3.]);
}

#[test]
fn move_array_1d_nested_dynamic() {
    let mut jlrs = unsafe { Julia::testing_instance() };

    let unboxed = jlrs
        .frame(0, |frame| {
            frame.dynamic_frame(|frame| {
                let data = vec![1i8, 2, 3];
                let array = Value::move_array(frame, data, 3)?;
                array.try_unbox::<Array<i8>>()
            })
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 1);
    assert_eq!(dims.n_elements(0), 3);
    assert_eq!(data, vec![1, 2, 3]);
}

#[test]
fn move_array_1d_dynamic() {
    let mut jlrs = unsafe { Julia::testing_instance() };

    let unboxed = jlrs
        .dynamic_frame(|frame| {
            let data = vec![1i16, 2, 3];
                let array = Value::move_array(frame, data, 3)?;
            array.try_unbox::<Array<i16>>()
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 1);
    assert_eq!(dims.n_elements(0), 3);
    assert_eq!(data, vec![1, 2, 3]);
}

#[test]
fn move_array_1d_dynamic_nested() {
    let mut jlrs = unsafe { Julia::testing_instance() };

    let unboxed = jlrs
        .dynamic_frame(|frame| {
            frame.frame(1, |frame| {
                let data = vec![1i32, 2, 3];
                let array = Value::move_array(frame, data, 3)?;
                array.try_unbox::<Array<i32>>()
            })
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 1);
    assert_eq!(dims.n_elements(0), 3);
    assert_eq!(data, vec![1, 2, 3]);
}

#[test]
fn move_array_1d_dynamic_nested_dynamic() {
    let mut jlrs = unsafe { Julia::testing_instance() };

    let unboxed = jlrs
        .dynamic_frame(|frame| {
            frame.dynamic_frame(|frame| {
                let data = vec![1i64, 2, 3];
                let array = Value::move_array(frame, data, 3)?;
                array.try_unbox::<Array<i64>>()
            })
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 1);
    assert_eq!(dims.n_elements(0), 3);
    assert_eq!(data, vec![1, 2, 3]);
}

#[test]
fn move_array_2d() {
    let mut jlrs = unsafe { Julia::testing_instance() };

    let unboxed = jlrs
        .frame(1, |frame| {
            let data = vec![1u8, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4];
            let array = Value::move_array(frame, data, (3, 4))?;
            array.try_unbox::<Array<u8>>()
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 2);
    assert_eq!(dims.n_elements(0), 3);
    assert_eq!(dims.n_elements(1), 4);
    assert_eq!(data, vec![1u8, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4]);
}

#[test]
fn move_array_2d_nested() {
    let mut jlrs = unsafe { Julia::testing_instance() };

    let unboxed = jlrs
        .frame(0, |frame| {
            frame.frame(1, |frame| {
                let data = vec![1u16, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4];
            let array = Value::move_array(frame, data, (3, 4))?;
                array.try_unbox::<Array<u16>>()
            })
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 2);
    assert_eq!(dims.n_elements(0), 3);
    assert_eq!(dims.n_elements(1), 4);
    assert_eq!(data, vec![1u16, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4]);
}

#[test]
fn move_array_2d_nested_dynamic() {
    let mut jlrs = unsafe { Julia::testing_instance() };

    let unboxed = jlrs
        .frame(0, |frame| {
            frame.dynamic_frame(|frame| {
                let data = vec![1u32, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4];
            let array = Value::move_array(frame, data, (3, 4))?;
                array.try_unbox::<Array<u32>>()
            })
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 2);
    assert_eq!(dims.n_elements(0), 3);
    assert_eq!(dims.n_elements(1), 4);
    assert_eq!(data, vec![1u32, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4]);
}

#[test]
fn move_array_2d_dynamic() {
    let mut jlrs = unsafe { Julia::testing_instance() };

    let unboxed = jlrs
        .dynamic_frame(|frame| {
            let data = vec![1u64, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4];
            let array = Value::move_array(frame, data, (3, 4))?;
            array.try_unbox::<Array<u64>>()
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 2);
    assert_eq!(dims.n_elements(0), 3);
    assert_eq!(dims.n_elements(1), 4);
    assert_eq!(data, vec![1u64, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4]);
}

#[test]
fn move_array_2d_dynamic_nested() {
    let mut jlrs = unsafe { Julia::testing_instance() };

    let unboxed = jlrs
        .dynamic_frame(|frame| {
            frame.frame(1, |frame| {
                let data = vec![1usize, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4];
            let array = Value::move_array(frame, data, (3, 4))?;
                array.try_unbox::<Array<usize>>()
            })
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 2);
    assert_eq!(dims.n_elements(0), 3);
    assert_eq!(dims.n_elements(1), 4);
    assert_eq!(data, vec![1usize, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4]);
}

#[test]
fn move_array_2d_dynamic_nested_dynamic() {
    let mut jlrs = unsafe { Julia::testing_instance() };

    let unboxed = jlrs
        .dynamic_frame(|frame| {
            frame.dynamic_frame(|frame| {
                let data = vec![1isize, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4];
            let array = Value::move_array(frame, data, (3, 4))?;
                array.try_unbox::<Array<isize>>()
            })
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 2);
    assert_eq!(dims.n_elements(0), 3);
    assert_eq!(dims.n_elements(1), 4);
    assert_eq!(data, vec![1isize, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4]);
}
