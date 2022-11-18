mod util;

#[cfg(feature = "sync-rt")]
#[cfg(not(all(target_os = "windows", feature = "lts")))]
mod tests {
    use crate::util::JULIA;
    use jlrs::prelude::*;

    fn create_tracked_array() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let arr = Array::new::<f32, _, _>(frame.as_extended_target(), (1, 2))
                        .into_jlrs_result()?;
                    assert!(arr.track().is_ok());
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
                    let arr = Array::new::<f32, _, _>(frame.as_extended_target(), (1, 2))
                        .into_jlrs_result()?;
                    let t1 = arr.track();
                    let t2 = arr.track();
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
                .scope(|mut frame| {
                    let mut arr = Array::new::<f32, _, _>(frame.as_extended_target(), (1, 2))
                        .into_jlrs_result()?;
                    assert!(arr.track_mut().is_ok());
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
                .scope(|mut frame| {
                    let mut arr = Array::new::<f32, _, _>(frame.as_extended_target(), (1, 2))
                        .into_jlrs_result()?;
                    let mut arr2 = arr;

                    {
                        let t1 = arr.track_mut();
                        assert!(arr2.track_mut().is_err());
                        assert!(arr2.track().is_err());
                        assert!(t1.is_ok());
                    }

                    {
                        let t1 = arr.track();
                        assert!(arr2.track_mut().is_err());
                        assert!(arr2.track().is_ok());
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
                    let arr = TypedArray::<f32>::new(frame.as_extended_target(), (1, 2))
                        .into_jlrs_result()?;
                    assert!(arr.track().is_ok());
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
                    let arr = TypedArray::<f32>::new(frame.as_extended_target(), (1, 2))
                        .into_jlrs_result()?;
                    let t1 = arr.track();
                    let t2 = arr.track();
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
                .scope(|mut frame| {
                    let mut arr = TypedArray::<f32>::new(frame.as_extended_target(), (1, 2))
                        .into_jlrs_result()?;
                    assert!(arr.track_mut().is_ok());
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
                .scope(|mut frame| {
                    let mut arr = TypedArray::<f32>::new(frame.as_extended_target(), (1, 2))
                        .into_jlrs_result()?;
                    let mut arr2 = arr;

                    {
                        let t1 = arr.track_mut();
                        assert!(arr2.track_mut().is_err());
                        assert!(arr2.track().is_err());
                        assert!(t1.is_ok());
                    }

                    {
                        let t1 = arr.track();
                        assert!(arr2.track_mut().is_err());
                        assert!(arr2.track().is_ok());
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
