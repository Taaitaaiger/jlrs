mod util;
#[cfg(feature = "sync-rt")]
#[cfg(not(all(target_os = "windows", feature = "lts")))]
mod tests {
    use super::util::JULIA;
    use jlrs::prelude::*;

    #[test]
    fn access_raw_fields_bits() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, mut frame| {
                let ty = unsafe {
                    Module::main(global)
                        .submodule_ref("JlrsTests")?
                        .wrapper_unchecked()
                        .global_ref("NoUnionsBits")?
                        .value_unchecked()
                };
                let arg1 = Value::new(&mut frame, 3i16);
                let arg2 = Value::new(&mut frame, -3i32);
                let instance = ty
                    .cast::<DataType>()?
                    .instantiate(&mut frame, &mut [arg1, arg2])?
                    .into_jlrs_result()?;

                let a = instance
                    .field_accessor(&mut frame)
                    .field("a")?
                    .access::<i16>()?;
                assert_eq!(a, 3);

                let b = instance
                    .field_accessor(&mut frame)
                    .field("b")?
                    .access::<i32>()?;
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
            jlrs.scope(|global, mut frame| {
                let ty = unsafe {
                    Module::main(global)
                        .submodule_ref("JlrsTests")?
                        .wrapper_unchecked()
                        .global_ref("NoUnionsBitsPtr")?
                        .value_unchecked()
                };
                let arg1 = Value::new(&mut frame, 3i16);
                let arg2 = DataType::bool_type(global);
                let instance = ty
                    .cast::<DataType>()?
                    .instantiate(&mut frame, &mut [arg1, arg2.as_value()])?
                    .into_jlrs_result()?;

                let a = instance
                    .field_accessor(&mut frame)
                    .field("a")?
                    .access::<i16>()?;
                assert_eq!(a, 3);

                let b = instance
                    .field_accessor(&mut frame)
                    .field("b")?
                    .access::<DataTypeRef>()?;
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
            jlrs.scope(|global, mut frame| {
                let ty = unsafe {
                    Module::main(global)
                        .submodule_ref("JlrsTests")?
                        .wrapper_unchecked()
                        .global_ref("BitsBitsUnion")?
                        .value_unchecked()
                };
                let arg1 = Value::new(&mut frame, 3i16);
                let arg2 = Value::new(&mut frame, -3i32);
                let instance = ty
                    .cast::<DataType>()?
                    .instantiate(&mut frame, &mut [arg1, arg2])?
                    .into_jlrs_result()?;

                let a = instance
                    .field_accessor(&mut frame)
                    .field("a")?
                    .access::<i16>()?;
                assert_eq!(a, 3);

                let b = instance
                    .field_accessor(&mut frame)
                    .field("b")?
                    .access::<i32>()?;
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
            jlrs.scope(|global, mut frame| {
                let ty = unsafe {
                    Module::main(global)
                        .submodule_ref("JlrsTests")?
                        .wrapper_unchecked()
                        .global_ref("PtrBitsUnion")?
                        .value_unchecked()
                };
                let arg1 = DataType::bool_type(global);
                let arg2 = Value::new(&mut frame, -3i32);
                let instance = ty
                    .cast::<DataType>()?
                    .instantiate(&mut frame, &mut [arg1.as_value(), arg2])?
                    .into_jlrs_result()?;

                let a = instance
                    .field_accessor(&mut frame)
                    .field("a")?
                    .access::<DataTypeRef>()?;
                assert_eq!(unsafe { a.wrapper_unchecked() }, arg1);

                let b = instance
                    .field_accessor(&mut frame)
                    .field("b")?
                    .access::<i32>()?;
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
            jlrs.scope(|global, mut frame| {
                let ty = unsafe {
                    Module::main(global)
                        .submodule_ref("JlrsTests")?
                        .wrapper_unchecked()
                        .global_ref("PtrNonBitsUnion")?
                        .value_unchecked()
                };
                let arg1 = DataType::bool_type(global);
                let arg2 = Value::new(&mut frame, -3i32);
                let instance = ty
                    .cast::<DataType>()?
                    .instantiate(&mut frame, &mut [arg1.as_value(), arg2])?
                    .into_jlrs_result()?;

                let a = instance
                    .field_accessor(&mut frame)
                    .field("a")?
                    .access::<DataTypeRef>()?;
                assert_eq!(unsafe { a.wrapper_unchecked() }, arg1);

                let b = instance
                    .field_accessor(&mut frame)
                    .field("b")?
                    .access::<i32>()?;
                assert_eq!(b, -3);

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn access_raw_fields_wrong_ty() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, mut frame| {
                let ty = unsafe {
                    Module::main(global)
                        .submodule_ref("JlrsTests")?
                        .wrapper_unchecked()
                        .global_ref("PtrNonBitsUnion")?
                        .value_unchecked()
                };
                let arg1 = DataType::bool_type(global);
                let arg2 = Value::new(&mut frame, -3i32);
                let instance = ty
                    .cast::<DataType>()?
                    .instantiate(&mut frame, &mut [arg1.as_value(), arg2])?
                    .into_jlrs_result()?;

                assert!(instance
                    .field_accessor(&mut frame)
                    .field("a")?
                    .access::<ArrayRef>()
                    .is_err());

                let b = instance
                    .field_accessor(&mut frame)
                    .field("b")?
                    .access::<i16>();
                assert!(b.is_err());

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn access_array_field() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, mut frame| {
                let ty = unsafe {
                    Module::main(global)
                        .submodule_ref("JlrsTests")?
                        .wrapper_unchecked()
                        .global_ref("HasArray")?
                        .value_unchecked()
                };
                let data = vec![1.0, 2.0, 3.0, 4.0];
                let arg1 = Array::from_vec(&mut frame, data, (2, 2))?.into_jlrs_result()?;
                let instance = ty
                    .cast::<DataType>()?
                    .instantiate(&mut frame, &mut [arg1.as_value()])?
                    .into_jlrs_result()?;

                assert!(instance
                    .field_accessor(&mut frame)
                    .field("a")?
                    .access::<ArrayRef>()
                    .is_ok());
                assert!(instance
                    .field_accessor(&mut frame)
                    .field("a")?
                    .access::<TypedArrayRef<f64>>()
                    .is_ok());
                assert!(instance
                    .field_accessor(&mut frame)
                    .field("a")?
                    .access::<TypedArrayRef<f32>>()
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
            jlrs.scope(|global, mut frame| {
                let ty = unsafe {
                    Module::main(global)
                        .submodule_ref("JlrsTests")?
                        .wrapper_unchecked()
                        .global_ref("UaArray")?
                        .value_unchecked()
                };
                let data = vec![1.0, 2.0, 3.0, 4.0];
                let arg1 = Array::from_vec(&mut frame, data, (2, 2))?.into_jlrs_result()?;
                let instance = ty
                    .cast::<DataType>()?
                    .instantiate(&mut frame, &mut [arg1.as_value()])?
                    .into_jlrs_result()?;

                assert!(instance
                    .field_accessor(&mut frame)
                    .field("a")?
                    .access::<ArrayRef>()
                    .is_ok());
                assert!(instance
                    .field_accessor(&mut frame)
                    .field("a")?
                    .access::<TypedArrayRef<f64>>()
                    .is_ok());
                assert!(instance
                    .field_accessor(&mut frame)
                    .field("a")?
                    .access::<TypedArrayRef<f32>>()
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
            jlrs.scope(|global, mut frame| {
                let ty = unsafe {
                    Module::main(global)
                        .submodule_ref("JlrsTests")?
                        .wrapper_unchecked()
                        .global_ref("PtrNonBitsUnion")?
                        .value_unchecked()
                };
                let arg1 = DataType::bool_type(global);
                let arg2 = Value::new(&mut frame, -3i32);
                let instance = ty
                    .cast::<DataType>()?
                    .instantiate(&mut frame, &mut [arg1.as_value(), arg2])?
                    .into_jlrs_result()?;

                assert!(instance.field_accessor(&mut frame).field("c").is_err());
                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn access_nth_raw_fields_bits() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, mut frame| {
                let ty = unsafe {
                    Module::main(global)
                        .submodule_ref("JlrsTests")?
                        .wrapper_unchecked()
                        .global_ref("NoUnionsBits")?
                        .value_unchecked()
                };
                let arg1 = Value::new(&mut frame, 3i16);
                let arg2 = Value::new(&mut frame, -3i32);
                let instance = ty
                    .cast::<DataType>()?
                    .instantiate(&mut frame, &mut [arg1, arg2])?
                    .into_jlrs_result()?;

                let a = instance
                    .field_accessor(&mut frame)
                    .field(0)?
                    .access::<i16>()?;
                assert_eq!(a, 3);

                let b = instance
                    .field_accessor(&mut frame)
                    .field(1)?
                    .access::<i32>()?;
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
            jlrs.scope(|global, mut frame| {
                let ty = unsafe {
                    Module::main(global)
                        .submodule_ref("JlrsTests")?
                        .wrapper_unchecked()
                        .global_ref("NoUnionsBitsPtr")?
                        .value_unchecked()
                };
                let arg1 = Value::new(&mut frame, 3i16);
                let arg2 = DataType::bool_type(global);
                let instance = ty
                    .cast::<DataType>()?
                    .instantiate(&mut frame, &mut [arg1, arg2.as_value()])?
                    .into_jlrs_result()?;

                let a = instance
                    .field_accessor(&mut frame)
                    .field(0)?
                    .access::<i16>()?;
                assert_eq!(a, 3);

                let b = instance
                    .field_accessor(&mut frame)
                    .field(1)?
                    .access::<DataTypeRef>()?;
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
            jlrs.scope(|global, mut frame| {
                let ty = unsafe {
                    Module::main(global)
                        .submodule_ref("JlrsTests")?
                        .wrapper_unchecked()
                        .global_ref("BitsBitsUnion")?
                        .value_unchecked()
                };
                let arg1 = Value::new(&mut frame, 3i16);
                let arg2 = Value::new(&mut frame, -3i32);
                let instance = ty
                    .cast::<DataType>()?
                    .instantiate(&mut frame, &mut [arg1, arg2])?
                    .into_jlrs_result()?;

                let a = instance
                    .field_accessor(&mut frame)
                    .field(0)?
                    .access::<i16>()?;
                assert_eq!(a, 3);

                let b = instance
                    .field_accessor(&mut frame)
                    .field(1)?
                    .access::<i32>()?;
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
            jlrs.scope(|global, mut frame| {
                let ty = unsafe {
                    Module::main(global)
                        .submodule_ref("JlrsTests")?
                        .wrapper_unchecked()
                        .global_ref("PtrNonBitsUnion")?
                        .value_unchecked()
                };
                let arg1 = DataType::bool_type(global);
                let arg2 = Value::new(&mut frame, -3i32);
                let instance = ty
                    .cast::<DataType>()?
                    .instantiate(&mut frame, &mut [arg1.as_value(), arg2])?
                    .into_jlrs_result()?;

                let a = instance
                    .field_accessor(&mut frame)
                    .field(0)?
                    .access::<DataTypeRef>()?;
                assert_eq!(unsafe { a.wrapper_unchecked() }, arg1);

                let b = instance
                    .field_accessor(&mut frame)
                    .field(1)?
                    .access::<i32>()?;
                assert_eq!(b, -3);

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn access_nth_raw_fields_wrong_ty() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, mut frame| {
                let ty = unsafe {
                    Module::main(global)
                        .submodule_ref("JlrsTests")?
                        .wrapper_unchecked()
                        .global_ref("PtrNonBitsUnion")?
                        .value_unchecked()
                };
                let arg1 = DataType::bool_type(global);
                let arg2 = Value::new(&mut frame, -3i32);
                let instance = ty
                    .cast::<DataType>()?
                    .instantiate(&mut frame, &mut [arg1.as_value(), arg2])?
                    .into_jlrs_result()?;

                assert!(instance
                    .field_accessor(&mut frame)
                    .field(0)?
                    .access::<ArrayRef>()
                    .is_err());

                let b = instance
                    .field_accessor(&mut frame)
                    .field(1)?
                    .access::<i16>();
                assert!(b.is_err());

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn access_nth_array_field() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, mut frame| {
                let ty = unsafe {
                    Module::main(global)
                        .submodule_ref("JlrsTests")?
                        .wrapper_unchecked()
                        .global_ref("HasArray")?
                        .value_unchecked()
                };
                let data = vec![1.0, 2.0, 3.0, 4.0];
                let arg1 = Array::from_vec(&mut frame, data, (2, 2))?.into_jlrs_result()?;
                let instance = ty
                    .cast::<DataType>()?
                    .instantiate(&mut frame, &mut [arg1.as_value()])?
                    .into_jlrs_result()?;

                assert!(instance
                    .field_accessor(&mut frame)
                    .field(0)?
                    .access::<ArrayRef>()
                    .is_ok());
                assert!(instance
                    .field_accessor(&mut frame)
                    .field(0)?
                    .access::<TypedArrayRef<f64>>()
                    .is_ok());
                assert!(instance
                    .field_accessor(&mut frame)
                    .field(0)?
                    .access::<TypedArrayRef<f32>>()
                    .is_err());

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn access_ua_array_field_by_idx() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, mut frame| {
                let ty = unsafe {
                    Module::main(global)
                        .submodule_ref("JlrsTests")?
                        .wrapper_unchecked()
                        .global_ref("UaArray")?
                        .value_unchecked()
                };
                let data = vec![1.0, 2.0, 3.0, 4.0];
                let arg1 = Array::from_vec(&mut frame, data, (2, 2))?.into_jlrs_result()?;
                let instance = ty
                    .cast::<DataType>()?
                    .instantiate(&mut frame, &mut [arg1.as_value()])?
                    .into_jlrs_result()?;

                assert!(instance
                    .field_accessor(&mut frame)
                    .field(0)?
                    .access::<ArrayRef>()
                    .is_ok());
                assert!(instance
                    .field_accessor(&mut frame)
                    .field(0)?
                    .access::<TypedArrayRef<f64>>()
                    .is_ok());
                assert!(instance
                    .field_accessor(&mut frame)
                    .field(0)?
                    .access::<TypedArrayRef<f32>>()
                    .is_err());

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn access_raw_fields_nonexistent_idx() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, mut frame| {
                let ty = unsafe {
                    Module::main(global)
                        .submodule_ref("JlrsTests")?
                        .wrapper_unchecked()
                        .global_ref("PtrNonBitsUnion")?
                        .value_unchecked()
                };
                let arg1 = DataType::bool_type(global);
                let arg2 = Value::new(&mut frame, -3i32);
                let instance = ty
                    .cast::<DataType>()?
                    .instantiate(&mut frame, &mut [arg1.as_value(), arg2])?
                    .into_jlrs_result()?;

                assert!(instance.field_accessor(&mut frame).field(2).is_err());
                Ok(())
            })
            .unwrap();
        })
    }
}
