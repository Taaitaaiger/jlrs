use jlrs::prelude::*;
use jlrs::util::JULIA;
use jlrs::wrappers::ptr::ArrayRef;
use jlrs::wrappers::ptr::DataTypeRef;
use jlrs::wrappers::ptr::TypedArrayRef;
use jlrs::wrappers::ptr::ValueRef;

#[test]
fn access_raw_fields_bits() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.scope(|global, frame| {
            Value::eval_string(&mut *frame, "struct NoUnionsBits a::Int16; b::Int32 end")?
                .into_jlrs_result()?;

            let ty = unsafe {
                Module::main(global)
                    .global_ref("NoUnionsBits")?
                    .value_unchecked()
            };
            let arg1 = Value::new(&mut *frame, 3i16)?;
            let arg2 = Value::new(&mut *frame, -3i32)?;
            let instance = ty
                .cast::<DataType>()?
                .instantiate(&mut *frame, &mut [arg1, arg2])?;

            let a = instance.unbox_field::<i16, _>("a")?;
            assert_eq!(a, 3);

            let b = instance.unbox_field::<i32, _>("b")?;
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
            Value::eval_string(
                &mut *frame,
                "struct NoUnionsBitsPtr a::Int16; b::DataType end",
            )?
            .into_jlrs_result()?;

            let ty = unsafe {
                Module::main(global)
                    .global_ref("NoUnionsBitsPtr")?
                    .value_unchecked()
            };
            let arg1 = Value::new(&mut *frame, 3i16)?;
            let arg2 = DataType::bool_type(global);
            let instance = ty
                .cast::<DataType>()?
                .instantiate(&mut *frame, &mut [arg1, arg2.as_value()])?;

            let a = instance.unbox_field::<i16, _>("a")?;
            assert_eq!(a, 3);

            let b = instance.unbox_field::<DataTypeRef, _>("b")?;
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
            Value::eval_string(
                &mut *frame,
                "struct BitsBitsUnion a::Int16; b::Union{Int16, Int32} end",
            )?
            .into_jlrs_result()?;

            let ty = unsafe {
                Module::main(global)
                    .global_ref("BitsBitsUnion")?
                    .value_unchecked()
            };
            let arg1 = Value::new(&mut *frame, 3i16)?;
            let arg2 = Value::new(&mut *frame, -3i32)?;
            let instance = ty
                .cast::<DataType>()?
                .instantiate(&mut *frame, &mut [arg1, arg2])?;

            let a = instance.unbox_field::<i16, _>("a")?;
            assert_eq!(a, 3);

            let b = instance.unbox_field::<i32, _>("b")?;
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
            Value::eval_string(
                &mut *frame,
                "struct PtrBitsUnion a::DataType; b::Union{Int16, Int32} end",
            )?
            .into_jlrs_result()?;

            let ty = unsafe {
                Module::main(global)
                    .global_ref("PtrBitsUnion")?
                    .value_unchecked()
            };
            let arg1 = DataType::bool_type(global);
            let arg2 = Value::new(&mut *frame, -3i32)?;
            let instance = ty
                .cast::<DataType>()?
                .instantiate(&mut *frame, &mut [arg1.as_value(), arg2])?;

            let a = instance.unbox_field::<DataTypeRef, _>("a")?;
            assert_eq!(unsafe { a.wrapper_unchecked() }, arg1);

            let b = instance.unbox_field::<i32, _>("b")?;
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
            Value::eval_string(
                &mut *frame,
                "struct PtrNonBitsUnion a::DataType; b::Union{Int16, Int32, DataType} end",
            )?
            .into_jlrs_result()?;

            let ty = unsafe {
                Module::main(global)
                    .global_ref("PtrNonBitsUnion")?
                    .value_unchecked()
            };
            let arg1 = DataType::bool_type(global);
            let arg2 = Value::new(&mut *frame, -3i32)?;
            let instance = ty
                .cast::<DataType>()?
                .instantiate(&mut *frame, &mut [arg1.as_value(), arg2])?;

            let a = instance.unbox_field::<DataTypeRef, _>("a")?;
            assert_eq!(unsafe { a.wrapper_unchecked() }, arg1);

            let b = instance.unbox_field::<ValueRef, _>("b")?;
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
            Value::eval_string(
                &mut *frame,
                "struct PtrNonBitsUnion a::DataType; b::Union{Int16, Int32, DataType} end",
            )?
            .into_jlrs_result()?;

            let ty = unsafe {
                Module::main(global)
                    .global_ref("PtrNonBitsUnion")?
                    .value_unchecked()
            };
            let arg1 = DataType::bool_type(global);
            let arg2 = Value::new(&mut *frame, -3i32)?;
            let instance = ty
                .cast::<DataType>()?
                .instantiate(&mut *frame, &mut [arg1.as_value(), arg2])?;

            assert!(instance.unbox_field::<ArrayRef, _>("a").is_err());

            let b = instance.unbox_field::<ValueRef, _>("b")?;
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
            Value::eval_string(&mut *frame, "struct HasArray a::Array{Float64, 2} end")?
                .into_jlrs_result()?;

            let ty = unsafe {
                Module::main(global)
                    .global_ref("HasArray")?
                    .value_unchecked()
            };
            let data = vec![1.0, 2.0, 3.0, 4.0];
            let arg1 = Array::from_vec(&mut *frame, data, (2, 2))?;
            let instance = ty
                .cast::<DataType>()?
                .instantiate(&mut *frame, &mut [arg1])?;

            assert!(instance.unbox_field::<ArrayRef, _>("a").is_ok());
            assert!(instance.unbox_field::<TypedArrayRef<f64>, _>("a").is_ok());
            assert!(instance.unbox_field::<TypedArrayRef<f32>, _>("a").is_err());

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
            Value::eval_string(&mut *frame, "struct UaArray a::Array end")?.into_jlrs_result()?;

            let ty = unsafe {
                Module::main(global)
                    .global_ref("UaArray")?
                    .value_unchecked()
            };
            let data = vec![1.0, 2.0, 3.0, 4.0];
            let arg1 = Array::from_vec(&mut *frame, data, (2, 2))?;
            let instance = ty
                .cast::<DataType>()?
                .instantiate(&mut *frame, &mut [arg1])?;

            assert!(instance.unbox_field::<ArrayRef, _>("a").is_ok());
            assert!(instance.unbox_field::<TypedArrayRef<f64>, _>("a").is_ok());
            assert!(instance.unbox_field::<TypedArrayRef<f32>, _>("a").is_err());

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
            Value::eval_string(
                &mut *frame,
                "struct PtrNonBitsUnion a::DataType; b::Union{Int16, Int32, DataType} end",
            )?
            .into_jlrs_result()?;

            let ty = unsafe {
                Module::main(global)
                    .global_ref("PtrNonBitsUnion")?
                    .value_unchecked()
            };
            let arg1 = DataType::bool_type(global);
            let arg2 = Value::new(&mut *frame, -3i32)?;
            let instance = ty
                .cast::<DataType>()?
                .instantiate(&mut *frame, &mut [arg1.as_value(), arg2])?;

            assert!(instance.unbox_field::<DataTypeRef, _>("c").is_err());
            Ok(())
        })
        .unwrap();
    })
}
