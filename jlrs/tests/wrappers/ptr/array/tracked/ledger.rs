#[cfg(feature = "sync-rt")]
mod tests {
    use crate::util::JULIA;
    use jlrs::prelude::*;
    
    #[test]
    fn create_tracked_array() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|_, mut frame| {
                let arr =
                    Array::new::<f32, _, _, _>(&mut frame, (1, 2)).into_jlrs_result()?;
                assert!(arr.track(&frame).is_ok());
                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn alias_tracked_array() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|_, mut frame| {
                let arr =
                    Array::new::<f32, _, _, _>(&mut frame, (1, 2)).into_jlrs_result()?;
                let t1 = arr.track(&frame);
                let t2 = arr.track(&frame);
                assert!(t1.is_ok());
                assert!(t2.is_ok());
                Ok(())
            })
            .unwrap();
        });
    }
    
    #[test]
    fn create_mutable_tracked_array() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|_, mut frame| {
                let mut arr =
                    Array::new::<f32, _, _, _>(&mut frame, (1, 2)).into_jlrs_result()?;
                assert!(arr.track_mut(&frame).is_ok());
                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn cannot_alias_mutable_tracked_array() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|_, mut frame| {
                let mut arr =
                    Array::new::<f32, _, _, _>(&mut frame, (1, 2)).into_jlrs_result()?;
                let mut arr2 = arr;
                
                {
                    let t1 = arr.track_mut(&frame);
                    assert!(arr2.track_mut(&frame).is_err());
                    assert!(arr2.track(&frame).is_err());
                    assert!(t1.is_ok());
                }

                {
                    let t1 = arr.track(&frame);
                    assert!(arr2.track_mut(&frame).is_err());
                    assert!(arr2.track(&frame).is_ok());
                    assert!(t1.is_ok());
                }

                Ok(())
            })
            .unwrap();
        });
    }
    
    #[test]
    fn create_tracked_typed_array() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|_, mut frame| {
                let arr =
                    TypedArray::<f32>::new(&mut frame, (1, 2)).into_jlrs_result()?;
                assert!(arr.track(&frame).is_ok());
                Ok(())
            })
            .unwrap();
        });
    }
    
    #[test]
    fn alias_tracked_typed_array() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|_, mut frame| {
                let arr =
                    TypedArray::<f32>::new(&mut frame, (1, 2)).into_jlrs_result()?;
                let t1 = arr.track(&frame);
                let t2 = arr.track(&frame);
                assert!(t1.is_ok());
                assert!(t2.is_ok());
                Ok(())
            })
            .unwrap();
        });
    }
    
    #[test]
    fn create_mutable_tracked_typed_array() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|_, mut frame| {
                let mut arr =
                    TypedArray::<f32>::new(&mut frame, (1, 2)).into_jlrs_result()?;
                assert!(arr.track_mut(&frame).is_ok());
                Ok(())
            })
            .unwrap();
        });
    }
    
    #[test]
    fn cannot_alias_mutable_tracked_typed_array() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|_, mut frame| {
                let mut arr =
                    TypedArray::<f32>::new(&mut frame, (1, 2)).into_jlrs_result()?;
                let mut arr2 = arr;
                
                {
                    let t1 = arr.track_mut(&frame);
                    assert!(arr2.track_mut(&frame).is_err());
                    assert!(arr2.track(&frame).is_err());
                    assert!(t1.is_ok());
                }

                {
                    let t1 = arr.track(&frame);
                    assert!(arr2.track_mut(&frame).is_err());
                    assert!(arr2.track(&frame).is_ok());
                    assert!(t1.is_ok());
                }

                Ok(())
            })
            .unwrap();
        });
    }
}