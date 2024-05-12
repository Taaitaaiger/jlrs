mod util;

#[cfg(feature = "local-rt")]
mod tests {
    use std::{ffi::c_void, ptr::null_mut};

    use jlrs::{
        data::{managed::union_all::UnionAll, types::typecheck::*},
        prelude::*,
    };

    use super::util::JULIA;

    macro_rules! impl_typecheck_test {
        ($name:ident, $invalid_name:ident, $t:ty, $val:expr) => {
            fn $name() {
                JULIA.with(|j| {
                    let mut frame = StackFrame::new();
                    let mut jlrs = j.borrow_mut();
                    jlrs.instance(&mut frame)
                        .returning::<JlrsResult<_>>()
                        .scope(|mut frame| {
                            let val: $t = $val;
                            let v = Value::new(&mut frame, val);
                            assert!(<$t as Typecheck>::typecheck(v.datatype()));
                            Ok(())
                        })
                        .unwrap();
                })
            }

            fn $invalid_name() {
                JULIA.with(|j| {
                    let mut frame = StackFrame::new();
                    let mut jlrs = j.borrow_mut();
                    jlrs.instance(&mut frame)
                        .returning::<JlrsResult<_>>()
                        .scope(|mut frame| {
                            let val: $t = $val;
                            let v = Value::new(&mut frame, val);
                            assert!(!<*mut $t as Typecheck>::typecheck(v.datatype()));
                            Ok(())
                        })
                        .unwrap();
                })
            }
        };
    }

    impl_typecheck_test!(i8_typecheck, i8_failing_typecheck, i8, 0i8);
    impl_typecheck_test!(i16_typecheck, i16_failing_typecheck, i16, 0i16);
    impl_typecheck_test!(i32_typecheck, i32_failing_typecheck, i32, 0i32);
    impl_typecheck_test!(i64_typecheck, i64_failing_typecheck, i64, 0i64);
    impl_typecheck_test!(isize_typecheck, isize_failing_typecheck, isize, 0isize);
    impl_typecheck_test!(u8_typecheck, u8_failing_typecheck, u8, 0u8);
    impl_typecheck_test!(u16_typecheck, u16_failing_typecheck, u16, 0u16);
    impl_typecheck_test!(u32_typecheck, u32_failing_typecheck, u32, 0u32);
    impl_typecheck_test!(u64_typecheck, u64_failing_typecheck, u64, 0u64);
    impl_typecheck_test!(usize_typecheck, usize_failing_typecheck, usize, 0usize);
    impl_typecheck_test!(f32_typecheck, f32_failing_typecheck, f32, 0f32);
    impl_typecheck_test!(f64_typecheck, f64_failing_typecheck, f64, 0f64);
    impl_typecheck_test!(bool_typecheck, bool_failing_typecheck, bool, false);
    impl_typecheck_test!(char_typecheck, char_failing_typecheck, char, 'a');

    impl_typecheck_test!(
        i8_ptr_typecheck,
        i8_ptr_failing_typecheck,
        *mut i8,
        null_mut()
    );
    impl_typecheck_test!(
        i16_ptr_typecheck,
        i16_ptr_failing_typecheck,
        *mut i16,
        null_mut()
    );
    impl_typecheck_test!(
        i32_ptr_typecheck,
        i32_ptr_failing_typecheck,
        *mut i32,
        null_mut()
    );
    impl_typecheck_test!(
        i64_ptr_typecheck,
        i64_ptr_failing_typecheck,
        *mut i64,
        null_mut()
    );
    impl_typecheck_test!(
        isize_ptr_typecheck,
        isize_ptr_failing_typecheck,
        *mut isize,
        null_mut()
    );
    impl_typecheck_test!(
        u8_ptr_typecheck,
        u8_ptr_failing_typecheck,
        *mut u8,
        null_mut()
    );
    impl_typecheck_test!(
        u16_ptr_typecheck,
        u16_ptr_failing_typecheck,
        *mut u16,
        null_mut()
    );
    impl_typecheck_test!(
        u32_ptr_typecheck,
        u32_ptr_failing_typecheck,
        *mut u32,
        null_mut()
    );
    impl_typecheck_test!(
        u64_ptr_typecheck,
        u64_ptr_failing_typecheck,
        *mut u64,
        null_mut()
    );
    impl_typecheck_test!(
        usize_ptr_typecheck,
        usize_ptr_failing_typecheck,
        *mut usize,
        null_mut()
    );
    impl_typecheck_test!(
        f32_ptr_typecheck,
        f32_ptr_failing_typecheck,
        *mut f32,
        null_mut()
    );
    impl_typecheck_test!(
        f64_ptr_typecheck,
        f64_ptr_failing_typecheck,
        *mut f64,
        null_mut()
    );
    impl_typecheck_test!(
        bool_ptr_typecheck,
        bool_ptr_failing_typecheck,
        *mut bool,
        null_mut()
    );
    impl_typecheck_test!(
        char_ptr_typecheck,
        char_ptr_failing_typecheck,
        *mut char,
        null_mut()
    );

    impl_typecheck_test!(
        void_ptr_typecheck,
        failing_void_ptr_typecheck,
        *mut c_void,
        null_mut()
    );

    fn type_typecheck() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|frame| {
                    assert!(Type::typecheck(DataType::datatype_type(&frame)));
                    assert!(Type::typecheck(DataType::unionall_type(&frame)));
                    assert!(Type::typecheck(DataType::uniontype_type(&frame)));
                    assert!(Type::typecheck(DataType::typeofbottom_type(&frame)));
                    assert!(!Type::typecheck(DataType::bool_type(&frame)));
                    Ok(())
                })
                .unwrap();
        })
    }

    fn bits_typecheck() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|frame| {
                    assert!(Bits::typecheck(DataType::bool_type(&frame)));
                    assert!(!Bits::typecheck(DataType::datatype_type(&frame)));
                    Ok(())
                })
                .unwrap();
        })
    }

    fn abstract_typecheck() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|frame| {
                    assert!(Abstract::typecheck(DataType::floatingpoint_type(&frame)));
                    assert!(!Abstract::typecheck(DataType::datatype_type(&frame)));
                    Ok(())
                })
                .unwrap();
        })
    }

    fn abstract_ref_typecheck() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let args = [DataType::uint8_type(&frame).as_value()];
                    let v = UnionAll::ref_type(&frame)
                        .as_value()
                        .apply_type_unchecked(&mut frame, args)
                        .cast::<DataType>()?;

                    assert!(AbstractRef::typecheck(v));
                    assert!(!AbstractRef::typecheck(DataType::bool_type(&frame)));
                    Ok(())
                })
                .unwrap();
        })
    }

    #[cfg(not(feature = "julia-1-6"))]
    fn vec_element_typecheck() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let value = Value::new(&mut frame, 0u8);
                    let args = [DataType::uint8_type(&frame).as_value()];
                    let vec_elem_ty = Module::base(&frame)
                        .global(&mut frame, "VecElement")?
                        .as_value()
                        .apply_type_unchecked(&mut frame, args)
                        .cast::<DataType>()?
                        .instantiate(&mut frame, &mut [value])?
                        .into_jlrs_result()?
                        .datatype();

                    assert!(VecElement::typecheck(vec_elem_ty));
                    assert!(!VecElement::typecheck(DataType::bool_type(&frame)));
                    Ok(())
                })
                .unwrap();
        })
    }

    fn type_type_typecheck() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let args = [DataType::uint8_type(&frame).as_value()];
                    let ty = UnionAll::type_type(&frame)
                        .as_value()
                        .apply_type_unchecked(&mut frame, args)
                        .cast::<DataType>()?;

                    assert!(TypeType::typecheck(ty));
                    assert!(!TypeType::typecheck(DataType::bool_type(&frame)));
                    Ok(())
                })
                .unwrap();
        })
    }

    fn named_tuple_typecheck() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let a = Value::new(&mut frame, 1usize);
                    let named_tuple = named_tuple!(&mut frame, "a" => a);

                    assert!(NamedTuple::typecheck(named_tuple.datatype()));
                    assert!(!NamedTuple::typecheck(DataType::bool_type(&frame)));
                    Ok(())
                })
                .unwrap();
        })
    }

    fn mutable_typecheck() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|frame| {
                    assert!(Mutable::typecheck(DataType::datatype_type(&frame)));
                    assert!(!Mutable::typecheck(DataType::bool_type(&frame)));
                    Ok(())
                })
                .unwrap();
        })
    }

    fn nothing_typecheck() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|frame| {
                    let nothing = Value::nothing(&frame);
                    assert!(Nothing::typecheck(nothing.datatype()));
                    assert!(!Nothing::typecheck(DataType::bool_type(&frame)));
                    Ok(())
                })
                .unwrap();
        })
    }

    fn immutable_typecheck() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|frame| {
                    assert!(Immutable::typecheck(DataType::bool_type(&frame)));
                    assert!(!Immutable::typecheck(DataType::datatype_type(&frame)));
                    Ok(())
                })
                .unwrap();
        })
    }

    fn primitive_type_typecheck() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|frame| {
                    assert!(PrimitiveType::typecheck(DataType::bool_type(&frame)));
                    assert!(!PrimitiveType::typecheck(DataType::datatype_type(&frame)));
                    assert!(!PrimitiveType::typecheck(DataType::floatingpoint_type(
                        &frame
                    )));
                    Ok(())
                })
                .unwrap();
        })
    }

    fn struct_type_typecheck() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|frame| {
                    assert!(StructType::typecheck(DataType::datatype_type(&frame)));
                    assert!(!StructType::typecheck(DataType::bool_type(&frame)));
                    assert!(!StructType::typecheck(DataType::floatingpoint_type(&frame)));
                    Ok(())
                })
                .unwrap();
        })
    }

    fn singleton_typecheck() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|frame| {
                    assert!(Singleton::typecheck(DataType::nothing_type(&frame)));
                    assert!(!Singleton::typecheck(DataType::bool_type(&frame)));
                    Ok(())
                })
                .unwrap();
        })
    }

    fn string_typecheck() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|frame| {
                    assert!(String::typecheck(DataType::string_type(&frame)));
                    assert!(!String::typecheck(DataType::bool_type(&frame)));
                    Ok(())
                })
                .unwrap();
        })
    }

    fn pointer_typecheck() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let v: *mut u8 = null_mut();
                    let value = Value::new(&mut frame, v);
                    assert!(Pointer::typecheck(value.datatype()));
                    assert!(!Pointer::typecheck(DataType::bool_type(&frame)));
                    Ok(())
                })
                .unwrap();
        })
    }

    fn llvm_pointer_typecheck() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let cmd = "reinterpret(Core.LLVMPtr{UInt8,1}, 0)";
                    let value = Value::eval_string(&mut frame, cmd).into_jlrs_result()?;
                    assert!(LLVMPointer::typecheck(value.datatype()));
                    assert!(!LLVMPointer::typecheck(DataType::bool_type(&frame)));
                    Ok(())
                })
                .unwrap();
        })
    }

    fn concrete_typecheck() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|frame| {
                    assert!(Concrete::typecheck(DataType::bool_type(&frame)));
                    assert!(!Concrete::typecheck(DataType::floatingpoint_type(&frame)));
                    Ok(())
                })
                .unwrap();
        })
    }

    #[test]
    fn typecheck_tests() {
        i8_typecheck();
        i8_failing_typecheck();
        i16_typecheck();
        i16_failing_typecheck();
        i32_typecheck();
        i32_failing_typecheck();
        i64_typecheck();
        i64_failing_typecheck();
        isize_typecheck();
        isize_failing_typecheck();
        u8_typecheck();
        u8_failing_typecheck();
        u16_typecheck();
        u16_failing_typecheck();
        u32_typecheck();
        u32_failing_typecheck();
        u64_typecheck();
        u64_failing_typecheck();
        usize_typecheck();
        usize_failing_typecheck();
        f32_typecheck();
        f32_failing_typecheck();
        f64_typecheck();
        f64_failing_typecheck();
        bool_typecheck();
        bool_failing_typecheck();
        char_typecheck();
        char_failing_typecheck();

        i8_ptr_typecheck();
        i8_ptr_failing_typecheck();
        i16_ptr_typecheck();
        i16_ptr_failing_typecheck();
        i32_ptr_typecheck();
        i32_ptr_failing_typecheck();
        i64_ptr_typecheck();
        i64_ptr_failing_typecheck();
        isize_ptr_typecheck();
        isize_ptr_failing_typecheck();
        u8_ptr_typecheck();
        u8_ptr_failing_typecheck();
        u16_ptr_typecheck();
        u16_ptr_failing_typecheck();
        u32_ptr_typecheck();
        u32_ptr_failing_typecheck();
        u64_ptr_typecheck();
        u64_ptr_failing_typecheck();
        usize_ptr_typecheck();
        usize_ptr_failing_typecheck();
        f32_ptr_typecheck();
        f32_ptr_failing_typecheck();
        f64_ptr_typecheck();
        f64_ptr_failing_typecheck();
        bool_ptr_typecheck();
        bool_ptr_failing_typecheck();
        char_ptr_typecheck();
        char_ptr_failing_typecheck();
        void_ptr_typecheck();
        failing_void_ptr_typecheck();
        type_typecheck();
        bits_typecheck();
        abstract_typecheck();
        abstract_ref_typecheck();
        type_type_typecheck();
        named_tuple_typecheck();
        mutable_typecheck();
        nothing_typecheck();
        immutable_typecheck();
        primitive_type_typecheck();
        struct_type_typecheck();
        singleton_typecheck();
        string_typecheck();
        pointer_typecheck();
        llvm_pointer_typecheck();
        concrete_typecheck();
        #[cfg(not(feature = "julia-1-6"))]
        vec_element_typecheck();
    }
}
