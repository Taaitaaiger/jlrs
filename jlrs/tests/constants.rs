use jlrs::prelude::*;
use jlrs::util::JULIA;

macro_rules! impl_constant_test {
    ($func:ident, $tyname:expr) => {
        #[test]
        fn $func() {
            JULIA.with(|j| {
                let mut jlrs = j.borrow_mut();

                jlrs.frame(0, |global, _| {
                    let v1 = Value::$func(global);
                    let v2 = Module::core(global).global($tyname)?;
                    assert!(v1.datatype().unwrap().as_value().egal(v2));
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

                jlrs.frame(0, |global, _| {
                    #[allow(unused_unsafe)]
                    unsafe {
                        let v1 = Value::$func(global);
                        let v2 = Module::core(global).global($tyname)?;
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

                jlrs.frame(0, |global, _| {
                    #[allow(unused_unsafe)]
                    unsafe {
                        let v1 = Value::$func(global);
                        let v2 = Module::core(global).global($tyname)?;
                        assert!(v1.subtype(v2));
                    }
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

/*

    /// The instance of `true`.
    pub fn true_v(_: Global<'base>) -> Self {
        unsafe { Value::wrap(jl_true) }
    }

    /// The instance of `false`.
    pub fn false_v(_: Global<'base>) -> Self {
        unsafe { Value::wrap(jl_false) }
    }

    /// The instance of `Core.Nothing`, `nothing`.
    pub fn nothing(_: Global<'base>) -> Self {
        unsafe { Value::wrap(jl_nothing) }
    }
*/

/*
impl<'base> UnionAll<'base> {
    pub fn type_type(_: Global<'base>) -> Self {
        unsafe { UnionAll::wrap(jl_type_type) }
    }

    pub fn typetype_type(_: Global<'base>) -> Self {
        unsafe { UnionAll::wrap(jl_typetype_type) }
    }

    pub fn anytuple_type_type(_: Global<'base>) -> Self {
        unsafe { UnionAll::wrap(jl_anytuple_type_type) }
    }

    pub fn vararg_type(_: Global<'base>) -> Self {
        unsafe { UnionAll::wrap(jl_vararg_type) }
    }

    pub fn abstractarray_type(_: Global<'base>) -> Self {
        unsafe { UnionAll::wrap(jl_abstractarray_type) }
    }

    pub fn densearray_type(_: Global<'base>) -> Self {
        unsafe { UnionAll::wrap(jl_densearray_type) }
    }

    pub fn array_type(_: Global<'base>) -> Self {
        unsafe { UnionAll::wrap(jl_array_type) }
    }

    pub fn pointer_type(_: Global<'base>) -> Self {
        unsafe { UnionAll::wrap(jl_pointer_type) }
    }

    pub fn llvmpointer_type(_: Global<'base>) -> Self {
        unsafe { UnionAll::wrap(jl_llvmpointer_type) }
    }

    pub fn ref_type(_: Global<'base>) -> Self {
        unsafe { UnionAll::wrap(jl_ref_type) }
    }

    pub fn namedtuple_type(_: Global<'base>) -> Self {
        unsafe { UnionAll::wrap(jl_namedtuple_type) }
    }
}
*/

/*
impl<'base> DataType<'base> {
    pub fn typeofbottom_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_typeofbottom_type) }
    }

    pub fn datatype_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_datatype_type) }
    }

    pub fn uniontype_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_uniontype_type) }
    }

    pub fn unionall_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_unionall_type) }
    }

    pub fn tvar_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_tvar_type) }
    }

    pub fn any_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_any_type) }
    }

    pub fn typename_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_typename_type) }
    }

    pub fn symbol_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_symbol_type) }
    }

    pub fn ssavalue_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_ssavalue_type) }
    }

    pub fn abstractslot_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_abstractslot_type) }
    }

    pub fn slotnumber_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_slotnumber_type) }
    }

    pub fn typedslot_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_typedslot_type) }
    }

    pub fn simplevector_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_simplevector_type) }
    }

    pub fn anytuple_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_anytuple_type) }
    }

    pub fn tuple_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_anytuple_type) }
    }

    pub fn emptytuple_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_emptytuple_type) }
    }

    pub fn function_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_function_type) }
    }

    pub fn builtin_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_builtin_type) }
    }

    pub fn method_instance_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_method_instance_type) }
    }

    pub fn code_instance_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_code_instance_type) }
    }

    pub fn code_info_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_code_info_type) }
    }

    pub fn method_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_method_type) }
    }

    pub fn module_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_module_type) }
    }

    pub fn weakref_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_weakref_type) }
    }

    pub fn abstractstring_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_abstractstring_type) }
    }

    pub fn string_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_string_type) }
    }

    pub fn errorexception_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_errorexception_type) }
    }

    pub fn argumenterror_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_argumenterror_type) }
    }

    pub fn loaderror_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_loaderror_type) }
    }

    pub fn initerror_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_initerror_type) }
    }

    pub fn typeerror_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_typeerror_type) }
    }

    pub fn methoderror_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_methoderror_type) }
    }

    pub fn undefvarerror_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_undefvarerror_type) }
    }

    pub fn lineinfonode_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_lineinfonode_type) }
    }

    pub fn boundserror_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_boundserror_type) }
    }

    pub fn bool_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_bool_type) }
    }

    pub fn char_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_char_type) }
    }

    pub fn int8_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_int8_type) }
    }

    pub fn uint8_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_uint8_type) }
    }

    pub fn int16_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_int16_type) }
    }

    pub fn uint16_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_uint16_type) }
    }

    pub fn int32_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_int32_type) }
    }

    pub fn uint32_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_uint32_type) }
    }

    pub fn int64_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_int64_type) }
    }

    pub fn uint64_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_uint64_type) }
    }

    pub fn float16_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_float16_type) }
    }

    pub fn float32_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_float32_type) }
    }

    pub fn float64_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_float64_type) }
    }

    pub fn floatingpoint_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_floatingpoint_type) }
    }

    pub fn number_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_number_type) }
    }

    pub fn nothing_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_nothing_type) }
    }

    pub fn signed_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_signed_type) }
    }

    pub fn voidpointer_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_voidpointer_type) }
    }

    pub fn task_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_task_type) }
    }

    pub fn expr_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_expr_type) }
    }

    pub fn globalref_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_globalref_type) }
    }

    pub fn linenumbernode_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_linenumbernode_type) }
    }

    pub fn gotonode_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_gotonode_type) }
    }

    pub fn phinode_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_phinode_type) }
    }

    pub fn pinode_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_pinode_type) }
    }

    pub fn phicnode_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_phicnode_type) }
    }

    pub fn upsilonnode_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_upsilonnode_type) }
    }

    pub fn quotenode_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_quotenode_type) }
    }

    pub fn newvarnode_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_newvarnode_type) }
    }

    pub fn intrinsic_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_intrinsic_type) }
    }

    pub fn methtable_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_methtable_type) }
    }

    pub fn typemap_level_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_typemap_level_type) }
    }

    pub fn typemap_entry_type(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_typemap_entry_type) }
    }
}
*/
