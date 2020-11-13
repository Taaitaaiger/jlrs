#![allow(unused_imports)]

use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(target_os = "linux")]
fn find_julia() -> Option<String> {
    if let Ok(path) = env::var("JULIA_DIR") {
        return Some(path);
    }

    if Path::new("/usr/include/julia/julia.h").exists() {
        return Some("/usr".to_string());
    }

    None
}

#[cfg(target_os = "windows")]
fn flags() -> Vec<String> {
    let julia_dir = env::var("JULIA_DIR").expect("Julia cannot be found. You can specify the Julia installation path with the JULIA_DIR environment variable.");
    let cygwin_path = env::var("CYGWIN_DIR").expect("Cygwin cannot be found. You can specify the Cygwin installation path with the CYGWIN_DIR environment variable.");

    let jl_include_path = format!("-I{}/include/julia/", julia_dir);
    let cygwin_include_path = format!("-I{}/usr/include", cygwin_path);
    let w32api_include_path = format!("-I{}/usr/include/w32api", cygwin_path);
    let jl_lib_path = format!("-L{}/bin/", julia_dir);

    println!("cargo:rustc-flags={}", &jl_lib_path);
    println!("cargo:rustc-link-lib=julia");
    vec![
        jl_include_path,
        cygwin_include_path,
        w32api_include_path,
        jl_lib_path,
    ]
}

#[cfg(target_os = "linux")]
fn flags() -> Vec<String> {
    let flags = match find_julia() {
        Some(julia_dir) => {
            let jl_include_path = format!("-I{}/include/julia/", julia_dir);
            let jl_lib_path = format!("-L{}/lib/", julia_dir);

            println!("cargo:rustc-flags={}", &jl_lib_path);
            vec![jl_include_path, jl_lib_path]
        }
        None => Vec::new(),
    };

    println!("cargo:rustc-link-lib=julia");
    flags
}

fn main() {
    let mut out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    out_path.push("bindings.rs");

    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-env-changed=JULIA_DIR");
    println!("cargo:rerun-if-env-changed=CYGWIN_DIR");

    if env::var("CARGO_FEATURE_DOCS_RS").is_ok() {
        fs::copy("dummy-bindings.rs", &out_path)
            .expect("Couldn't create bindings from dummy bindings.");
        return;
    }

    let flags = flags();

    // Only generate bindings if it's used by Jlrs
    let bindings = bindgen::Builder::default()
        .clang_args(&flags)
        .header("wrapper.h")
        .size_t_is_usize(true)
        .whitelist_function("jl_alloc_array_1d")
        .whitelist_function("jl_alloc_array_2d")
        .whitelist_function("jl_alloc_array_3d")
        .whitelist_function("jl_alloc_svec")
        .whitelist_function("jl_alloc_svec_uninit")
        .whitelist_function("jl_apply_array_type")
        .whitelist_function("jl_apply_tuple_type_v")
        .whitelist_function("jl_apply_type")
        .whitelist_function("jl_array_eltype")
        .whitelist_function("jl_atexit_hook")
        .whitelist_function("jl_box_bool")
        .whitelist_function("jl_box_char")
        .whitelist_function("jl_box_float32")
        .whitelist_function("jl_box_float64")
        .whitelist_function("jl_box_int16")
        .whitelist_function("jl_box_int32")
        .whitelist_function("jl_box_int64")
        .whitelist_function("jl_box_int8")
        .whitelist_function("jl_box_uint16")
        .whitelist_function("jl_box_uint32")
        .whitelist_function("jl_box_uint64")
        .whitelist_function("jl_box_uint8")
        .whitelist_function("jl_box_voidpointer")
        .whitelist_function("jl_call")
        .whitelist_function("jl_call0")
        .whitelist_function("jl_call1")
        .whitelist_function("jl_call2")
        .whitelist_function("jl_call3")
        .whitelist_function("jl_compute_fieldtypes")
        .whitelist_function("jl_egal")
        .whitelist_function("jl_eval_string")
        .whitelist_function("jl_exception_occurred")
        .whitelist_function("jl_field_index")
        .whitelist_function("jl_field_isdefined")
        .whitelist_function("jl_finalize")
        .whitelist_function("jl_gc_add_finalizer")
        .whitelist_function("jl_gc_collect")
        .whitelist_function("jl_gc_enable")
        .whitelist_function("jl_gc_is_enabled")
        .whitelist_function("jl_gc_queue_root")
        .whitelist_function("jl_gc_safepoint")
        .whitelist_function("jl_get_field")
        .whitelist_function("jl_get_global")
        .whitelist_function("jl_get_kwsorter")
        .whitelist_function("jl_get_nth_field")
        .whitelist_function("jl_get_nth_field_noalloc")
        .whitelist_function("jl_get_ptls_states")
        .whitelist_function("jl_init__threading")
        .whitelist_function("jl_init_with_image__threading")
        .whitelist_function("jl_is_initialized")
        .whitelist_function("jl_islayout_inline")
        .whitelist_function("jl_new_array")
        .whitelist_function("jl_new_struct_uninit")
        .whitelist_function("jl_new_structv")
        .whitelist_function("jl_object_id")
        .whitelist_function("jl_pchar_to_string")
        .whitelist_function("jl_pgcstack")
        .whitelist_function("jl_process_events")
        .whitelist_function("jl_ptr_to_array")
        .whitelist_function("jl_ptr_to_array_1d")
        .whitelist_function("jl_set_const")
        .whitelist_function("jl_set_global")
        .whitelist_function("jl_set_nth_field")
        .whitelist_function("jl_subtype")
        .whitelist_function("jl_symbol")
        .whitelist_function("jl_symbol_n")
        .whitelist_function("jl_tupletype_fill")
        .whitelist_function("jl_typename_str")
        .whitelist_function("jl_typeof_str")
        .whitelist_function("jl_type_union")
        .whitelist_function("jl_type_unionall")
        .whitelist_function("jl_unbox_float32")
        .whitelist_function("jl_unbox_float64")
        .whitelist_function("jl_unbox_int16")
        .whitelist_function("jl_unbox_int32")
        .whitelist_function("jl_unbox_int64")
        .whitelist_function("jl_unbox_int8")
        .whitelist_function("jl_unbox_uint16")
        .whitelist_function("jl_unbox_uint32")
        .whitelist_function("jl_unbox_uint64")
        .whitelist_function("jl_unbox_uint8")
        .whitelist_function("jl_unbox_voidpointer")
        .whitelist_type("jl_code_instance_t")
        .whitelist_type("jl_datatype_t")
        .whitelist_type("jl_expr_t")
        .whitelist_type("jl_fielddesc16_t")
        .whitelist_type("jl_fielddesc32_t")
        .whitelist_type("jl_fielddesc8_t")
        .whitelist_type("jl_methtable_t")
        .whitelist_type("jl_taggedvalue_t")
        .whitelist_type("jl_task_t")
        .whitelist_type("jl_typemap_entry_t")
        .whitelist_type("jl_typemap_level_t")
        .whitelist_type("jl_uniontype_t")
        .whitelist_type("jl_value_t")
        .whitelist_type("jl_weakref_t")
        .whitelist_var("jl_abstractarray_type")
        .whitelist_var("jl_abstractslot_type")
        .whitelist_var("jl_abstractstring_type")
        .whitelist_var("jl_addrspace_pointer_typename")
        .whitelist_var("jl_an_empty_string")
        .whitelist_var("jl_an_empty_vec_any")
        .whitelist_var("jl_anytuple_type")
        .whitelist_var("jl_anytuple_type_type")
        .whitelist_var("jl_any_type")
        .whitelist_var("jl_argumenterror_type")
        .whitelist_var("jl_array_any_type")
        .whitelist_var("jl_array_int32_type")
        .whitelist_var("jl_array_symbol_type")
        .whitelist_var("jl_array_type")
        .whitelist_var("jl_array_typename")
        .whitelist_var("jl_array_uint8_type")
        .whitelist_var("jl_base_module")
        .whitelist_var("jl_bool_type")
        .whitelist_var("jl_bottom_type")
        .whitelist_var("jl_boundserror_type")
        .whitelist_var("jl_builtin_type")
        .whitelist_var("jl_char_type")
        .whitelist_var("jl_code_info_type")
        .whitelist_var("jl_code_instance_type")
        .whitelist_var("jl_core_module")
        .whitelist_var("jl_datatype_type")
        .whitelist_var("jl_densearray_type")
        .whitelist_var("jl_diverror_exception")
        .whitelist_var("jl_egal")
        .whitelist_var("jl_emptysvec")
        .whitelist_var("jl_emptytuple")
        .whitelist_var("jl_emptytuple_type")
        .whitelist_var("jl_errorexception_type")
        .whitelist_var("jl_expr_type")
        .whitelist_var("jl_false")
        .whitelist_var("jl_float16_type")
        .whitelist_var("jl_float32_type")
        .whitelist_var("jl_float64_type")
        .whitelist_var("jl_floatingpoint_type")
        .whitelist_var("jl_function_type")
        .whitelist_var("jl_globalref_type")
        .whitelist_var("jl_gotonode_type")
        .whitelist_var("jl_incomplete_sym")
        .whitelist_var("jl_initerror_type")
        .whitelist_var("jl_int16_type")
        .whitelist_var("jl_int32_type")
        .whitelist_var("jl_int64_type")
        .whitelist_var("jl_int8_type")
        .whitelist_var("jl_interrupt_exception")
        .whitelist_var("jl_intrinsic_type")
        .whitelist_var("jl_lineinfonode_type")
        .whitelist_var("jl_linenumbernode_type")
        .whitelist_var("jl_llvmpointer_type")
        .whitelist_var("jl_llvmpointer_typename")
        .whitelist_var("jl_loaderror_type")
        .whitelist_var("jl_main_module")
        .whitelist_var("jl_memory_exception")
        .whitelist_var("jl_methoderror_type")
        .whitelist_var("jl_method_instance_type")
        .whitelist_var("jl_method_type")
        .whitelist_var("jl_methtable_type")
        .whitelist_var("jl_module_type")
        .whitelist_var("jl_namedtuple_type")
        .whitelist_var("jl_namedtuple_typename")
        .whitelist_var("jl_newvarnode_type")
        .whitelist_var("jl_nothing")
        .whitelist_var("jl_nothing_type")
        .whitelist_var("jl_number_type")
        .whitelist_var("jl_phicnode_type")
        .whitelist_var("jl_phinode_type")
        .whitelist_var("jl_pinode_type")
        .whitelist_var("jl_pointer_type")
        .whitelist_var("jl_pointer_typename")
        .whitelist_var("jl_quotenode_type")
        .whitelist_var("jl_readonlymemory_exception")
        .whitelist_var("jl_ref_type")
        .whitelist_var("jl_signed_type")
        .whitelist_var("jl_simplevector_type")
        .whitelist_var("jl_slotnumber_type")
        .whitelist_var("jl_ssavalue_type")
        .whitelist_var("jl_stackovf_exception")
        .whitelist_var("jl_string_type")
        .whitelist_var("jl_symbol_type")
        .whitelist_var("jl_task_type")
        .whitelist_var("jl_true")
        .whitelist_var("jl_tuple_type")
        .whitelist_var("jl_tuple_typename")
        .whitelist_var("jl_tvar_type")
        .whitelist_var("jl_typedslot_type")
        .whitelist_var("jl_typeerror_type")
        .whitelist_var("jl_typemap_entry_type")
        .whitelist_var("jl_typemap_level_type")
        .whitelist_var("jl_typename_type")
        .whitelist_var("jl_typeofbottom_type")
        .whitelist_var("jl_type_type")
        .whitelist_var("jl_type_typename")
        .whitelist_var("jl_typetype_type")
        .whitelist_var("jl_uint16_type")
        .whitelist_var("jl_uint32_type")
        .whitelist_var("jl_uint64_type")
        .whitelist_var("jl_uint8_type")
        .whitelist_var("jl_undefref_exception")
        .whitelist_var("jl_undefvarerror_type")
        .whitelist_var("jl_unionall_type")
        .whitelist_var("jl_uniontype_type")
        .whitelist_var("jl_upsilonnode_type")
        .whitelist_var("jl_vararg_type")
        .whitelist_var("jl_vararg_typename")
        .whitelist_var("jl_vecelement_typename")
        .whitelist_var("jl_voidpointer_type")
        .whitelist_var("jl_weakref_type")
        .whitelist_var("jl_weakref_typejl_abstractslot_type")
        .rustfmt_bindings(true)
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    bindings
        .write_to_file(&out_path)
        .expect("Couldn't write bindings!");
}
