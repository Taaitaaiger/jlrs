mod util;
#[cfg(feature = "sync-rt")]
#[cfg(not(all(target_os = "windows", feature = "lts")))]
mod tests {
    use super::util::JULIA;
    use jlrs::prelude::*;
    use jlrs::wrappers::ptr::union_all::UnionAll;

    macro_rules! impl_constant_test {
        ($func:ident, $tyname:expr) => {
            #[test]
            fn $func() {
                JULIA.with(|j| {
                    let mut jlrs = j.borrow_mut();

                    jlrs.scope_with_capacity(0, |global, _| {
                        let v1 = Value::$func(global);
                        let v2 = unsafe {
                            Module::core(global)
                                .global_ref($tyname)?
                                .wrapper_unchecked()
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
            #[test]
            fn $func() {
                JULIA.with(|j| {
                    let mut jlrs = j.borrow_mut();

                    jlrs.scope_with_capacity(0, |global, _| {
                        #[allow(unused_unsafe)]
                        unsafe {
                            let v1 = Value::$func(global);
                            let v2 = unsafe {
                                Module::core(global)
                                    .global_ref($tyname)?
                                    .wrapper_unchecked()
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
            #[test]
            fn $func() {
                JULIA.with(|j| {
                    let mut jlrs = j.borrow_mut();

                    jlrs.scope_with_capacity(0, |global, _| {
                        #[allow(unused_unsafe)]
                        unsafe {
                            let v1 = Value::$func(global);
                            let v2 = unsafe {
                                Module::core(global)
                                    .global_ref($tyname)?
                                    .wrapper_unchecked()
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
            #[test]
            fn $func() {
                JULIA.with(|j| {
                    let mut jlrs = j.borrow_mut();

                    jlrs.scope_with_capacity(0, |global, _| {
                        let v1 = UnionAll::$func(global);
                        let v2 = unsafe {
                            Module::core(global)
                                .global_ref($tyname)?
                                .wrapper_unchecked()
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
            #[test]
            fn $func() {
                JULIA.with(|j| {
                    let mut jlrs = j.borrow_mut();

                    jlrs.scope_with_capacity(0, |global, _| {
                        let v1 = UnionAll::$func(global);
                        let v2 = unsafe {
                            Module::core(global)
                                .global_ref($tyname)?
                                .wrapper_unchecked()
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
            #[test]
            fn $func() {
                JULIA.with(|j| {
                    let mut jlrs = j.borrow_mut();

                    jlrs.scope_with_capacity(0, |global, _| {
                        let v1 = DataType::$func(global);
                        let v2 = unsafe {
                            Module::core(global)
                                .global_ref($tyname)?
                                .wrapper_unchecked()
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
    impl_datatype_constant_isa_test!(ssavalue_type, "DataType");
    impl_datatype_constant_isa_test!(abstractslot_type, "DataType");
    impl_datatype_constant_isa_test!(slotnumber_type, "DataType");
    impl_datatype_constant_isa_test!(typedslot_type, "DataType");
    impl_datatype_constant_isa_test!(simplevector_type, "DataType");
    impl_datatype_constant_isa_test!(anytuple_type, "DataType");
    impl_datatype_constant_isa_test!(tuple_type, "DataType");
    impl_datatype_constant_isa_test!(emptytuple_type, "DataType");
    impl_datatype_constant_isa_test!(function_type, "DataType");
    impl_datatype_constant_isa_test!(builtin_type, "DataType");
    impl_datatype_constant_isa_test!(method_instance_type, "DataType");
    impl_datatype_constant_isa_test!(code_instance_type, "DataType");
    impl_datatype_constant_isa_test!(code_info_type, "DataType");
    impl_datatype_constant_isa_test!(method_type, "DataType");
    impl_datatype_constant_isa_test!(module_type, "DataType");
    impl_datatype_constant_isa_test!(weakref_type, "DataType");
    impl_datatype_constant_isa_test!(abstractstring_type, "DataType");
    impl_datatype_constant_isa_test!(string_type, "DataType");
    impl_datatype_constant_isa_test!(errorexception_type, "DataType");
    impl_datatype_constant_isa_test!(argumenterror_type, "DataType");
    impl_datatype_constant_isa_test!(loaderror_type, "DataType");
    impl_datatype_constant_isa_test!(initerror_type, "DataType");
    impl_datatype_constant_isa_test!(typeerror_type, "DataType");
    impl_datatype_constant_isa_test!(methoderror_type, "DataType");
    impl_datatype_constant_isa_test!(undefvarerror_type, "DataType");
    impl_datatype_constant_isa_test!(lineinfonode_type, "DataType");
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
    impl_datatype_constant_isa_test!(globalref_type, "DataType");
    impl_datatype_constant_isa_test!(linenumbernode_type, "DataType");
    impl_datatype_constant_isa_test!(gotonode_type, "DataType");
    impl_datatype_constant_isa_test!(phinode_type, "DataType");
    impl_datatype_constant_isa_test!(pinode_type, "DataType");
    impl_datatype_constant_isa_test!(phicnode_type, "DataType");
    impl_datatype_constant_isa_test!(upsilonnode_type, "DataType");
    impl_datatype_constant_isa_test!(quotenode_type, "DataType");
    impl_datatype_constant_isa_test!(newvarnode_type, "DataType");
    impl_datatype_constant_isa_test!(intrinsic_type, "DataType");
    impl_datatype_constant_isa_test!(methtable_type, "DataType");
    impl_datatype_constant_isa_test!(typemap_level_type, "DataType");
    impl_datatype_constant_isa_test!(typemap_entry_type, "DataType");
}
