mod util;
#[cfg(feature = "local-rt")]
mod tests {
    use jlrs::{data::managed::union_all::UnionAll, prelude::*};

    use super::util::JULIA;

    macro_rules! impl_constant_test {
        ($func:ident, $tyname:expr) => {
            fn $func() {
                JULIA.with(|j| {
                    let mut frame = StackFrame::new();
                    let mut jlrs = j.borrow_mut();

                    jlrs.instance(&mut frame)
                        .returning::<JlrsResult<_>>()
                        .scope(|frame| {
                            let v1 = Value::$func(&frame);
                            let v2 = unsafe {
                                Module::core(&frame).global(&frame, $tyname)?.as_managed()
                            };
                            assert!(v1.datatype().as_value() == v2);
                            Ok(())
                        })
                        .unwrap();
                });
            }
        };
    }

    macro_rules! impl_constant_isa_test {
        ($func:ident, $tyname:expr) => {
            fn $func() {
                JULIA.with(|j| {
                    let mut frame = StackFrame::new();
                    let mut jlrs = j.borrow_mut();

                    jlrs.instance(&mut frame)
                        .returning::<JlrsResult<_>>()
                        .scope(|frame| {
                            #[allow(unused_unsafe)]
                            unsafe {
                                let v1 = Value::$func(&frame);
                                let v2 = unsafe {
                                    Module::core(&frame).global(&frame, $tyname)?.as_managed()
                                };
                                assert!(v1.isa(v2));
                            }
                            Ok(())
                        })
                        .unwrap();
                });
            }
        };
    }

    macro_rules! impl_constant_subtype_test {
        ($func:ident, $tyname:expr) => {
            fn $func() {
                JULIA.with(|j| {
                    let mut frame = StackFrame::new();
                    let mut jlrs = j.borrow_mut();

                    jlrs.instance(&mut frame)
                        .returning::<JlrsResult<_>>()
                        .scope(|frame| {
                            #[allow(unused_unsafe)]
                            unsafe {
                                let v1 = Value::$func(&frame);
                                let v2 = unsafe {
                                    Module::core(&frame).global(&frame, $tyname)?.as_managed()
                                };
                                assert!(v1.subtype(v2));
                            }
                            Ok(())
                        })
                        .unwrap();
                });
            }
        };
    }

    macro_rules! impl_unionall_constant_test {
        ($func:ident, $tyname:expr) => {
            fn $func() {
                JULIA.with(|j| {
                    let mut frame = StackFrame::new();
                    let mut jlrs = j.borrow_mut();

                    jlrs.instance(&mut frame)
                        .returning::<JlrsResult<_>>()
                        .scope(|frame| {
                            let v1 = UnionAll::$func(&frame);
                            let v2 = unsafe {
                                Module::core(&frame).global(&frame, $tyname)?.as_managed()
                            };
                            assert!(v1.as_value() == v2);
                            Ok(())
                        })
                        .unwrap();
                });
            }
        };
    }

    macro_rules! impl_unionall_constant_isa_test {
        ($func:ident, $tyname:expr) => {
            fn $func() {
                JULIA.with(|j| {
                    let mut frame = StackFrame::new();
                    let mut jlrs = j.borrow_mut();

                    jlrs.instance(&mut frame)
                        .returning::<JlrsResult<_>>()
                        .scope(|frame| {
                            let v1 = UnionAll::$func(&frame);
                            let v2 = unsafe {
                                Module::core(&frame).global(&frame, $tyname)?.as_managed()
                            };
                            assert!(v1.as_value().isa(v2));
                            Ok(())
                        })
                        .unwrap();
                });
            }
        };
    }

    macro_rules! impl_datatype_constant_isa_test {
        ($func:ident, $tyname:expr) => {
            fn $func() {
                JULIA.with(|j| {
                    let mut frame = StackFrame::new();
                    let mut jlrs = j.borrow_mut();

                    jlrs.instance(&mut frame)
                        .returning::<JlrsResult<_>>()
                        .scope(|frame| {
                            let v1 = DataType::$func(&frame);
                            let v2 = unsafe {
                                Module::core(&frame).global(&frame, $tyname)?.as_managed()
                            };
                            assert!(v1.as_value().isa(v2));
                            Ok(())
                        })
                        .unwrap();
                });
            }
        };
    }

    impl_constant_test!(bottom_type, "TypeofBottom");
    impl_constant_test!(stackovf_exception, "StackOverflowError");
    impl_constant_test!(memory_exception, "OutOfMemoryError");
    impl_constant_test!(readonlymemory_exception, "ReadOnlyMemoryError");
    impl_constant_test!(diverror_exception, "DivideError");
    impl_constant_test!(undefref_exception, "UndefRefError");
    impl_constant_test!(interrupt_exception, "InterruptException");
    impl_constant_isa_test!(an_empty_vec_any, "Array");
    impl_constant_test!(an_empty_string, "String");
    impl_constant_subtype_test!(array_uint8_type, "Array");
    impl_constant_subtype_test!(array_any_type, "Array");
    impl_constant_subtype_test!(array_symbol_type, "Array");
    impl_constant_subtype_test!(array_int32_type, "Array");
    impl_constant_isa_test!(emptytuple, "Tuple");
    impl_constant_isa_test!(true_v, "Bool");
    impl_constant_isa_test!(false_v, "Bool");
    impl_constant_isa_test!(nothing, "Nothing");

    impl_unionall_constant_test!(type_type, "Type");
    impl_unionall_constant_isa_test!(anytuple_type_type, "Type");
    impl_unionall_constant_isa_test!(abstractarray_type, "Type");
    impl_unionall_constant_isa_test!(densearray_type, "Type");
    impl_unionall_constant_isa_test!(array_type, "Type");
    impl_unionall_constant_isa_test!(pointer_type, "Type");
    impl_unionall_constant_isa_test!(llvmpointer_type, "Type");
    impl_unionall_constant_isa_test!(ref_type, "Type");
    impl_unionall_constant_isa_test!(namedtuple_type, "Type");

    impl_datatype_constant_isa_test!(typeofbottom_type, "DataType");
    impl_datatype_constant_isa_test!(datatype_type, "DataType");
    impl_datatype_constant_isa_test!(uniontype_type, "DataType");
    impl_datatype_constant_isa_test!(unionall_type, "DataType");
    impl_datatype_constant_isa_test!(tvar_type, "DataType");
    impl_datatype_constant_isa_test!(any_type, "DataType");
    impl_datatype_constant_isa_test!(typename_type, "DataType");
    impl_datatype_constant_isa_test!(symbol_type, "DataType");
    impl_datatype_constant_isa_test!(simplevector_type, "DataType");
    impl_datatype_constant_isa_test!(anytuple_type, "DataType");
    impl_datatype_constant_isa_test!(tuple_type, "DataType");
    impl_datatype_constant_isa_test!(emptytuple_type, "DataType");
    impl_datatype_constant_isa_test!(function_type, "DataType");
    impl_datatype_constant_isa_test!(module_type, "DataType");
    impl_datatype_constant_isa_test!(abstractstring_type, "DataType");
    impl_datatype_constant_isa_test!(string_type, "DataType");
    impl_datatype_constant_isa_test!(errorexception_type, "DataType");
    impl_datatype_constant_isa_test!(argumenterror_type, "DataType");
    impl_datatype_constant_isa_test!(loaderror_type, "DataType");
    impl_datatype_constant_isa_test!(initerror_type, "DataType");
    impl_datatype_constant_isa_test!(typeerror_type, "DataType");
    impl_datatype_constant_isa_test!(methoderror_type, "DataType");
    impl_datatype_constant_isa_test!(undefvarerror_type, "DataType");
    impl_datatype_constant_isa_test!(boundserror_type, "DataType");
    impl_datatype_constant_isa_test!(bool_type, "DataType");
    impl_datatype_constant_isa_test!(char_type, "DataType");
    impl_datatype_constant_isa_test!(int8_type, "DataType");
    impl_datatype_constant_isa_test!(uint8_type, "DataType");
    impl_datatype_constant_isa_test!(int16_type, "DataType");
    impl_datatype_constant_isa_test!(uint16_type, "DataType");
    impl_datatype_constant_isa_test!(int32_type, "DataType");
    impl_datatype_constant_isa_test!(uint32_type, "DataType");
    impl_datatype_constant_isa_test!(int64_type, "DataType");
    impl_datatype_constant_isa_test!(uint64_type, "DataType");
    impl_datatype_constant_isa_test!(float16_type, "DataType");
    impl_datatype_constant_isa_test!(float32_type, "DataType");
    impl_datatype_constant_isa_test!(float64_type, "DataType");
    impl_datatype_constant_isa_test!(floatingpoint_type, "DataType");
    impl_datatype_constant_isa_test!(number_type, "DataType");
    impl_datatype_constant_isa_test!(nothing_type, "DataType");
    impl_datatype_constant_isa_test!(signed_type, "DataType");
    impl_datatype_constant_isa_test!(voidpointer_type, "DataType");
    impl_datatype_constant_isa_test!(task_type, "DataType");
    impl_datatype_constant_isa_test!(expr_type, "DataType");

    #[test]
    fn constant_tests() {
        bottom_type();
        stackovf_exception();
        memory_exception();
        readonlymemory_exception();
        diverror_exception();
        undefref_exception();
        interrupt_exception();
        an_empty_vec_any();
        an_empty_string();
        array_uint8_type();
        array_any_type();
        array_symbol_type();
        array_int32_type();
        emptytuple();
        true_v();
        false_v();
        nothing();
        type_type();
        anytuple_type_type();
        abstractarray_type();
        densearray_type();
        array_type();
        pointer_type();
        llvmpointer_type();
        ref_type();
        namedtuple_type();
        typeofbottom_type();
        datatype_type();
        uniontype_type();
        unionall_type();
        tvar_type();
        any_type();
        typename_type();
        symbol_type();
        simplevector_type();
        anytuple_type();
        tuple_type();
        emptytuple_type();
        function_type();
        module_type();
        abstractstring_type();
        string_type();
        argumenterror_type();
        loaderror_type();
        initerror_type();
        typeerror_type();
        methoderror_type();
        undefvarerror_type();
        boundserror_type();
        bool_type();
        char_type();
        int8_type();
        uint8_type();
        int16_type();
        errorexception_type();
        uint16_type();
        int32_type();
        uint32_type();
        int64_type();
        uint64_type();
        float16_type();
        float32_type();
        float64_type();
        floatingpoint_type();
        number_type();
        nothing_type();
        signed_type();
        voidpointer_type();
        task_type();
        expr_type();
    }
}
