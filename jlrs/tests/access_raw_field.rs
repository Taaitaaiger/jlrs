mod util;
#[cfg(feature = "sync-rt")]
#[cfg(not(all(target_os = "windows", feature = "julia-1-6")))]
mod tests {
    use jlrs::prelude::*;

    use super::util::JULIA;

    fn access_raw_fields_bits() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let ty = unsafe {
                        Module::main(&frame)
                            .submodule(&frame, "JlrsTests")?
                            .wrapper()
                            .global(&frame, "NoUnionsBits")?
                            .value()
                    };
                    let arg1 = Value::new(&mut frame, 3i16);
                    let arg2 = Value::new(&mut frame, -3i32);
                    let instance = ty
                        .cast::<DataType>()?
                        .instantiate(&mut frame, &mut [arg1, arg2])?
                        .into_jlrs_result()?;

                    let a = instance.field_accessor().field("a")?.access::<i16>()?;
                    assert_eq!(a, 3);

                    let b = instance.field_accessor().field("b")?.access::<i32>()?;
                    assert_eq!(b, -3);

                    Ok(())
                })
                .unwrap();
        })
    }

    fn access_raw_fields_bits_and_ptr() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let ty = unsafe {
                        Module::main(&frame)
                            .submodule(&frame, "JlrsTests")?
                            .wrapper()
                            .global(&frame, "NoUnionsBitsPtr")?
                            .value()
                    };
                    let arg1 = Value::new(&mut frame, 3i16);
                    let arg2 = DataType::bool_type(&frame);
                    let instance = ty
                        .cast::<DataType>()?
                        .instantiate(&mut frame, &mut [arg1, arg2.as_value()])?
                        .into_jlrs_result()?;

                    let a = instance.field_accessor().field("a")?.access::<i16>()?;
                    assert_eq!(a, 3);

                    let b = instance
                        .field_accessor()
                        .field("b")?
                        .access::<DataTypeRef>()?;
                    assert_eq!(unsafe { b.wrapper() }, arg2);

                    Ok(())
                })
                .unwrap();
        })
    }

    fn access_raw_fields_bits_and_bits_union() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let ty = unsafe {
                        Module::main(&frame)
                            .submodule(&frame, "JlrsTests")?
                            .wrapper()
                            .global(&frame, "BitsBitsUnion")?
                            .value()
                    };
                    let arg1 = Value::new(&mut frame, 3i16);
                    let arg2 = Value::new(&mut frame, -3i32);
                    let instance = ty
                        .cast::<DataType>()?
                        .instantiate(&mut frame, &mut [arg1, arg2])?
                        .into_jlrs_result()?;

                    let a = instance.field_accessor().field("a")?.access::<i16>()?;
                    assert_eq!(a, 3);

                    let b = instance.field_accessor().field("b")?.access::<i32>()?;
                    assert_eq!(b, -3);

                    Ok(())
                })
                .unwrap();
        })
    }

    fn access_raw_fields_ptr_and_bits_union() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let ty = unsafe {
                        Module::main(&frame)
                            .submodule(&frame, "JlrsTests")?
                            .wrapper()
                            .global(&frame, "PtrBitsUnion")?
                            .value()
                    };
                    let arg1 = DataType::bool_type(&frame);
                    let arg2 = Value::new(&mut frame, -3i32);
                    let instance = ty
                        .cast::<DataType>()?
                        .instantiate(&mut frame, &mut [arg1.as_value(), arg2])?
                        .into_jlrs_result()?;

                    let a = instance
                        .field_accessor()
                        .field("a")?
                        .access::<DataTypeRef>()?;
                    assert_eq!(unsafe { a.wrapper() }, arg1);

                    let b = instance.field_accessor().field("b")?.access::<i32>()?;
                    assert_eq!(b, -3);

                    Ok(())
                })
                .unwrap();
        })
    }

    fn access_raw_fields_ptr_and_non_bits_union() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let ty = unsafe {
                        Module::main(&frame)
                            .submodule(&frame, "JlrsTests")?
                            .wrapper()
                            .global(&frame, "PtrNonBitsUnion")?
                            .value()
                    };
                    let arg1 = DataType::bool_type(&frame);
                    let arg2 = Value::new(&mut frame, -3i32);
                    let instance = ty
                        .cast::<DataType>()?
                        .instantiate(&mut frame, &mut [arg1.as_value(), arg2])?
                        .into_jlrs_result()?;

                    let a = instance
                        .field_accessor()
                        .field("a")?
                        .access::<DataTypeRef>()?;
                    assert_eq!(unsafe { a.wrapper() }, arg1);

                    let b = instance.field_accessor().field("b")?.access::<i32>()?;
                    assert_eq!(b, -3);

                    Ok(())
                })
                .unwrap();
        })
    }

    fn access_raw_fields_wrong_ty() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let ty = unsafe {
                        Module::main(&frame)
                            .submodule(&frame, "JlrsTests")?
                            .wrapper()
                            .global(&frame, "PtrNonBitsUnion")?
                            .value()
                    };
                    let arg1 = DataType::bool_type(&frame);
                    let arg2 = Value::new(&mut frame, -3i32);
                    let instance = ty
                        .cast::<DataType>()?
                        .instantiate(&mut frame, &mut [arg1.as_value(), arg2])?
                        .into_jlrs_result()?;

                    assert!(instance
                        .field_accessor()
                        .field("a")?
                        .access::<ArrayRef>()
                        .is_err());

                    let b = instance.field_accessor().field("b")?.access::<i16>();
                    assert!(b.is_err());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn access_array_field() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let ty = unsafe {
                        Module::main(&frame)
                            .submodule(&frame, "JlrsTests")?
                            .wrapper()
                            .global(&frame, "HasArray")?
                            .value()
                    };
                    let data = vec![1.0, 2.0, 3.0, 4.0];
                    let arg1 = Array::from_vec(frame.as_extended_target(), data, (2, 2))?
                        .into_jlrs_result()?;
                    let instance = ty
                        .cast::<DataType>()?
                        .instantiate(&mut frame, &mut [arg1.as_value()])?
                        .into_jlrs_result()?;

                    assert!(instance
                        .field_accessor()
                        .field("a")?
                        .access::<ArrayRef>()
                        .is_ok());
                    assert!(instance
                        .field_accessor()
                        .field("a")?
                        .access::<TypedArrayRef<f64>>()
                        .is_ok());
                    assert!(instance
                        .field_accessor()
                        .field("a")?
                        .access::<TypedArrayRef<f32>>()
                        .is_err());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn access_ua_array_field() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let ty = unsafe {
                        Module::main(&frame)
                            .submodule(&frame, "JlrsTests")?
                            .wrapper()
                            .global(&frame, "UaArray")?
                            .value()
                    };
                    let data = vec![1.0, 2.0, 3.0, 4.0];
                    let arg1 = Array::from_vec(frame.as_extended_target(), data, (2, 2))?
                        .into_jlrs_result()?;
                    let instance = ty
                        .cast::<DataType>()?
                        .instantiate(&mut frame, &mut [arg1.as_value()])?
                        .into_jlrs_result()?;

                    assert!(instance
                        .field_accessor()
                        .field("a")?
                        .access::<ArrayRef>()
                        .is_ok());
                    assert!(instance
                        .field_accessor()
                        .field("a")?
                        .access::<TypedArrayRef<f64>>()
                        .is_ok());
                    assert!(instance
                        .field_accessor()
                        .field("a")?
                        .access::<TypedArrayRef<f32>>()
                        .is_err());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn access_raw_fields_nonexistent_name() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let ty = unsafe {
                        Module::main(&frame)
                            .submodule(&frame, "JlrsTests")?
                            .wrapper()
                            .global(&frame, "PtrNonBitsUnion")?
                            .value()
                    };
                    let arg1 = DataType::bool_type(&frame);
                    let arg2 = Value::new(&mut frame, -3i32);
                    let instance = ty
                        .cast::<DataType>()?
                        .instantiate(&mut frame, &mut [arg1.as_value(), arg2])?
                        .into_jlrs_result()?;

                    assert!(instance.field_accessor().field("c").is_err());
                    Ok(())
                })
                .unwrap();
        })
    }

    fn access_nth_raw_fields_bits() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let ty = unsafe {
                        Module::main(&frame)
                            .submodule(&frame, "JlrsTests")?
                            .wrapper()
                            .global(&frame, "NoUnionsBits")?
                            .value()
                    };
                    let arg1 = Value::new(&mut frame, 3i16);
                    let arg2 = Value::new(&mut frame, -3i32);
                    let instance = ty
                        .cast::<DataType>()?
                        .instantiate(&mut frame, &mut [arg1, arg2])?
                        .into_jlrs_result()?;

                    let a = instance.field_accessor().field(0)?.access::<i16>()?;
                    assert_eq!(a, 3);

                    let b = instance.field_accessor().field(1)?.access::<i32>()?;
                    assert_eq!(b, -3);

                    Ok(())
                })
                .unwrap();
        })
    }

    fn access_nth_raw_fields_bits_and_ptr() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let ty = unsafe {
                        Module::main(&frame)
                            .submodule(&frame, "JlrsTests")?
                            .wrapper()
                            .global(&frame, "NoUnionsBitsPtr")?
                            .value()
                    };
                    let arg1 = Value::new(&mut frame, 3i16);
                    let arg2 = DataType::bool_type(&frame);
                    let instance = ty
                        .cast::<DataType>()?
                        .instantiate(&mut frame, &mut [arg1, arg2.as_value()])?
                        .into_jlrs_result()?;

                    let a = instance.field_accessor().field(0)?.access::<i16>()?;
                    assert_eq!(a, 3);

                    let b = instance
                        .field_accessor()
                        .field(1)?
                        .access::<DataTypeRef>()?;
                    assert_eq!(unsafe { b.wrapper() }, arg2);

                    Ok(())
                })
                .unwrap();
        })
    }

    fn access_nth_raw_fields_bits_and_bits_union() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let ty = unsafe {
                        Module::main(&frame)
                            .submodule(&frame, "JlrsTests")?
                            .wrapper()
                            .global(&frame, "BitsBitsUnion")?
                            .value()
                    };
                    let arg1 = Value::new(&mut frame, 3i16);
                    let arg2 = Value::new(&mut frame, -3i32);
                    let instance = ty
                        .cast::<DataType>()?
                        .instantiate(&mut frame, &mut [arg1, arg2])?
                        .into_jlrs_result()?;

                    let a = instance.field_accessor().field(0)?.access::<i16>()?;
                    assert_eq!(a, 3);

                    let b = instance.field_accessor().field(1)?.access::<i32>()?;
                    assert_eq!(b, -3);

                    Ok(())
                })
                .unwrap();
        })
    }

    fn access_nth_raw_fields_ptr_and_non_bits_union() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let ty = unsafe {
                        Module::main(&frame)
                            .submodule(&frame, "JlrsTests")?
                            .wrapper()
                            .global(&frame, "PtrNonBitsUnion")?
                            .value()
                    };
                    let arg1 = DataType::bool_type(&frame);
                    let arg2 = Value::new(&mut frame, -3i32);
                    let instance = ty
                        .cast::<DataType>()?
                        .instantiate(&mut frame, &mut [arg1.as_value(), arg2])?
                        .into_jlrs_result()?;

                    let a = instance
                        .field_accessor()
                        .field(0)?
                        .access::<DataTypeRef>()?;
                    assert_eq!(unsafe { a.wrapper() }, arg1);

                    let b = instance.field_accessor().field(1)?.access::<i32>()?;
                    assert_eq!(b, -3);

                    Ok(())
                })
                .unwrap();
        })
    }

    fn access_nth_raw_fields_wrong_ty() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let ty = unsafe {
                        Module::main(&frame)
                            .submodule(&frame, "JlrsTests")?
                            .wrapper()
                            .global(&frame, "PtrNonBitsUnion")?
                            .value()
                    };
                    let arg1 = DataType::bool_type(&frame);
                    let arg2 = Value::new(&mut frame, -3i32);
                    let instance = ty
                        .cast::<DataType>()?
                        .instantiate(&mut frame, &mut [arg1.as_value(), arg2])?
                        .into_jlrs_result()?;

                    assert!(instance
                        .field_accessor()
                        .field(0)?
                        .access::<ArrayRef>()
                        .is_err());

                    let b = instance.field_accessor().field(1)?.access::<i16>();
                    assert!(b.is_err());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn access_nth_array_field() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let ty = unsafe {
                        Module::main(&frame)
                            .submodule(&frame, "JlrsTests")?
                            .wrapper()
                            .global(&frame, "HasArray")?
                            .value()
                    };
                    let data = vec![1.0, 2.0, 3.0, 4.0];
                    let arg1 = Array::from_vec(frame.as_extended_target(), data, (2, 2))?
                        .into_jlrs_result()?;
                    let instance = ty
                        .cast::<DataType>()?
                        .instantiate(&mut frame, &mut [arg1.as_value()])?
                        .into_jlrs_result()?;

                    assert!(instance
                        .field_accessor()
                        .field(0)?
                        .access::<ArrayRef>()
                        .is_ok());
                    assert!(instance
                        .field_accessor()
                        .field(0)?
                        .access::<TypedArrayRef<f64>>()
                        .is_ok());
                    assert!(instance
                        .field_accessor()
                        .field(0)?
                        .access::<TypedArrayRef<f32>>()
                        .is_err());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn access_ua_array_field_by_idx() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let ty = unsafe {
                        Module::main(&frame)
                            .submodule(&frame, "JlrsTests")?
                            .wrapper()
                            .global(&frame, "UaArray")?
                            .value()
                    };
                    let data = vec![1.0, 2.0, 3.0, 4.0];
                    let arg1 = Array::from_vec(frame.as_extended_target(), data, (2, 2))?
                        .into_jlrs_result()?;
                    let instance = ty
                        .cast::<DataType>()?
                        .instantiate(&mut frame, &mut [arg1.as_value()])?
                        .into_jlrs_result()?;

                    assert!(instance
                        .field_accessor()
                        .field(0)?
                        .access::<ArrayRef>()
                        .is_ok());
                    assert!(instance
                        .field_accessor()
                        .field(0)?
                        .access::<TypedArrayRef<f64>>()
                        .is_ok());
                    assert!(instance
                        .field_accessor()
                        .field(0)?
                        .access::<TypedArrayRef<f32>>()
                        .is_err());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn access_raw_fields_nonexistent_idx() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let ty = unsafe {
                        Module::main(&frame)
                            .submodule(&frame, "JlrsTests")?
                            .wrapper()
                            .global(&frame, "PtrNonBitsUnion")?
                            .value()
                    };
                    let arg1 = DataType::bool_type(&frame);
                    let arg2 = Value::new(&mut frame, -3i32);
                    let instance = ty
                        .cast::<DataType>()?
                        .instantiate(&mut frame, &mut [arg1.as_value(), arg2])?
                        .into_jlrs_result()?;

                    assert!(instance.field_accessor().field(2).is_err());
                    Ok(())
                })
                .unwrap();
        })
    }

    #[test]
    fn access_raw_field_tests() {
        access_raw_fields_bits();
        access_raw_fields_bits_and_ptr();
        access_raw_fields_bits_and_bits_union();
        access_raw_fields_ptr_and_bits_union();
        access_raw_fields_ptr_and_non_bits_union();
        access_raw_fields_wrong_ty();
        access_array_field();
        access_ua_array_field();
        access_raw_fields_nonexistent_name();
        access_nth_raw_fields_bits();
        access_nth_raw_fields_bits_and_ptr();
        access_nth_raw_fields_bits_and_bits_union();
        access_nth_raw_fields_ptr_and_non_bits_union();
        access_nth_raw_fields_wrong_ty();
        access_nth_array_field();
        access_ua_array_field_by_idx();
        access_raw_fields_nonexistent_idx();
    }
}
