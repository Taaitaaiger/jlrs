use std::{env, process::Command};
use std::{ffi::OsStr, os::unix::prelude::OsStrExt, path::PathBuf};

fn find_julia() -> Option<String> {
    if let Ok(path) = env::var("JULIA_DIR") {
        return Some(path);
    }

    let out = Command::new("which").arg("julia").output().ok()?.stdout;

    let mut julia_path = PathBuf::from(OsStr::from_bytes(out.as_ref()));
    if !julia_path.pop() {
        return None;
    }

    if !julia_path.pop() {
        return None;
    }

    Some(julia_path.to_string_lossy().to_string())
}

fn flags() -> String {
    match find_julia() {
        Some(julia_dir) => {
            let jl_include_path = format!("{}/include/julia/", julia_dir);

            #[cfg(target_os = "linux")]
            {
                let jl_lib_path = format!("-L{}/lib/", julia_dir);
                println!("cargo:rustc-flags={}", &jl_lib_path);

                if env::var("CARGO_FEATURE_UV").is_ok() {
                    let jl_internal_lib_path = format!("-L{}/lib/julia", julia_dir);
                    println!("cargo:rustc-flags={}", &jl_internal_lib_path);
                }
            }

            #[cfg(target_os = "windows")]
            {
                let jl_lib_path = format!("-L{}/bin/", julia_dir);
                println!("cargo:rustc-flags={}", &jl_lib_path);
            }

            if env::var("CARGO_FEATURE_DEBUG").is_ok() {
                println!("cargo:rustc-link-lib=julia-debug");
            } else {
                println!("cargo:rustc-link-lib=julia");
            }

            if env::var("CARGO_FEATURE_UV").is_ok() {
                #[cfg(target_os = "windows")]
                {
                    println!("cargo:rustc-link-lib=uv-2");
                }

                #[cfg(target_os = "linux")]
                {
                    println!("cargo:rustc-link-lib=uv");
                }
            }

            jl_include_path
        }
        None => panic!("Unable to set compiler flags: JULIA_DIR is not set and no installed version of Julia can be found"),
    }
}

fn main() {
    if env::var("DOCS_RS").is_ok() {
        return;
    }

    println!("cargo:rerun-if-changed=src/jlrs_c.c");
    println!("cargo:rerun-if-changed=src/jlrs_c.h");
    println!("cargo:rerun-if-env-changed=JULIA_DIR");

    let include_dir = flags();

    let mut c = cc::Build::new();
    c.file("src/jlrs_c.c")
        .flag_if_supported("-std=gnu99")
        .include(&include_dir)
        .compile("jlrs_c");

    #[cfg(feature = "use-bindgen")]
    {
        let mut out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        out_path.push("bindings.rs");

        let include_dir_flag = format!("-I{}", include_dir);
        let bindings = bindgen::Builder::default()
            .clang_arg(include_dir_flag)
            .header("src/jlrs_c.h")
            .size_t_is_usize(true)
            .allowlist_function("jl_alloc_array_1d")
            .allowlist_function("jl_alloc_array_2d")
            .allowlist_function("jl_alloc_array_3d")
            .allowlist_function("jl_alloc_svec")
            .allowlist_function("jl_alloc_svec_uninit")
            .allowlist_function("jl_apply_array_type")
            .allowlist_function("jl_apply_generic")
            .allowlist_function("jl_apply_tuple_type_v")
            .allowlist_function("jl_apply_type")
            .allowlist_function("jl_array_eltype")
            .allowlist_function("jl_array_typetagdata")
            .allowlist_function("jl_atexit_hook")
            .allowlist_function("jl_box_bool")
            .allowlist_function("jl_box_char")
            .allowlist_function("jl_box_float32")
            .allowlist_function("jl_box_float64")
            .allowlist_function("jl_box_int16")
            .allowlist_function("jl_box_int32")
            .allowlist_function("jl_box_int64")
            .allowlist_function("jl_box_int8")
            .allowlist_function("jl_box_uint16")
            .allowlist_function("jl_box_uint32")
            .allowlist_function("jl_box_uint64")
            .allowlist_function("jl_box_uint8")
            .allowlist_function("jl_box_voidpointer")
            .allowlist_function("jl_call")
            .allowlist_function("jl_call0")
            .allowlist_function("jl_call1")
            .allowlist_function("jl_call2")
            .allowlist_function("jl_call3")
            .allowlist_function("jl_compute_fieldtypes")
            .allowlist_function("jl_cpu_threads")
            .allowlist_function("jl_egal")
            .allowlist_function("jl_eval_string")
            .allowlist_function("jl_exception_occurred")
            .allowlist_function("jl_environ")
            .allowlist_function("jl_field_index")
            .allowlist_function("jl_gc_add_finalizer")
            .allowlist_function("jl_gc_add_ptr_finalizer")
            .allowlist_function("jl_gc_collect")
            .allowlist_function("jl_gc_enable")
            .allowlist_function("jl_gc_is_enabled")
            .allowlist_function("jl_gc_queue_root")
            .allowlist_function("jl_gc_safepoint")
            .allowlist_function("jl_getallocationgranularity")
            .allowlist_function("jl_get_current_task")
            .allowlist_function("jl_get_global")
            .allowlist_function("jl_get_libllvm")
            .allowlist_function("jl_get_kwsorter")
            .allowlist_function("jl_get_nth_field")
            .allowlist_function("jl_get_nth_field_noalloc")
            .allowlist_function("jl_get_ptls_states")
            .allowlist_function("jl_get_ARCH")
            .allowlist_function("jl_get_UNAME")
            .allowlist_function("jl_getpagesize")
            .allowlist_function("jl_git_branch")
            .allowlist_function("jl_git_commit")
            .allowlist_function("jl_init")
            .allowlist_function("jl_init__threading")
            .allowlist_function("jl_init_with_image")
            .allowlist_function("jl_init_with_image__threading")
            .allowlist_function("jl_is_debugbuild")
            .allowlist_function("jl_is_imported")
            .allowlist_function("jl_is_initialized")
            .allowlist_function("jl_ver_is_release")
            .allowlist_function("jl_isa")
            .allowlist_function("jl_islayout_inline")
            .allowlist_function("jl_new_array")
            .allowlist_function("jl_new_struct_uninit")
            .allowlist_function("jl_new_structv")
            .allowlist_function("jl_new_typevar")
            .allowlist_function("jl_object_id")
            .allowlist_function("jl_pchar_to_array")
            .allowlist_function("jl_pchar_to_string")
            .allowlist_function("jl_process_events")
            .allowlist_function("jl_ptr_to_array")
            .allowlist_function("jl_ptr_to_array_1d")
            .allowlist_function("jl_set_const")
            .allowlist_function("jl_set_global")
            .allowlist_function("jl_set_nth_field")
            .allowlist_function("jl_stderr_obj")
            .allowlist_function("jl_stdout_obj")
            .allowlist_function("jl_subtype")
            .allowlist_function("jl_symbol")
            .allowlist_function("jl_symbol_n")
            .allowlist_function("jl_typename_str")
            .allowlist_function("jl_typeof_str")
            .allowlist_function("jl_type_union")
            .allowlist_function("jl_type_unionall")
            .allowlist_function("jl_unbox_float32")
            .allowlist_function("jl_unbox_float64")
            .allowlist_function("jl_unbox_int16")
            .allowlist_function("jl_unbox_int32")
            .allowlist_function("jl_unbox_int64")
            .allowlist_function("jl_unbox_int8")
            .allowlist_function("jl_unbox_uint16")
            .allowlist_function("jl_unbox_uint32")
            .allowlist_function("jl_unbox_uint64")
            .allowlist_function("jl_unbox_uint8")
            .allowlist_function("jl_unbox_voidpointer")
            .allowlist_function("jl_ver_is_released")
            .allowlist_function("jl_ver_major")
            .allowlist_function("jl_ver_minor")
            .allowlist_function("jl_ver_patch")
            .allowlist_function("jl_ver_string")
            .allowlist_function("jl_yield")
            .allowlist_function("uv_async_send")
            .allowlist_function("jlrs_alloc_array_1d")
            .allowlist_function("jlrs_alloc_array_2d")
            .allowlist_function("jlrs_alloc_array_3d")
            .allowlist_function("jlrs_apply_array_type")
            .allowlist_function("jlrs_apply_type")
            .allowlist_function("jlrs_get_nth_field")
            .allowlist_function("jlrs_new_array")
            .allowlist_function("jlrs_new_structv")
            .allowlist_function("jlrs_new_typevar")
            .allowlist_function("jlrs_set_const")
            .allowlist_function("jlrs_set_global")
            .allowlist_function("jlrs_set_nth_field")
            .allowlist_function("jlrs_type_union")
            .allowlist_function("jlrs_type_unionall")
            .allowlist_function("jlrs_reshape_array")
            .allowlist_function("jlrs_array_grow_end")
            .allowlist_function("jlrs_array_del_end")
            .allowlist_function("jlrs_array_grow_beg")
            .allowlist_function("jlrs_array_del_beg")
            .allowlist_function("jlrs_array_sizehint")
            .allowlist_function("jlrs_array_ptr_1d_push")
            .allowlist_function("jlrs_array_ptr_1d_append")
            .allowlist_function("jlrs_array_data_owner_offset")
            .allowlist_function("jlrs_print_stack")
            .allowlist_type("jl_code_instance_t")
            .allowlist_type("jl_datatype_t")
            .allowlist_type("jl_expr_t")
            .allowlist_type("jl_fielddesc16_t")
            .allowlist_type("jl_fielddesc32_t")
            .allowlist_type("jl_fielddesc8_t")
            .allowlist_type("jl_method_match_t")
            .allowlist_type("jl_methtable_t")
            .allowlist_type("jl_opaque_closure_t")
            .allowlist_type("jl_taggedvalue_t")
            .allowlist_type("jl_task_t")
            .allowlist_type("jl_typemap_entry_t")
            .allowlist_type("jl_typemap_level_t")
            .allowlist_type("jl_uniontype_t")
            .allowlist_type("jl_value_t")
            .allowlist_type("jl_vararg_t")
            .allowlist_type("jl_weakref_t")
            .allowlist_var("jl_abstractarray_type")
            .allowlist_var("jl_abstractslot_type")
            .allowlist_var("jl_abstractstring_type")
            .allowlist_var("jl_argument_type")
            .allowlist_var("jl_const_type")
            .allowlist_var("jl_partial_struct_type")
            .allowlist_var("jl_partial_opaque_type")
            .allowlist_var("jl_interconditional_type")
            .allowlist_var("jl_method_match_type")
            .allowlist_var("jl_atomicerror_type")
            .allowlist_var("jl_gotoifnot_type")
            .allowlist_var("jl_returnnode_type")
            .allowlist_var("jl_addrspace_pointer_typename")
            .allowlist_var("jl_an_empty_string")
            .allowlist_var("jl_an_empty_vec_any")
            .allowlist_var("jl_anytuple_type")
            .allowlist_var("jl_anytuple_type_type")
            .allowlist_var("jl_any_type")
            .allowlist_var("jl_argumenterror_type")
            .allowlist_var("jl_array_any_type")
            .allowlist_var("jl_array_int32_type")
            .allowlist_var("jl_array_symbol_type")
            .allowlist_var("jl_array_type")
            .allowlist_var("jl_array_typename")
            .allowlist_var("jl_array_uint8_type")
            .allowlist_var("jl_base_module")
            .allowlist_var("jl_bool_type")
            .allowlist_var("jl_bottom_type")
            .allowlist_var("jl_boundserror_type")
            .allowlist_var("jl_builtin_type")
            .allowlist_var("jl_char_type")
            .allowlist_var("jl_code_info_type")
            .allowlist_var("jl_code_instance_type")
            .allowlist_var("jl_core_module")
            .allowlist_var("jl_pgcstack")
            .allowlist_var("jl_datatype_type")
            .allowlist_var("jl_densearray_type")
            .allowlist_var("jl_diverror_exception")
            .allowlist_var("jl_egal")
            .allowlist_var("jl_emptysvec")
            .allowlist_var("jl_emptytuple")
            .allowlist_var("jl_emptytuple_type")
            .allowlist_var("jl_errorexception_type")
            .allowlist_var("jl_expr_type")
            .allowlist_var("jl_false")
            .allowlist_var("jl_float16_type")
            .allowlist_var("jl_float32_type")
            .allowlist_var("jl_float64_type")
            .allowlist_var("jl_floatingpoint_type")
            .allowlist_var("jl_function_type")
            .allowlist_var("jl_globalref_type")
            .allowlist_var("jl_gotonode_type")
            .allowlist_var("jl_initerror_type")
            .allowlist_var("jl_int16_type")
            .allowlist_var("jl_int32_type")
            .allowlist_var("jl_int64_type")
            .allowlist_var("jl_int8_type")
            .allowlist_var("jl_interrupt_exception")
            .allowlist_var("jl_intrinsic_type")
            .allowlist_var("jl_lineinfonode_type")
            .allowlist_var("jl_linenumbernode_type")
            .allowlist_var("jl_llvmpointer_type")
            .allowlist_var("jl_llvmpointer_typename")
            .allowlist_var("jl_loaderror_type")
            .allowlist_var("jl_main_module")
            .allowlist_var("jl_memory_exception")
            .allowlist_var("jl_methoderror_type")
            .allowlist_var("jl_method_instance_type")
            .allowlist_var("jl_method_match_type")
            .allowlist_var("jl_method_type")
            .allowlist_var("jl_methtable_type")
            .allowlist_var("jl_module_type")
            .allowlist_var("jl_n_threads")
            .allowlist_var("jl_namedtuple_type")
            .allowlist_var("jl_namedtuple_typename")
            .allowlist_var("jl_newvarnode_type")
            .allowlist_var("jl_nothing")
            .allowlist_var("jl_nothing_type")
            .allowlist_var("jl_number_type")
            .allowlist_var("jl_opaque_closure_type")
            .allowlist_var("jl_opaque_closure_typename")
            .allowlist_var("jl_phicnode_type")
            .allowlist_var("jl_phinode_type")
            .allowlist_var("jl_pinode_type")
            .allowlist_var("jl_pointer_type")
            .allowlist_var("jl_pointer_typename")
            .allowlist_var("jl_quotenode_type")
            .allowlist_var("jl_readonlymemory_exception")
            .allowlist_var("jl_ref_type")
            .allowlist_var("jl_signed_type")
            .allowlist_var("jl_simplevector_type")
            .allowlist_var("jl_slotnumber_type")
            .allowlist_var("jl_ssavalue_type")
            .allowlist_var("jl_stackovf_exception")
            .allowlist_var("jl_string_type")
            .allowlist_var("jl_symbol_type")
            .allowlist_var("jl_task_type")
            .allowlist_var("jl_true")
            .allowlist_var("jl_tuple_type")
            .allowlist_var("jl_tuple_typename")
            .allowlist_var("jl_tvar_type")
            .allowlist_var("jl_typedslot_type")
            .allowlist_var("jl_typeerror_type")
            .allowlist_var("jl_typemap_entry_type")
            .allowlist_var("jl_typemap_level_type")
            .allowlist_var("jl_typename_type")
            .allowlist_var("jl_typeofbottom_type")
            .allowlist_var("jl_type_type")
            .allowlist_var("jl_type_typename")
            .allowlist_var("jl_typetype_type")
            .allowlist_var("jl_uint16_type")
            .allowlist_var("jl_uint32_type")
            .allowlist_var("jl_uint64_type")
            .allowlist_var("jl_uint8_type")
            .allowlist_var("jl_undefref_exception")
            .allowlist_var("jl_undefvarerror_type")
            .allowlist_var("jl_unionall_type")
            .allowlist_var("jl_uniontype_type")
            .allowlist_var("jl_upsilonnode_type")
            .allowlist_var("jl_vararg_type")
            .allowlist_var("jl_vararg_typename")
            .allowlist_var("jl_vecelement_typename")
            .allowlist_var("jl_voidpointer_type")
            .allowlist_var("jl_weakref_type")
            .allowlist_var("jl_weakref_typejl_abstractslot_type")
            .rustfmt_bindings(true)
            .dynamic_link_require_all(true)
            .generate()
            .expect("Unable to generate bindings");

        bindings
            .write_to_file(&out_path)
            .expect("Couldn't write bindings!");
    }
}
