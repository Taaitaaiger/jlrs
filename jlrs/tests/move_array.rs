use jlrs::prelude::*;
mod util;
use util::JULIA;

#[test]
fn move_array_1d() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let unboxed = jlrs
            .frame(1, |_, frame| {
                let data = vec![1.0f32, 2., 3.];
                let array = Value::move_array(frame, data, 3)?;
                array.try_unbox::<CopiedArray<f32>>()
            })
            .unwrap();

        let (data, dims) = unboxed.splat();
        assert_eq!(dims.n_dimensions(), 1);
        assert_eq!(dims.n_elements(0), 3);
        assert_eq!(data, vec![1., 2., 3.]);
    });
}

#[test]
fn move_array_1d_nested() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let unboxed = jlrs
            .frame(0, |_, frame| {
                frame.frame(1, |frame| {
                    let data = vec![1.0f64, 2., 3.];
                    let array = Value::move_array(frame, data, 3)?;
                    array.try_unbox::<CopiedArray<f64>>()
                })
            })
            .unwrap();

        let (data, dims) = unboxed.splat();
        assert_eq!(dims.n_dimensions(), 1);
        assert_eq!(dims.n_elements(0), 3);
        assert_eq!(data, vec![1., 2., 3.]);
    });
}

#[test]
fn move_array_1d_nested_dynamic() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let unboxed = jlrs
            .frame(0, |_, frame| {
                frame.dynamic_frame(|frame| {
                    let data = vec![1i8, 2, 3];
                    let array = Value::move_array(frame, data, 3)?;
                    array.try_unbox::<CopiedArray<i8>>()
                })
            })
            .unwrap();

        let (data, dims) = unboxed.splat();
        assert_eq!(dims.n_dimensions(), 1);
        assert_eq!(dims.n_elements(0), 3);
        assert_eq!(data, vec![1, 2, 3]);
    });
}

#[test]
fn move_array_1d_dynamic() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let unboxed = jlrs
            .dynamic_frame(|_, frame| {
                let data = vec![1i16, 2, 3];
                let array = Value::move_array(frame, data, 3)?;
                array.try_unbox::<CopiedArray<i16>>()
            })
            .unwrap();

        let (data, dims) = unboxed.splat();
        assert_eq!(dims.n_dimensions(), 1);
        assert_eq!(dims.n_elements(0), 3);
        assert_eq!(data, vec![1, 2, 3]);
    });
}

#[test]
fn move_array_1d_dynamic_nested() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let unboxed = jlrs
            .dynamic_frame(|_, frame| {
                frame.frame(1, |frame| {
                    let data = vec![1i32, 2, 3];
                    let array = Value::move_array(frame, data, 3)?;
                    array.try_unbox::<CopiedArray<i32>>()
                })
            })
            .unwrap();

        let (data, dims) = unboxed.splat();
        assert_eq!(dims.n_dimensions(), 1);
        assert_eq!(dims.n_elements(0), 3);
        assert_eq!(data, vec![1, 2, 3]);
    });
}

#[test]
fn move_array_1d_dynamic_nested_dynamic() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let unboxed = jlrs
            .dynamic_frame(|_, frame| {
                frame.dynamic_frame(|frame| {
                    let data = vec![1i64, 2, 3];
                    let array = Value::move_array(frame, data, 3)?;
                    array.try_unbox::<CopiedArray<i64>>()
                })
            })
            .unwrap();

        let (data, dims) = unboxed.splat();
        assert_eq!(dims.n_dimensions(), 1);
        assert_eq!(dims.n_elements(0), 3);
        assert_eq!(data, vec![1, 2, 3]);
    });
}

#[test]
fn move_array_2d() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let unboxed = jlrs
            .frame(1, |_, frame| {
                let data = vec![1u8, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4];
                let array = Value::move_array(frame, data, (3, 4))?;
                array.try_unbox::<CopiedArray<u8>>()
            })
            .unwrap();

        let (data, dims) = unboxed.splat();
        assert_eq!(dims.n_dimensions(), 2);
        assert_eq!(dims.n_elements(0), 3);
        assert_eq!(dims.n_elements(1), 4);
        assert_eq!(data, vec![1u8, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4]);
    });
}

#[test]
fn move_array_2d_nested() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let unboxed = jlrs
            .frame(0, |_, frame| {
                frame.frame(1, |frame| {
                    let data = vec![1u16, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4];
                    let array = Value::move_array(frame, data, (3, 4))?;
                    array.try_unbox::<CopiedArray<u16>>()
                })
            })
            .unwrap();

        let (data, dims) = unboxed.splat();
        assert_eq!(dims.n_dimensions(), 2);
        assert_eq!(dims.n_elements(0), 3);
        assert_eq!(dims.n_elements(1), 4);
        assert_eq!(data, vec![1u16, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4]);
    });
}

#[test]
fn move_array_2d_nested_dynamic() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let unboxed = jlrs
            .frame(0, |_, frame| {
                frame.dynamic_frame(|frame| {
                    let data = vec![1u32, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4];
                    let array = Value::move_array(frame, data, (3, 4))?;
                    array.try_unbox::<CopiedArray<u32>>()
                })
            })
            .unwrap();

        let (data, dims) = unboxed.splat();
        assert_eq!(dims.n_dimensions(), 2);
        assert_eq!(dims.n_elements(0), 3);
        assert_eq!(dims.n_elements(1), 4);
        assert_eq!(data, vec![1u32, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4]);
    });
}

#[test]
fn move_array_2d_dynamic() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let unboxed = jlrs
            .dynamic_frame(|_, frame| {
                let data = vec![1u64, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4];
                let array = Value::move_array(frame, data, (3, 4))?;
                array.try_unbox::<CopiedArray<u64>>()
            })
            .unwrap();

        let (data, dims) = unboxed.splat();
        assert_eq!(dims.n_dimensions(), 2);
        assert_eq!(dims.n_elements(0), 3);
        assert_eq!(dims.n_elements(1), 4);
        assert_eq!(data, vec![1u64, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4]);
    });
}

#[test]
fn move_array_2d_dynamic_nested() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let unboxed = jlrs
            .dynamic_frame(|_, frame| {
                frame.frame(1, |frame| {
                    let data = vec![1usize, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4];
                    let array = Value::move_array(frame, data, (3, 4))?;
                    array.try_unbox::<CopiedArray<usize>>()
                })
            })
            .unwrap();

        let (data, dims) = unboxed.splat();
        assert_eq!(dims.n_dimensions(), 2);
        assert_eq!(dims.n_elements(0), 3);
        assert_eq!(dims.n_elements(1), 4);
        assert_eq!(data, vec![1usize, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4]);
    });
}

#[test]
fn move_array_2d_dynamic_nested_dynamic() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let unboxed = jlrs
            .dynamic_frame(|_, frame| {
                frame.dynamic_frame(|frame| {
                    let data = vec![1isize, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4];
                    let array = Value::move_array(frame, data, (3, 4))?;
                    array.try_unbox::<CopiedArray<isize>>()
                })
            })
            .unwrap();

        let (data, dims) = unboxed.splat();
        assert_eq!(dims.n_dimensions(), 2);
        assert_eq!(dims.n_elements(0), 3);
        assert_eq!(dims.n_elements(1), 4);
        assert_eq!(data, vec![1isize, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4]);
    });
}
