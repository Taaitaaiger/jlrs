mod util;
#[cfg(feature = "sync-rt")]
#[cfg(not(all(target_os = "windows", feature = "lts")))]
mod tests {
    use super::util::JULIA;
    use jlrs::prelude::*;
    use jlrs::wrappers::ptr::ArrayRef;
    use jlrs::wrappers::ptr::DataTypeRef;
    use jlrs::wrappers::ptr::TypedArrayRef;
    use jlrs::wrappers::ptr::ValueRef;

    #[test]
    fn access_raw_fields_bits() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, frame| {
                let ty = unsafe {
                    Module::main(global)
                        .submodule_ref("JlrsTests")?
                        .wrapper_unchecked()
                        .global_ref("NoUnionsBits")?
                        .value_unchecked()
                };
                let arg1 = Value::new(&mut *frame, 3i16)?;
                let arg2 = Value::new(&mut *frame, -3i32)?;
                let instance = ty
                    .cast::<DataType>()?
                    .instantiate(&mut *frame, &mut [arg1, arg2])?
                    .into_jlrs_result()?;

                let a = instance.get_raw_field::<i16, _>("a")?;
                assert_eq!(a, 3);

                let b = instance.get_raw_field::<i32, _>("b")?;
                assert_eq!(b, -3);

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn access_raw_fields_bits_and_ptr() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, frame| {
                let ty = unsafe {
                    Module::main(global)
                        .submodule_ref("JlrsTests")?
                        .wrapper_unchecked()
                        .global_ref("NoUnionsBitsPtr")?
                        .value_unchecked()
                };
                let arg1 = Value::new(&mut *frame, 3i16)?;
                let arg2 = DataType::bool_type(global);
                let instance = ty
                    .cast::<DataType>()?
                    .instantiate(&mut *frame, &mut [arg1, arg2.as_value()])?
                    .into_jlrs_result()?;

                let a = instance.get_raw_field::<i16, _>("a")?;
                assert_eq!(a, 3);

                let b = instance.get_raw_field::<DataTypeRef, _>("b")?;
                assert_eq!(unsafe { b.wrapper_unchecked() }, arg2);

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn access_raw_fields_bits_and_bits_union() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, frame| {
                let ty = unsafe {
                    Module::main(global)
                        .submodule_ref("JlrsTests")?
                        .wrapper_unchecked()
                        .global_ref("BitsBitsUnion")?
                        .value_unchecked()
                };
                let arg1 = Value::new(&mut *frame, 3i16)?;
                let arg2 = Value::new(&mut *frame, -3i32)?;
                let instance = ty
                    .cast::<DataType>()?
                    .instantiate(&mut *frame, &mut [arg1, arg2])?
                    .into_jlrs_result()?;

                let a = instance.get_raw_field::<i16, _>("a")?;
                assert_eq!(a, 3);

                let b = instance.get_raw_field::<i32, _>("b")?;
                assert_eq!(b, -3);

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn access_raw_fields_ptr_and_bits_union() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, frame| {
                let ty = unsafe {
                    Module::main(global)
                        .submodule_ref("JlrsTests")?
                        .wrapper_unchecked()
                        .global_ref("PtrBitsUnion")?
                        .value_unchecked()
                };
                let arg1 = DataType::bool_type(global);
                let arg2 = Value::new(&mut *frame, -3i32)?;
                let instance = ty
                    .cast::<DataType>()?
                    .instantiate(&mut *frame, &mut [arg1.as_value(), arg2])?
                    .into_jlrs_result()?;

                let a = instance.get_raw_field::<DataTypeRef, _>("a")?;
                assert_eq!(unsafe { a.wrapper_unchecked() }, arg1);

                let b = instance.get_raw_field::<i32, _>("b")?;
                assert_eq!(b, -3);

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn access_raw_fields_ptr_and_non_bits_union() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, frame| {
                let ty = unsafe {
                    Module::main(global)
                        .submodule_ref("JlrsTests")?
                        .wrapper_unchecked()
                        .global_ref("PtrNonBitsUnion")?
                        .value_unchecked()
                };
                let arg1 = DataType::bool_type(global);
                let arg2 = Value::new(&mut *frame, -3i32)?;
                let instance = ty
                    .cast::<DataType>()?
                    .instantiate(&mut *frame, &mut [arg1.as_value(), arg2])?
                    .into_jlrs_result()?;

                let a = instance.get_raw_field::<DataTypeRef, _>("a")?;
                assert_eq!(unsafe { a.wrapper_unchecked() }, arg1);

                let b = instance.get_raw_field::<ValueRef, _>("b")?;
                let v = unsafe { b.value_unchecked().unbox::<i32>() }?;
                assert_eq!(v, -3);

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn access_raw_fields_wrong_ty() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, frame| {
                let ty = unsafe {
                    Module::main(global)
                        .submodule_ref("JlrsTests")?
                        .wrapper_unchecked()
                        .global_ref("PtrNonBitsUnion")?
                        .value_unchecked()
                };
                let arg1 = DataType::bool_type(global);
                let arg2 = Value::new(&mut *frame, -3i32)?;
                let instance = ty
                    .cast::<DataType>()?
                    .instantiate(&mut *frame, &mut [arg1.as_value(), arg2])?
                    .into_jlrs_result()?;

                assert!(instance.get_raw_field::<ArrayRef, _>("a").is_err());

                let b = instance.get_raw_field::<ValueRef, _>("b")?;
                assert!(unsafe { b.value_unchecked().unbox::<i16>() }.is_err());

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn access_array_field() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, frame| {
                let ty = unsafe {
                    Module::main(global)
                        .submodule_ref("JlrsTests")?
                        .wrapper_unchecked()
                        .global_ref("HasArray")?
                        .value_unchecked()
                };
                let data = vec![1.0, 2.0, 3.0, 4.0];
                let arg1 = Array::from_vec(&mut *frame, data, (2, 2))?;
                let instance = ty
                    .cast::<DataType>()?
                    .instantiate(&mut *frame, &mut [arg1])?
                    .into_jlrs_result()?;

                assert!(instance.get_raw_field::<ArrayRef, _>("a").is_ok());
                assert!(instance.get_raw_field::<TypedArrayRef<f64>, _>("a").is_ok());
                assert!(instance
                    .get_raw_field::<TypedArrayRef<f32>, _>("a")
                    .is_err());

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn access_ua_array_field() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, frame| {
                let ty = unsafe {
                    Module::main(global)
                        .submodule_ref("JlrsTests")?
                        .wrapper_unchecked()
                        .global_ref("UaArray")?
                        .value_unchecked()
                };
                let data = vec![1.0, 2.0, 3.0, 4.0];
                let arg1 = Array::from_vec(&mut *frame, data, (2, 2))?;
                let instance = ty
                    .cast::<DataType>()?
                    .instantiate(&mut *frame, &mut [arg1])?
                    .into_jlrs_result()?;

                assert!(instance.get_raw_field::<ArrayRef, _>("a").is_ok());
                assert!(instance.get_raw_field::<TypedArrayRef<f64>, _>("a").is_ok());
                assert!(instance
                    .get_raw_field::<TypedArrayRef<f32>, _>("a")
                    .is_err());

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn access_raw_fields_nonexistent_name() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, frame| {
                let ty = unsafe {
                    Module::main(global)
                        .submodule_ref("JlrsTests")?
                        .wrapper_unchecked()
                        .global_ref("PtrNonBitsUnion")?
                        .value_unchecked()
                };
                let arg1 = DataType::bool_type(global);
                let arg2 = Value::new(&mut *frame, -3i32)?;
                let instance = ty
                    .cast::<DataType>()?
                    .instantiate(&mut *frame, &mut [arg1.as_value(), arg2])?
                    .into_jlrs_result()?;

                assert!(instance.get_raw_field::<DataTypeRef, _>("c").is_err());
                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn access_nth_raw_fields_bits() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, frame| {
                let ty = unsafe {
                    Module::main(global)
                        .submodule_ref("JlrsTests")?
                        .wrapper_unchecked()
                        .global_ref("NoUnionsBits")?
                        .value_unchecked()
                };
                let arg1 = Value::new(&mut *frame, 3i16)?;
                let arg2 = Value::new(&mut *frame, -3i32)?;
                let instance = ty
                    .cast::<DataType>()?
                    .instantiate(&mut *frame, &mut [arg1, arg2])?
                    .into_jlrs_result()?;

                let a = instance.get_nth_raw_field::<i16>(0)?;
                assert_eq!(a, 3);

                let b = instance.get_nth_raw_field::<i32>(1)?;
                assert_eq!(b, -3);

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn access_nth_raw_fields_bits_and_ptr() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, frame| {
                let ty = unsafe {
                    Module::main(global)
                        .submodule_ref("JlrsTests")?
                        .wrapper_unchecked()
                        .global_ref("NoUnionsBitsPtr")?
                        .value_unchecked()
                };
                let arg1 = Value::new(&mut *frame, 3i16)?;
                let arg2 = DataType::bool_type(global);
                let instance = ty
                    .cast::<DataType>()?
                    .instantiate(&mut *frame, &mut [arg1, arg2.as_value()])?
                    .into_jlrs_result()?;

                let a = instance.get_nth_raw_field::<i16>(0)?;
                assert_eq!(a, 3);

                let b = instance.get_nth_raw_field::<DataTypeRef>(1)?;
                assert_eq!(unsafe { b.wrapper_unchecked() }, arg2);

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn access_nth_raw_fields_bits_and_bits_union() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, frame| {
                let ty = unsafe {
                    Module::main(global)
                        .submodule_ref("JlrsTests")?
                        .wrapper_unchecked()
                        .global_ref("BitsBitsUnion")?
                        .value_unchecked()
                };
                let arg1 = Value::new(&mut *frame, 3i16)?;
                let arg2 = Value::new(&mut *frame, -3i32)?;
                let instance = ty
                    .cast::<DataType>()?
                    .instantiate(&mut *frame, &mut [arg1, arg2])?
                    .into_jlrs_result()?;

                let a = instance.get_nth_raw_field::<i16>(0)?;
                assert_eq!(a, 3);

                let b = instance.get_nth_raw_field::<i32>(1)?;
                assert_eq!(b, -3);

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn access_nth_raw_fields_ptr_and_non_bits_union() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, frame| {
                let ty = unsafe {
                    Module::main(global)
                        .submodule_ref("JlrsTests")?
                        .wrapper_unchecked()
                        .global_ref("PtrNonBitsUnion")?
                        .value_unchecked()
                };
                let arg1 = DataType::bool_type(global);
                let arg2 = Value::new(&mut *frame, -3i32)?;
                let instance = ty
                    .cast::<DataType>()?
                    .instantiate(&mut *frame, &mut [arg1.as_value(), arg2])?
                    .into_jlrs_result()?;

                let a = instance.get_nth_raw_field::<DataTypeRef>(0)?;
                assert_eq!(unsafe { a.wrapper_unchecked() }, arg1);

                let b = instance.get_nth_raw_field::<ValueRef>(1)?;
                let v = unsafe { b.value_unchecked().unbox::<i32>() }?;
                assert_eq!(v, -3);

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn access_nth_raw_fields_wrong_ty() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, frame| {
                let ty = unsafe {
                    Module::main(global)
                        .submodule_ref("JlrsTests")?
                        .wrapper_unchecked()
                        .global_ref("PtrNonBitsUnion")?
                        .value_unchecked()
                };
                let arg1 = DataType::bool_type(global);
                let arg2 = Value::new(&mut *frame, -3i32)?;
                let instance = ty
                    .cast::<DataType>()?
                    .instantiate(&mut *frame, &mut [arg1.as_value(), arg2])?
                    .into_jlrs_result()?;

                assert!(instance.get_nth_raw_field::<ArrayRef>(0).is_err());

                let b = instance.get_nth_raw_field::<ValueRef>(1)?;
                assert!(unsafe { b.value_unchecked().unbox::<i16>() }.is_err());

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn access_nth_array_field() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, frame| {
                let ty = unsafe {
                    Module::main(global)
                        .submodule_ref("JlrsTests")?
                        .wrapper_unchecked()
                        .global_ref("HasArray")?
                        .value_unchecked()
                };
                let data = vec![1.0, 2.0, 3.0, 4.0];
                let arg1 = Array::from_vec(&mut *frame, data, (2, 2))?;
                let instance = ty
                    .cast::<DataType>()?
                    .instantiate(&mut *frame, &mut [arg1])?
                    .into_jlrs_result()?;

                assert!(instance.get_nth_raw_field::<ArrayRef>(0).is_ok());
                assert!(instance.get_nth_raw_field::<TypedArrayRef<f64>>(0).is_ok());
                assert!(instance.get_nth_raw_field::<TypedArrayRef<f32>>(0).is_err());

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn access_ua_array_field_by_idx() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, frame| {
                let ty = unsafe {
                    Module::main(global)
                        .submodule_ref("JlrsTests")?
                        .wrapper_unchecked()
                        .global_ref("UaArray")?
                        .value_unchecked()
                };
                let data = vec![1.0, 2.0, 3.0, 4.0];
                let arg1 = Array::from_vec(&mut *frame, data, (2, 2))?;
                let instance = ty
                    .cast::<DataType>()?
                    .instantiate(&mut *frame, &mut [arg1])?
                    .into_jlrs_result()?;

                assert!(instance.get_nth_raw_field::<ArrayRef>(0).is_ok());
                assert!(instance.get_nth_raw_field::<TypedArrayRef<f64>>(0).is_ok());
                assert!(instance.get_nth_raw_field::<TypedArrayRef<f32>>(0).is_err());

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn access_raw_fields_nonexistent_idx() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, frame| {
                let ty = unsafe {
                    Module::main(global)
                        .submodule_ref("JlrsTests")?
                        .wrapper_unchecked()
                        .global_ref("PtrNonBitsUnion")?
                        .value_unchecked()
                };
                let arg1 = DataType::bool_type(global);
                let arg2 = Value::new(&mut *frame, -3i32)?;
                let instance = ty
                    .cast::<DataType>()?
                    .instantiate(&mut *frame, &mut [arg1.as_value(), arg2])?
                    .into_jlrs_result()?;

                assert!(instance.get_nth_raw_field::<DataTypeRef>(2).is_err());
                Ok(())
            })
            .unwrap();
        })
    }
}
