mod util;

#[cfg(feature = "local-rt")]
mod tests {
    use jlrs::{
        data::{
            layout::valid_layout::ValidLayout,
            managed::{
                simple_vector::{SimpleVector, SimpleVectorRef},
                union_all::UnionAll,
            },
        },
        prelude::*,
    };

    use crate::util::JULIA;

    fn create_simple_vector() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let svec = SimpleVector::with_capacity(&mut frame, 1);
                    assert!(svec.as_value().is::<SimpleVector>());
                    Ok(())
                })
                .unwrap();
        })
    }

    fn set_simple_vector_contents() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let svec = unsafe { SimpleVector::with_capacity_uninit(&mut frame, 1) };
                    let value = Value::new(&mut frame, 1usize);

                    unsafe {
                        let data = svec.data();
                        assert!(data.set(0, Some(value)).is_ok());
                    }
                    Ok(())
                })
                .unwrap();
        })
    }

    fn set_simple_vector_contents_unrestricted() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let svec = unsafe { SimpleVector::with_capacity_uninit(&mut frame, 1) };
                    let value = Value::new(&mut frame, 1usize);

                    unsafe {
                        let data = svec.data();
                        assert!(data.set(0, Some(value)).is_ok());
                    }
                    Ok(())
                })
                .unwrap();
        })
    }

    fn typed_simple_vector_contents() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let svec = unsafe { SimpleVector::with_capacity_uninit(&mut frame, 1) };
                    let sym = Symbol::new(&frame, "Foo");

                    unsafe {
                        let data = svec.data();
                        assert!(data.set(0, Some(sym.as_value())).is_ok());
                    }

                    unsafe {
                        let data = svec.typed_data_unchecked::<Symbol>();
                        assert_eq!(data.as_atomic_slice().assume_immutable_non_null()[0], sym);
                    }
                    Ok(())
                })
                .unwrap();
        })
    }

    fn set_simple_vector_contents_oob() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let svec = unsafe { SimpleVector::with_capacity_uninit(&mut frame, 1) };
                    let value = Value::new(&mut frame, 1usize);

                    unsafe {
                        let data = svec.data();
                        assert!(data.set(1, Some(value)).is_err());
                    }
                    Ok(())
                })
                .unwrap();
        })
    }

    fn set_simple_vector_contents_unrestricted_oob() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let svec = unsafe { SimpleVector::with_capacity_uninit(&mut frame, 1) };
                    let value = Value::new(&mut frame, 1usize);

                    unsafe {
                        let data = svec.data();
                        assert!(data.set(1, Some(value)).is_err());
                    }
                    Ok(())
                })
                .unwrap();
        })
    }

    fn extend_lifetime() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame).scope(|mut frame| {
                let output = frame.output();
                frame.scope(|mut frame| {
                    let svec = SimpleVector::with_capacity(&mut frame, 0).clone();
                    svec.root(output)
                });
            });
        })
    }

    fn empty() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame).scope(|mut frame| {
                let svec = SimpleVector::with_capacity(&mut frame, 0);
                assert_eq!(svec.as_value(), SimpleVector::emptysvec(&frame));
            });
        })
    }

    fn root_ref() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let res = unsafe { SimpleVector::emptysvec(&frame).as_ref().root(&mut frame) };
                    assert_eq!(res.len(), 0);

                    Ok(())
                })
                .unwrap();
        })
    }

    fn valid_layout() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|frame| {
                    let res = SimpleVector::emptysvec(&frame);
                    assert!(res.as_value().is::<SimpleVector>());
                    assert!(SimpleVectorRef::valid_layout(
                        res.as_value().datatype().as_value()
                    ));

                    let value = DataType::unionall_type(&frame).as_value();
                    assert!(!value.is::<SimpleVector>());
                    assert!(!SimpleVectorRef::valid_layout(
                        UnionAll::array_type(&frame).as_value()
                    ));

                    Ok(())
                })
                .unwrap();
        })
    }

    fn debug_fmt() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|frame| {
                    let res = SimpleVector::emptysvec(&frame);
                    let fmt = format!("{:?}", res);
                    assert_eq!(fmt, "svec()");

                    Ok(())
                })
                .unwrap();
        })
    }

    #[test]
    fn simple_vector_tests() {
        create_simple_vector();
        set_simple_vector_contents();
        set_simple_vector_contents_unrestricted();
        typed_simple_vector_contents();
        set_simple_vector_contents_oob();
        set_simple_vector_contents_unrestricted_oob();
        extend_lifetime();
        empty();
        root_ref();
        valid_layout();
        debug_fmt();
    }
}
