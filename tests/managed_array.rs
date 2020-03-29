use jlrs::prelude::*;

#[test]
fn array_1d() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    let unboxed = jlrs
        .frame(1, |frame| {
            let array = Value::array::<f32, _, _>(frame, 3)?;
            array.try_unbox::<Array<f32>>()
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 1);
    assert_eq!(dims.n_elements(0), 3);
    assert_eq!(data.len(), 3);
}

#[test]
fn array_1d_nested() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    let unboxed = jlrs
        .frame(0, |frame| {
            frame.frame(1, |frame| {
                let array = Value::array::<f64, _, _>(frame, 3)?;
                array.try_unbox::<Array<f64>>()
            })
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 1);
    assert_eq!(dims.n_elements(0), 3);
    assert_eq!(data.len(), 3);
}

#[test]
fn array_1d_nested_dynamic() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    let unboxed = jlrs
        .frame(0, |frame| {
            frame.dynamic_frame(|frame| {
                let array = Value::array::<i8, _, _>(frame, 3)?;
                array.try_unbox::<Array<i8>>()
            })
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 1);
    assert_eq!(dims.n_elements(0), 3);
    assert_eq!(data.len(), 3);
}

#[test]
fn array_1d_dynamic() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    let unboxed = jlrs
        .dynamic_frame(|frame| {
            let array = Value::array::<i16, _, _>(frame, 3)?;
            array.try_unbox::<Array<i16>>()
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 1);
    assert_eq!(dims.n_elements(0), 3);
    assert_eq!(data.len(), 3);
}

#[test]
fn array_1d_dynamic_nested() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    let unboxed = jlrs
        .dynamic_frame(|frame| {
            frame.frame(1, |frame| {
                let array = Value::array::<i32, _, _>(frame, 3)?;
                array.try_unbox::<Array<i32>>()
            })
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 1);
    assert_eq!(dims.n_elements(0), 3);
    assert_eq!(data.len(), 3);
}

#[test]
fn array_1d_dynamic_nested_dynamic() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    let unboxed = jlrs
        .dynamic_frame(|frame| {
            frame.dynamic_frame(|frame| {
                let array = Value::array::<i64, _, _>(frame, 3)?;
                array.try_unbox::<Array<i64>>()
            })
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 1);
    assert_eq!(dims.n_elements(0), 3);
    assert_eq!(data.len(), 3);
}

#[test]
fn array_2d() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    let unboxed = jlrs
        .frame(1, |frame| {
            let array = Value::array::<u8, _, _>(frame, (3, 4))?;
            array.try_unbox::<Array<u8>>()
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 2);
    assert_eq!(dims.n_elements(0), 3);
    assert_eq!(dims.n_elements(1), 4);
    assert_eq!(data.len(), 12);
}

#[test]
fn array_2d_nested() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    let unboxed = jlrs
        .frame(0, |frame| {
            frame.frame(1, |frame| {
                let array = Value::array::<u16, _, _>(frame, (3, 4))?;
                array.try_unbox::<Array<u16>>()
            })
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 2);
    assert_eq!(dims.n_elements(0), 3);
    assert_eq!(dims.n_elements(1), 4);
    assert_eq!(data.len(), 12);
}

#[test]
fn array_2d_nested_dynamic() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    let unboxed = jlrs
        .frame(0, |frame| {
            frame.dynamic_frame(|frame| {
                let array = Value::array::<u32, _, _>(frame, (3, 4))?;
                array.try_unbox::<Array<u32>>()
            })
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 2);
    assert_eq!(dims.n_elements(0), 3);
    assert_eq!(dims.n_elements(1), 4);
    assert_eq!(data.len(), 12);
}

#[test]
fn array_2d_dynamic() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    let unboxed = jlrs
        .dynamic_frame(|frame| {
            let array = Value::array::<u64, _, _>(frame, (3, 4))?;
            array.try_unbox::<Array<u64>>()
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 2);
    assert_eq!(dims.n_elements(0), 3);
    assert_eq!(dims.n_elements(1), 4);
    assert_eq!(data.len(), 12);
}

#[test]
fn array_2d_dynamic_nested() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    let unboxed = jlrs
        .dynamic_frame(|frame| {
            frame.frame(1, |frame| {
                let array = Value::array::<usize, _, _>(frame, (3, 4))?;
                array.try_unbox::<Array<usize>>()
            })
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 2);
    assert_eq!(dims.n_elements(0), 3);
    assert_eq!(dims.n_elements(1), 4);
    assert_eq!(data.len(), 12);
}

#[test]
fn array_2d_dynamic_nested_dynamic() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    let unboxed = jlrs
        .dynamic_frame(|frame| {
            frame.dynamic_frame(|frame| {
                let array = Value::array::<isize, _, _>(frame, (3, 4))?;
                array.try_unbox::<Array<isize>>()
            })
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 2);
    assert_eq!(dims.n_elements(0), 3);
    assert_eq!(dims.n_elements(1), 4);
    assert_eq!(data.len(), 12);
}

#[test]
fn array_3d() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    let unboxed = jlrs
        .frame(1, |frame| {
            let array = Value::array::<u8, _, _>(frame, (3, 4, 5))?;
            array.try_unbox::<Array<u8>>()
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 3);
    assert_eq!(dims.n_elements(0), 3);
    assert_eq!(dims.n_elements(1), 4);
    assert_eq!(dims.n_elements(2), 5);
    assert_eq!(data.len(), 60);
}

#[test]
fn array_3d_nested() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    let unboxed = jlrs
        .frame(0, |frame| {
            frame.frame(1, |frame| {
                let array = Value::array::<u16, _, _>(frame, (3, 4, 5))?;
                array.try_unbox::<Array<u16>>()
            })
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 3);
    assert_eq!(dims.n_elements(0), 3);
    assert_eq!(dims.n_elements(1), 4);
    assert_eq!(dims.n_elements(2), 5);
    assert_eq!(data.len(), 60);
}

#[test]
fn array_3d_nested_dynamic() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    let unboxed = jlrs
        .frame(0, |frame| {
            frame.dynamic_frame(|frame| {
                let array = Value::array::<u32, _, _>(frame, (3, 4, 5))?;
                array.try_unbox::<Array<u32>>()
            })
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 3);
    assert_eq!(dims.n_elements(0), 3);
    assert_eq!(dims.n_elements(1), 4);
    assert_eq!(dims.n_elements(2), 5);
    assert_eq!(data.len(), 60);
}

#[test]
fn array_3d_dynamic() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    let unboxed = jlrs
        .dynamic_frame(|frame| {
            let array = Value::array::<u64, _, _>(frame, (3, 4, 5))?;
            array.try_unbox::<Array<u64>>()
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 3);
    assert_eq!(dims.n_elements(0), 3);
    assert_eq!(dims.n_elements(1), 4);
    assert_eq!(dims.n_elements(2), 5);
    assert_eq!(data.len(), 60);
}

#[test]
fn array_3d_dynamic_nested() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    let unboxed = jlrs
        .dynamic_frame(|frame| {
            frame.frame(1, |frame| {
                let array = Value::array::<usize, _, _>(frame, (3, 4, 5))?;
                array.try_unbox::<Array<usize>>()
            })
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 3);
    assert_eq!(dims.n_elements(0), 3);
    assert_eq!(dims.n_elements(1), 4);
    assert_eq!(dims.n_elements(2), 5);
    assert_eq!(data.len(), 60);
}

#[test]
fn array_3d_dynamic_nested_dynamic() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    let unboxed = jlrs
        .dynamic_frame(|frame| {
            frame.dynamic_frame(|frame| {
                let array = Value::array::<isize, _, _>(frame, (3, 4, 5))?;
                array.try_unbox::<Array<isize>>()
            })
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 3);
    assert_eq!(dims.n_elements(0), 3);
    assert_eq!(dims.n_elements(1), 4);
    assert_eq!(dims.n_elements(2), 5);
    assert_eq!(data.len(), 60);
}

#[test]
fn array_4d() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    let unboxed = jlrs
        .frame(1, |frame| {
            let array = Value::array::<u8, _, _>(frame, (3, 4, 5, 6))?;
            array.try_unbox::<Array<u8>>()
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 4);
    assert_eq!(dims.n_elements(0), 3);
    assert_eq!(dims.n_elements(1), 4);
    assert_eq!(dims.n_elements(2), 5);
    assert_eq!(dims.n_elements(3), 6);
    assert_eq!(data.len(), 360);
}

#[test]
fn array_4d_nested() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    let unboxed = jlrs
        .frame(0, |frame| {
            frame.frame(1, |frame| {
                let array = Value::array::<u16, _, _>(frame, (3, 4, 5, 6))?;
                array.try_unbox::<Array<u16>>()
            })
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 4);
    assert_eq!(dims.n_elements(0), 3);
    assert_eq!(dims.n_elements(1), 4);
    assert_eq!(dims.n_elements(2), 5);
    assert_eq!(dims.n_elements(3), 6);
    assert_eq!(data.len(), 360);
}

#[test]
fn array_4d_nested_dynamic() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    let unboxed = jlrs
        .frame(0, |frame| {
            frame.dynamic_frame(|frame| {
                let array = Value::array::<u32, _, _>(frame, (3, 4, 5, 6))?;
                array.try_unbox::<Array<u32>>()
            })
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 4);
    assert_eq!(dims.n_elements(0), 3);
    assert_eq!(dims.n_elements(1), 4);
    assert_eq!(dims.n_elements(2), 5);
    assert_eq!(dims.n_elements(3), 6);
    assert_eq!(data.len(), 360);
}

#[test]
fn array_4d_dynamic() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    let unboxed = jlrs
        .dynamic_frame(|frame| {
            let array = Value::array::<u64, _, _>(frame, (3, 4, 5, 6))?;
            array.try_unbox::<Array<u64>>()
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 4);
    assert_eq!(dims.n_elements(0), 3);
    assert_eq!(dims.n_elements(1), 4);
    assert_eq!(dims.n_elements(2), 5);
    assert_eq!(dims.n_elements(3), 6);
    assert_eq!(data.len(), 360);
}

#[test]
fn array_4d_dynamic_nested() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    let unboxed = jlrs
        .dynamic_frame(|frame| {
            frame.frame(1, |frame| {
                let array = Value::array::<usize, _, _>(frame, (3, 4, 5, 6))?;
                array.try_unbox::<Array<usize>>()
            })
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 4);
    assert_eq!(dims.n_elements(0), 3);
    assert_eq!(dims.n_elements(1), 4);
    assert_eq!(dims.n_elements(2), 5);
    assert_eq!(dims.n_elements(3), 6);
    assert_eq!(data.len(), 360);
}

#[test]
fn array_4d_dynamic_nested_dynamic() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    let unboxed = jlrs
        .dynamic_frame(|frame| {
            frame.dynamic_frame(|frame| {
                let array = Value::array::<isize, _, _>(frame, (3, 4, 5, 6))?;
                array.try_unbox::<Array<isize>>()
            })
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 4);
    assert_eq!(dims.n_elements(0), 3);
    assert_eq!(dims.n_elements(1), 4);
    assert_eq!(dims.n_elements(2), 5);
    assert_eq!(dims.n_elements(3), 6);
    assert_eq!(data.len(), 360);
}
