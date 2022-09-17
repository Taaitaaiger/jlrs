#[cfg(feature = "sync-rt")]
mod tests {
    use crate::util::JULIA;
    use jlrs::{
        layout::valid_layout::ValidLayout,
        prelude::*,
        wrappers::ptr::{
            simple_vector::{SimpleVector, SimpleVectorRef},
            symbol::SymbolRef,
            union_all::UnionAll,
        },
    };

    #[test]
    fn create_simple_vector() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|_, mut frame| {
                let svec = SimpleVector::with_capacity(&mut frame, 1);
                assert!(svec.as_value().is::<SimpleVector>());
                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn ignore_undefined() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|_, mut frame| {
                let svec = SimpleVector::with_capacity(&mut frame, 1);

                unsafe {
                    assert!(svec.unrestricted_typed_data::<SymbolRef>().is_ok());
                }
                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn set_simple_vector_contents() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|_, mut frame| {
                let svec = unsafe { SimpleVector::with_capacity_uninit(&mut frame, 1) };
                let value = Value::new(&mut frame, 1usize);
                let mut data = svec.data(&mut frame);

                unsafe {
                    assert!(data.set(0, Some(value)).is_ok());
                }
                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn set_simple_vector_contents_unrestricted() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|_, mut frame| {
                let svec = unsafe { SimpleVector::with_capacity_uninit(&mut frame, 1) };
                let value = Value::new(&mut frame, 1usize);

                unsafe {
                    let mut data = svec.unrestricted_data();
                    assert!(data.set(0, Some(value)).is_ok());
                }
                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn typed_simple_vector_contents() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, mut frame| {
                let svec = unsafe { SimpleVector::with_capacity_uninit(&mut frame, 1) };
                let sym = Symbol::new(global, "Foo");
                let mut data = svec.data(&mut frame);

                unsafe {
                    assert!(data.set(0, Some(sym.as_value())).is_ok());
                }

                let data = svec.typed_data::<SymbolRef, _>(&mut frame);
                assert!(data.is_ok());

                let data = svec.typed_data::<ArrayRef, _>(&mut frame);
                assert!(data.is_err());

                unsafe {
                    let data = svec.unrestricted_typed_data::<ArrayRef>();
                    assert!(data.is_err());
                }

                unsafe {
                    let data = svec.typed_data_unchecked::<SymbolRef, _>(&mut frame);
                    assert_eq!(data.as_slice()[0].wrapper_unchecked(), sym);
                }
                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn set_simple_vector_contents_oob() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|_, mut frame| {
                let svec = unsafe { SimpleVector::with_capacity_uninit(&mut frame, 1) };
                let value = Value::new(&mut frame, 1usize);
                let mut data = svec.data(&mut frame);

                unsafe {
                    assert!(data.set(1, Some(value)).is_err());
                }
                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn set_simple_vector_contents_unrestricted_oob() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|_, mut frame| {
                let svec = unsafe { SimpleVector::with_capacity_uninit(&mut frame, 1) };
                let value = Value::new(&mut frame, 1usize);

                unsafe {
                    let mut data = svec.unrestricted_data();
                    assert!(data.set(1, Some(value)).is_err());
                }
                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn extend_lifetime() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|_, mut frame| {
                let output = frame.output();
                frame
                    .scope(|mut frame| {
                        let svec = SimpleVector::with_capacity(&mut frame, 0).clone();
                        Ok(svec.root(output))
                    })
                    .unwrap();

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn empty() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, mut frame| {
                let svec = SimpleVector::with_capacity(&mut frame, 0);
                assert_eq!(svec.as_value(), SimpleVector::emptysvec(global));

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn root_ref() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, mut frame| {
                let res = unsafe { SimpleVector::emptysvec(global).as_ref().root(&mut frame) };
                assert!(res.is_ok());

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn valid_layout() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, _| {
                let res = SimpleVector::emptysvec(global);
                assert!(res.as_value().is::<SimpleVector>());
                assert!(SimpleVectorRef::valid_layout(
                    res.as_value().datatype().as_value()
                ));

                let value = DataType::unionall_type(global).as_value();
                assert!(!value.is::<SimpleVector>());
                assert!(!SimpleVectorRef::valid_layout(
                    UnionAll::array_type(global).as_value()
                ));

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn debug_fmt() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, _| {
                let res = SimpleVector::emptysvec(global);
                let fmt = format!("{:?}", res);
                assert_eq!(fmt, "svec()");

                Ok(())
            })
            .unwrap();
        })
    }
}
