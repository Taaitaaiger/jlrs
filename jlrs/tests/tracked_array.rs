mod util;

#[cfg(feature = "sync-rt")]
mod tests {
    use jlrs::prelude::*;

    use crate::util::JULIA;

    fn create_tracked_array() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let arr = Array::new::<f32, _, _>(&mut frame, (1, 2)).into_jlrs_result()?;
                    assert!(arr.track_shared().is_ok());
                    Ok(())
                })
                .unwrap();
        });
    }

    fn alias_tracked_array() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let arr = Array::new::<f32, _, _>(&mut frame, (1, 2)).into_jlrs_result()?;
                    let t1 = arr.track_shared();
                    let t2 = arr.track_shared();
                    assert!(t1.is_ok());
                    assert!(t2.is_ok());
                    Ok(())
                })
                .unwrap();
        });
    }

    fn create_mutable_tracked_array() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let mut arr = Array::new::<f32, _, _>(&mut frame, (1, 2)).into_jlrs_result()?;
                    assert!(arr.track_exclusive().is_ok());
                    Ok(())
                })
                .unwrap();
        });
    }

    fn cannot_alias_mutable_tracked_array() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let mut arr = Array::new::<f32, _, _>(&mut frame, (1, 2)).into_jlrs_result()?;
                    let mut arr2 = arr;

                    {
                        let t1 = arr.track_exclusive();
                        assert!(arr2.track_exclusive().is_err());
                        assert!(arr2.track_shared().is_err());
                        assert!(t1.is_ok());
                    }

                    {
                        let t1 = arr.track_shared();
                        assert!(arr2.track_exclusive().is_err());
                        assert!(arr2.track_shared().is_ok());
                        assert!(t1.is_ok());
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn create_tracked_typed_array() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let arr = TypedArray::<f32>::new(&mut frame, (1, 2)).into_jlrs_result()?;
                    assert!(arr.track_shared().is_ok());
                    Ok(())
                })
                .unwrap();
        });
    }

    fn alias_tracked_typed_array() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let arr = TypedArray::<f32>::new(&mut frame, (1, 2)).into_jlrs_result()?;
                    let t1 = arr.track_shared();
                    let t2 = arr.track_shared();
                    assert!(t1.is_ok());
                    assert!(t2.is_ok());
                    Ok(())
                })
                .unwrap();
        });
    }

    fn create_mutable_tracked_typed_array() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let mut arr = TypedArray::<f32>::new(&mut frame, (1, 2)).into_jlrs_result()?;
                    assert!(arr.track_exclusive().is_ok());
                    Ok(())
                })
                .unwrap();
        });
    }

    fn cannot_alias_mutable_tracked_typed_array() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let mut arr = TypedArray::<f32>::new(&mut frame, (1, 2)).into_jlrs_result()?;
                    let mut arr2 = arr;

                    {
                        let t1 = arr.track_exclusive();
                        assert!(arr2.track_exclusive().is_err());
                        assert!(arr2.track_shared().is_err());
                        assert!(t1.is_ok());
                    }

                    {
                        let t1 = arr.track_shared();
                        assert!(arr2.track_exclusive().is_err());
                        assert!(arr2.track_shared().is_ok());
                        assert!(t1.is_ok());
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    #[test]
    fn tracked_array_tests() {
        create_tracked_array();
        alias_tracked_array();
        create_mutable_tracked_array();
        cannot_alias_mutable_tracked_array();
        create_tracked_typed_array();
        alias_tracked_typed_array();
        create_mutable_tracked_typed_array();
        cannot_alias_mutable_tracked_typed_array();
    }
}
