use jlrs::prelude::*;
mod util;
use util::JULIA;

#[test]
fn array_1d() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let unboxed = jlrs
            .scope_with_slots(1, |_, frame| {
                let new_array = Array::new::<f32, _, _, _>(&mut *frame, 3)?.into_jlrs_result()?;
                new_array.cast::<Array>()?.copy_inline_data::<f32>()
            })
            .unwrap();

        let (data, dims) = unboxed.splat();
        assert_eq!(dims.n_dimensions(), 1);
        assert_eq!(dims.n_elements(0), 3);
        assert_eq!(data.len(), 3);
    });
}

#[test]
fn array_1d_output() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let unboxed = jlrs
            .scope_with_slots(1, |_, frame| {
                let array = frame
                    .result_scope_with_slots(0, |output, frame| {
                        let output = output.into_scope(frame);
                        Array::new::<f32, _, _, _>(output, 3)
                    })?
                    .into_jlrs_result()?;
                array.cast::<Array>()?.copy_inline_data::<f32>()
            })
            .unwrap();

        let (data, dims) = unboxed.splat();
        assert_eq!(dims.n_dimensions(), 1);
        assert_eq!(dims.n_elements(0), 3);
        assert_eq!(data.len(), 3);
    });
}

#[test]
fn array_1d_nested() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let unboxed = jlrs
            .scope_with_slots(0, |_, frame| {
                frame.scope_with_slots(1, |frame| {
                    let new_array =
                        Array::new::<f64, _, _, _>(&mut *frame, 3)?.into_jlrs_result()?;
                    new_array.cast::<Array>()?.copy_inline_data::<f64>()
                })
            })
            .unwrap();

        let (data, dims) = unboxed.splat();
        assert_eq!(dims.n_dimensions(), 1);
        assert_eq!(dims.n_elements(0), 3);
        assert_eq!(data.len(), 3);
    });
}

#[test]
fn array_1d_nested_dynamic() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let unboxed = jlrs
            .scope_with_slots(0, |_, frame| {
                frame.scope(|frame| {
                    let new_array =
                        Array::new::<i8, _, _, _>(&mut *frame, 3)?.into_jlrs_result()?;
                    new_array.cast::<Array>()?.copy_inline_data::<i8>()
                })
            })
            .unwrap();

        let (data, dims) = unboxed.splat();
        assert_eq!(dims.n_dimensions(), 1);
        assert_eq!(dims.n_elements(0), 3);
        assert_eq!(data.len(), 3);
    });
}

#[test]
fn array_1d_dynamic() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let unboxed = jlrs
            .scope(|_, frame| {
                let new_array = Array::new::<i16, _, _, _>(&mut *frame, 3)?.into_jlrs_result()?;
                new_array.cast::<Array>()?.copy_inline_data::<i16>()
            })
            .unwrap();

        let (data, dims) = unboxed.splat();
        assert_eq!(dims.n_dimensions(), 1);
        assert_eq!(dims.n_elements(0), 3);
        assert_eq!(data.len(), 3);
    });
}

#[test]
fn array_1d_dynamic_nested() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let unboxed = jlrs
            .scope(|_, frame| {
                frame.scope_with_slots(1, |frame| {
                    let new_array =
                        Array::new::<i32, _, _, _>(&mut *frame, 3)?.into_jlrs_result()?;
                    new_array.cast::<Array>()?.copy_inline_data::<i32>()
                })
            })
            .unwrap();

        let (data, dims) = unboxed.splat();
        assert_eq!(dims.n_dimensions(), 1);
        assert_eq!(dims.n_elements(0), 3);
        assert_eq!(data.len(), 3);
    });
}

#[test]
fn array_1d_dynamic_nested_dynamic() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let unboxed = jlrs
            .scope(|_, frame| {
                frame.scope(|frame| {
                    let new_array =
                        Array::new::<i64, _, _, _>(&mut *frame, 3)?.into_jlrs_result()?;
                    new_array.cast::<Array>()?.copy_inline_data::<i64>()
                })
            })
            .unwrap();

        let (data, dims) = unboxed.splat();
        assert_eq!(dims.n_dimensions(), 1);
        assert_eq!(dims.n_elements(0), 3);
        assert_eq!(data.len(), 3);
    });
}

#[test]
fn array_2d() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let unboxed = jlrs
            .scope_with_slots(1, |_, frame| {
                let new_array =
                    Array::new::<u8, _, _, _>(&mut *frame, (3, 4))?.into_jlrs_result()?;
                new_array.cast::<Array>()?.copy_inline_data::<u8>()
            })
            .unwrap();

        let (data, dims) = unboxed.splat();
        assert_eq!(dims.n_dimensions(), 2);
        assert_eq!(dims.n_elements(0), 3);
        assert_eq!(dims.n_elements(1), 4);
        assert_eq!(data.len(), 12);
    });
}

#[test]
fn array_2d_nested() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let unboxed = jlrs
            .scope_with_slots(0, |_, frame| {
                frame.scope_with_slots(1, |frame| {
                    let new_array =
                        Array::new::<u16, _, _, _>(&mut *frame, (3, 4))?.into_jlrs_result()?;
                    new_array.cast::<Array>()?.copy_inline_data::<u16>()
                })
            })
            .unwrap();

        let (data, dims) = unboxed.splat();
        assert_eq!(dims.n_dimensions(), 2);
        assert_eq!(dims.n_elements(0), 3);
        assert_eq!(dims.n_elements(1), 4);
        assert_eq!(data.len(), 12);
    });
}

#[test]
fn array_2d_nested_dynamic() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let unboxed = jlrs
            .scope_with_slots(0, |_, frame| {
                frame.scope(|frame| {
                    let new_array =
                        Array::new::<u32, _, _, _>(&mut *frame, (3, 4))?.into_jlrs_result()?;
                    new_array.cast::<Array>()?.copy_inline_data::<u32>()
                })
            })
            .unwrap();

        let (data, dims) = unboxed.splat();
        assert_eq!(dims.n_dimensions(), 2);
        assert_eq!(dims.n_elements(0), 3);
        assert_eq!(dims.n_elements(1), 4);
        assert_eq!(data.len(), 12);
    });
}

#[test]
fn array_2d_dynamic() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let unboxed = jlrs
            .scope(|_, frame| {
                let new_array =
                    Array::new::<u64, _, _, _>(&mut *frame, (3, 4))?.into_jlrs_result()?;
                new_array.cast::<Array>()?.copy_inline_data::<u64>()
            })
            .unwrap();

        let (data, dims) = unboxed.splat();
        assert_eq!(dims.n_dimensions(), 2);
        assert_eq!(dims.n_elements(0), 3);
        assert_eq!(dims.n_elements(1), 4);
        assert_eq!(data.len(), 12);
    });
}

#[test]
fn array_2d_dynamic_nested() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let unboxed = jlrs
            .scope(|_, frame| {
                frame.scope_with_slots(1, |frame| {
                    let new_array =
                        Array::new::<usize, _, _, _>(&mut *frame, (3, 4))?.into_jlrs_result()?;
                    new_array.cast::<Array>()?.copy_inline_data::<usize>()
                })
            })
            .unwrap();

        let (data, dims) = unboxed.splat();
        assert_eq!(dims.n_dimensions(), 2);
        assert_eq!(dims.n_elements(0), 3);
        assert_eq!(dims.n_elements(1), 4);
        assert_eq!(data.len(), 12);
    });
}

#[test]
fn array_2d_dynamic_nested_dynamic() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let unboxed = jlrs
            .scope(|_, frame| {
                frame.scope(|frame| {
                    let new_array =
                        Array::new::<isize, _, _, _>(&mut *frame, (3, 4))?.into_jlrs_result()?;
                    new_array.cast::<Array>()?.copy_inline_data::<isize>()
                })
            })
            .unwrap();

        let (data, dims) = unboxed.splat();
        assert_eq!(dims.n_dimensions(), 2);
        assert_eq!(dims.n_elements(0), 3);
        assert_eq!(dims.n_elements(1), 4);
        assert_eq!(data.len(), 12);
    });
}

#[test]
fn array_3d() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let unboxed = jlrs
            .scope_with_slots(1, |_, frame| {
                let new_array =
                    Array::new::<u8, _, _, _>(&mut *frame, (3, 4, 5))?.into_jlrs_result()?;
                new_array.cast::<Array>()?.copy_inline_data::<u8>()
            })
            .unwrap();

        let (data, dims) = unboxed.splat();
        assert_eq!(dims.n_dimensions(), 3);
        assert_eq!(dims.n_elements(0), 3);
        assert_eq!(dims.n_elements(1), 4);
        assert_eq!(dims.n_elements(2), 5);
        assert_eq!(data.len(), 60);
    });
}

#[test]
fn array_3d_nested() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let unboxed = jlrs
            .scope_with_slots(0, |_, frame| {
                frame.scope_with_slots(1, |frame| {
                    let new_array =
                        Array::new::<u16, _, _, _>(&mut *frame, (3, 4, 5))?.into_jlrs_result()?;
                    new_array.cast::<Array>()?.copy_inline_data::<u16>()
                })
            })
            .unwrap();

        let (data, dims) = unboxed.splat();
        assert_eq!(dims.n_dimensions(), 3);
        assert_eq!(dims.n_elements(0), 3);
        assert_eq!(dims.n_elements(1), 4);
        assert_eq!(dims.n_elements(2), 5);
        assert_eq!(data.len(), 60);
    });
}

#[test]
fn array_3d_nested_dynamic() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let unboxed = jlrs
            .scope_with_slots(0, |_, frame| {
                frame.scope(|frame| {
                    let new_array =
                        Array::new::<u32, _, _, _>(&mut *frame, (3, 4, 5))?.into_jlrs_result()?;
                    new_array.cast::<Array>()?.copy_inline_data::<u32>()
                })
            })
            .unwrap();

        let (data, dims) = unboxed.splat();
        assert_eq!(dims.n_dimensions(), 3);
        assert_eq!(dims.n_elements(0), 3);
        assert_eq!(dims.n_elements(1), 4);
        assert_eq!(dims.n_elements(2), 5);
        assert_eq!(data.len(), 60);
    });
}

#[test]
fn array_3d_dynamic() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let unboxed = jlrs
            .scope(|_, frame| {
                let new_array =
                    Array::new::<u64, _, _, _>(&mut *frame, (3, 4, 5))?.into_jlrs_result()?;
                new_array.cast::<Array>()?.copy_inline_data::<u64>()
            })
            .unwrap();

        let (data, dims) = unboxed.splat();
        assert_eq!(dims.n_dimensions(), 3);
        assert_eq!(dims.n_elements(0), 3);
        assert_eq!(dims.n_elements(1), 4);
        assert_eq!(dims.n_elements(2), 5);
        assert_eq!(data.len(), 60);
    });
}

#[test]
fn array_3d_dynamic_nested() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let unboxed = jlrs
            .scope(|_, frame| {
                frame.scope_with_slots(1, |frame| {
                    let new_array =
                        Array::new::<usize, _, _, _>(&mut *frame, (3, 4, 5))?.into_jlrs_result()?;
                    new_array.cast::<Array>()?.copy_inline_data::<usize>()
                })
            })
            .unwrap();

        let (data, dims) = unboxed.splat();
        assert_eq!(dims.n_dimensions(), 3);
        assert_eq!(dims.n_elements(0), 3);
        assert_eq!(dims.n_elements(1), 4);
        assert_eq!(dims.n_elements(2), 5);
        assert_eq!(data.len(), 60);
    });
}

#[test]
fn array_3d_dynamic_nested_dynamic() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let unboxed = jlrs
            .scope(|_, frame| {
                frame.scope(|frame| {
                    let new_array =
                        Array::new::<isize, _, _, _>(&mut *frame, (3, 4, 5))?.into_jlrs_result()?;
                    new_array.cast::<Array>()?.copy_inline_data::<isize>()
                })
            })
            .unwrap();

        let (data, dims) = unboxed.splat();
        assert_eq!(dims.n_dimensions(), 3);
        assert_eq!(dims.n_elements(0), 3);
        assert_eq!(dims.n_elements(1), 4);
        assert_eq!(dims.n_elements(2), 5);
        assert_eq!(data.len(), 60);
    });
}

#[test]
fn array_4d() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let unboxed = jlrs
            .scope_with_slots(1, |_, frame| {
                let new_array =
                    Array::new::<u8, _, _, _>(&mut *frame, (3, 4, 5, 6))?.into_jlrs_result()?;
                new_array.cast::<Array>()?.copy_inline_data::<u8>()
            })
            .unwrap();

        let (data, dims) = unboxed.splat();
        assert_eq!(dims.n_dimensions(), 4);
        assert_eq!(dims.n_elements(0), 3);
        assert_eq!(dims.n_elements(1), 4);
        assert_eq!(dims.n_elements(2), 5);
        assert_eq!(dims.n_elements(3), 6);
        assert_eq!(data.len(), 360);
    });
}

#[test]
fn array_4d_nested() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let unboxed = jlrs
            .scope_with_slots(0, |_, frame| {
                frame.scope_with_slots(1, |frame| {
                    let new_array = Array::new::<u16, _, _, _>(&mut *frame, (3, 4, 5, 6))?
                        .into_jlrs_result()?;
                    new_array.cast::<Array>()?.copy_inline_data::<u16>()
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
    });
}

#[test]
fn array_4d_nested_dynamic() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let unboxed = jlrs
            .scope_with_slots(0, |_, frame| {
                frame.scope(|frame| {
                    let new_array = Array::new::<u32, _, _, _>(&mut *frame, (3, 4, 5, 6))?
                        .into_jlrs_result()?;
                    new_array.cast::<Array>()?.copy_inline_data::<u32>()
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
    });
}

#[test]
fn array_4d_dynamic() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let unboxed = jlrs
            .scope(|_, frame| {
                let new_array =
                    Array::new::<u64, _, _, _>(&mut *frame, (3, 4, 5, 6))?.into_jlrs_result()?;
                new_array.cast::<Array>()?.copy_inline_data::<u64>()
            })
            .unwrap();

        let (data, dims) = unboxed.splat();
        assert_eq!(dims.n_dimensions(), 4);
        assert_eq!(dims.n_elements(0), 3);
        assert_eq!(dims.n_elements(1), 4);
        assert_eq!(dims.n_elements(2), 5);
        assert_eq!(dims.n_elements(3), 6);
        assert_eq!(data.len(), 360);
    });
}

#[test]
fn array_4d_dynamic_nested() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let unboxed = jlrs
            .scope(|_, frame| {
                frame.scope_with_slots(1, |frame| {
                    let new_array = Array::new::<usize, _, _, _>(&mut *frame, (3, 4, 5, 6))?
                        .into_jlrs_result()?;
                    new_array.cast::<Array>()?.copy_inline_data::<usize>()
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
    });
}

#[test]
fn array_4d_dynamic_nested_dynamic() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let unboxed = jlrs
            .scope(|_, frame| {
                frame.scope(|frame| {
                    let new_array = Array::new::<isize, _, _, _>(&mut *frame, (3, 4, 5, 6))?
                        .into_jlrs_result()?;
                    new_array.cast::<Array>()?.copy_inline_data::<isize>()
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
    });
}

#[test]
fn array_of_bools() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let unboxed = jlrs
            .scope(|_, frame| {
                frame.scope(|frame| {
                    let new_array = Array::new::<bool, _, _, _>(&mut *frame, (3, 4, 5, 6))?
                        .into_jlrs_result()?;
                    new_array.cast::<Array>()?.copy_inline_data::<bool>()
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
    });
}

#[test]
fn array_of_chars() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let unboxed = jlrs
            .scope(|_, frame| {
                frame.scope(|frame| {
                    let new_array = Array::new::<char, _, _, _>(&mut *frame, (3, 4, 5, 6))?
                        .into_jlrs_result()?;
                    new_array.cast::<Array>()?.copy_inline_data::<char>()
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
    });
}
