#[cfg(feature = "local-rt")]
pub(crate) mod tests {
    use jlrs::{
        data::managed::array::{RankedArray, TypedRankedArray},
        prelude::*,
    };

    use crate::util::JULIA;

    fn array_has_rank_s() {
        assert!(!Array::has_rank_s());
        assert!(!TypedArray::<f32>::has_rank_s());
        assert!(RankedArray::<2>::has_rank_s());
        assert!(TypedRankedArray::<f32, 2>::has_rank_s());
    }

    fn array_set_rank() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let arr = TypedArray::<f32>::new(&mut frame, (1, 2)).unwrap();
                    assert_eq!(arr.generic_rank(), -1);
                    assert!(!arr.has_rank());
                    arr.assert_rank();
                    assert!(arr.set_rank::<1>().is_err());
                    let arr = arr.set_rank::<2>();
                    assert!(arr.is_ok());
                    let arr = arr.unwrap();
                    assert!(arr.has_rank());
                    arr.assert_rank();
                    assert_eq!(arr.generic_rank(), 2);

                    Ok(())
                })
                .unwrap();
        });
    }

    fn array_set_rank_err() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let arr = TypedArray::<f32>::new(&mut frame, (1, 2)).unwrap();
                    assert_eq!(arr.generic_rank(), -1);
                    assert!(!arr.has_rank());
                    arr.assert_rank();
                    assert!(arr.set_rank::<1>().is_err());
                    let arr = arr.set_rank::<3>();
                    assert!(arr.is_err());

                    Ok(())
                })
                .unwrap();
        });
    }

    fn array_set_rank_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let arr = TypedArray::<f32>::new(&mut frame, (1, 2)).unwrap();
                    assert_eq!(arr.generic_rank(), -1);
                    assert!(!arr.has_rank());
                    arr.assert_rank();
                    let arr = unsafe { arr.set_rank_unchecked::<2>() };
                    assert!(arr.has_rank());
                    arr.assert_rank();
                    assert_eq!(arr.generic_rank(), 2);

                    Ok(())
                })
                .unwrap();
        });
    }

    fn array_forget_rank() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let arr = TypedRankedArray::<f32, 2>::new(&mut frame, (1, 2)).unwrap();

                    assert_eq!(arr.generic_rank(), 2);
                    let arr = arr.forget_rank();
                    assert_eq!(arr.generic_rank(), -1);

                    Ok(())
                })
                .unwrap();
        });
    }

    fn array_set_type() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let dt = DataType::float32_type(&frame).as_value();
                    let arr = RankedArray::<2>::new_for(&mut frame, dt, (1, 2)).unwrap();
                    assert!(!arr.has_constrained_type());
                    let arr = arr.set_type::<f32>();
                    assert!(arr.is_ok());

                    Ok(())
                })
                .unwrap();
        });
    }

    fn array_set_type_err() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let dt = DataType::float32_type(&frame).as_value();
                    let arr = RankedArray::<2>::new_for(&mut frame, dt, (1, 2)).unwrap();
                    assert!(!arr.has_constrained_type());
                    let arr = arr.set_type::<f64>();
                    assert!(arr.is_err());

                    Ok(())
                })
                .unwrap();
        });
    }

    fn array_set_type_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let dt = DataType::float32_type(&frame).as_value();
                    let arr = RankedArray::<2>::new_for(&mut frame, dt, (1, 2)).unwrap();
                    assert!(!arr.has_constrained_type());
                    let arr = unsafe { arr.set_type_unchecked::<f32>() };
                    arr.assert_type();

                    Ok(())
                })
                .unwrap();
        });
    }

    fn array_forget_type() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let arr = TypedRankedArray::<f32, 2>::new(&mut frame, (1, 2)).unwrap();

                    assert!(arr.has_constrained_type());
                    assert!(!Array::has_constrained_type_s());
                    assert!(TypedArray::<f32>::has_constrained_type_s());
                    assert!(!RankedArray::<2>::has_constrained_type_s());
                    assert!(TypedRankedArray::<f32, 2>::has_constrained_type_s());

                    let arr = arr.forget_type();
                    assert!(!arr.has_constrained_type());

                    Ok(())
                })
                .unwrap();
        });
    }

    pub(crate) fn array_conversion_tests() {
        array_has_rank_s();
        array_set_rank();
        array_set_rank_err();
        array_set_rank_unchecked();
        array_forget_rank();
        array_set_type();
        array_set_type_err();
        array_set_type_unchecked();
        array_forget_type();
    }
}
