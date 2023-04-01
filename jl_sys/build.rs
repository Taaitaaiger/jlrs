#[cfg(target_os = "windows")]
use std::str::FromStr;
use std::{env, path::PathBuf, process::Command};
#[cfg(target_os = "linux")]
use std::{ffi::OsStr, os::unix::prelude::OsStrExt};

use cfg_if::cfg_if;
#[cfg(feature = "use-bindgen")]
use fix_bindings::fix_bindings;

#[cfg(feature = "use-bindgen")]
#[path = "build/fix_bindings.rs"]
mod fix_bindings;

fn main() {
    if env::var("DOCS_RS").is_ok() {
        return;
    }

    println!("cargo:rerun-if-changed=src/jlrs_cc.cc");
    println!("cargo:rerun-if-changed=src/jlrs_cc.h");
    println!("cargo:rerun-if-env-changed=JULIA_DIR");

    let julia_dir =
        find_julia().expect("JULIA_DIR is not set and no installed version of Julia can be found");

    #[cfg(not(feature = "no-link"))]
    set_flags(&julia_dir);
    compile_jlrs_cc(&julia_dir);

    #[cfg(feature = "use-bindgen")]
    generate_bindings(&julia_dir);
}

fn find_julia() -> Option<String> {
    if let Ok(path) = env::var("JULIA_DIR") {
        return Some(path);
    }

    cfg_if! {
        if #[cfg(target_os = "linux")] {
            let out = Command::new("which").arg("julia").output().ok()?.stdout;
            let mut julia_path = PathBuf::from(OsStr::from_bytes(out.as_ref()));

            if !julia_path.pop() {
                return None;
            }

            if !julia_path.pop() {
                return None;
            }

            Some(julia_path.to_string_lossy().to_string())
        } else if #[cfg(target_os = "windows")] {
            let out = Command::new("cmd")
                .args(["/C", "where", "julia"])
                .output()
                .ok()?;
            let results = String::from_utf8(out.stdout).ok()?;

            let mut lines = results.lines();
            let first = lines.next()?;

            let mut julia_path = PathBuf::from_str(first).unwrap();

            if !julia_path.pop() {
                return None;
            }

            if !julia_path.pop() {
                return None;
            }

            Some(julia_path.to_string_lossy().to_string())
        } else {
            unimplemented!("Only Linux and Windows are supported")
        }
    }
}

#[cfg(not(feature = "no-link"))]
fn set_flags(julia_dir: &str) {
    cfg_if! {
        if #[cfg(target_os = "linux")] {
            println!("cargo:rustc-link-search={}/lib", &julia_dir);
            println!("cargo:rustc-link-arg=-Wl,--export-dynamic");

            cfg_if! {
                if #[cfg(feature = "debug")] {
                    println!("cargo:rustc-link-lib=julia-debug");
                } else {
                    println!("cargo:rustc-link-lib=julia");
                }
            }

            cfg_if! {
                if #[cfg(feature = "uv")] {
                    println!("cargo:rustc-link-search={}/lib/julia", &julia_dir);
                    println!("cargo:rustc-link-lib=uv");
                }
            }
        } else if #[cfg(target_os = "macos")] {
            println!("cargo:rustc-link-search={}/lib", &julia_dir);
            println!("cargo:rustc-link-arg=-Wl,--export-dynamic");

            cfg_if! {
                if #[cfg(feature = "debug")] {
                    println!("cargo:rustc-link-lib=julia-debug");
                } else {
                    println!("cargo:rustc-link-lib=julia");
                }
            }

            cfg_if! {
                if #[cfg(feature = "uv")] {
                    println!("cargo:rustc-link-search={}/lib/julia", &julia_dir);
                    println!("cargo:rustc-link-lib=uv");
                }
            }
        } else if #[cfg(all(target_os = "windows", target_env = "msvc"))] {
            println!("cargo:rustc-link-search={}/bin", &julia_dir);
            println!("cargo:rustc-link-search={}/lib", &julia_dir);
        } else if #[cfg(all(target_os = "windows", target_env = "gnu"))] {
            println!("cargo:rustc-link-search={}/bin", &julia_dir);

            cfg_if! {
                if #[cfg(feature = "debug")] {
                    println!("cargo:rustc-link-lib=julia-debug");
                } else {
                    println!("cargo:rustc-link-lib=julia");
                }
            }

            println!("cargo:rustc-link-lib=openlibm");
            println!("cargo:rustc-link-arg=-Wl,--stack,8388608");

            cfg_if! {
                if #[cfg(feature = "uv")] {
                    println!("cargo:rustc-link-lib=uv-2");
                }
            }
        } else {
            unreachable!()
        }
    }
}

fn compile_jlrs_cc(julia_dir: &str) {
    let include_dir = format!("{}/include/julia/", julia_dir);

    let mut c = cc::Build::new();
    c.file("src/jlrs_cc.cc")
        .include(&include_dir)
        .cpp(true)
        .flag_if_supported("-fPIC");

    #[cfg(target_os = "macos")]
    c.cpp(false);

    cfg_if! {
        if #[cfg(feature = "yggdrasil")] {
            #[cfg(feature = "i686")]
            {
                c.no_default_flags(true);
                c.flag("-O3");
            }

            #[cfg(feature = "windows")]
            c.flag("-mwindows");

            #[cfg(feature = "windows")]
            c.flag("-Wl,--no-undefined");
        } else {
            #[cfg(feature = "i686")]
            c.flag("-march=pentium4");

            #[cfg(target_env = "msvc")]
            {
                c.flag("/std:c++20");

                let julia_dll_a = format!("{}/lib/libjulia.dll.a", julia_dir);
                c.object(&julia_dll_a);
            }
        }
    }

    #[cfg(feature = "julia-1-6")]
    c.define("JULIA_1_6", None);

    #[cfg(all(any(windows, feature = "windows"), feature = "julia-1-6"))]
    c.define("JLRS_WINDOWS_LTS", None);

    #[cfg(feature = "julia-1-7")]
    c.define("JULIA_1_7", None);

    #[cfg(feature = "julia-1-8")]
    c.define("JULIA_1_8", None);

    #[cfg(feature = "julia-1-9")]
    c.define("JULIA_1_9", None);

    #[cfg(feature = "julia-1-10")]
    c.define("JULIA_1_10", None);

    c.compile("jlrs_cc");
}

#[cfg(feature = "use-bindgen")]
fn generate_bindings(julia_dir: &str) {
    let include_dir = format!("{}/include/julia/", &julia_dir);
    let mut out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    out_path.push("bindings.rs");

    let include_dir_flag = format!("-I{}", include_dir);

    #[allow(unused_mut)]
    let mut builder = bindgen::Builder::default();

    #[cfg(feature = "i686")]
    let arch_flag = "-march=pentium4";
    #[cfg(not(feature = "i686"))]
    let arch_flag = "";

    builder = builder.clang_arg(include_dir_flag).clang_arg(arch_flag);

    #[cfg(feature = "julia-1-6")]
    {
        builder = builder.clang_arg("-DJULIA_1_6");
    }

    #[cfg(feature = "julia-1-7")]
    {
        builder = builder.clang_arg("-DJULIA_1_7");
    }

    #[cfg(feature = "julia-1-8")]
    {
        builder = builder.clang_arg("-DJULIA_1_8");
    }

    #[cfg(feature = "julia-1-9")]
    {
        builder = builder.clang_arg("-DJULIA_1_9");
    }

    #[cfg(feature = "julia-1-10")]
    {
        builder = builder.clang_arg("-DJULIA_1_10");
    }

    #[cfg(all(feature = "julia-1-6", any(windows, feature = "windows")))]
    {
        builder = builder.clang_arg("-DJLRS_WINDOWS_LTS");
    }

    builder = builder
        .header("src/jlrs_cc.h")
        .size_t_is_usize(true)
        .layout_tests(false)
        .allowlist_function("jl_adopt_thread")
        .allowlist_function("jl_alloc_array_1d")
        .allowlist_function("jl_alloc_array_2d")
        .allowlist_function("jl_alloc_array_3d")
        .allowlist_function("jl_alloc_svec")
        .allowlist_function("jl_alloc_svec_uninit")
        .allowlist_function("jl_apply_array_type")
        .allowlist_function("jl_apply_generic")
        .allowlist_function("jl_apply_tuple_type_v")
        .allowlist_function("jl_apply_type")
        .allowlist_function("jl_array_ptr_1d_push")
        .allowlist_function("jl_array_ptr_1d_append")
        .allowlist_function("jl_array_del_beg")
        .allowlist_function("jl_array_del_end")
        .allowlist_function("jl_array_eltype")
        .allowlist_function("jl_array_grow_beg")
        .allowlist_function("jl_array_grow_end")
        .allowlist_function("jl_array_typetagdata")
        .allowlist_function("jl_arrayset")
        .allowlist_function("jl_arrayref")
        .allowlist_function("jl_atexit_hook")
        .allowlist_function("jl_atomic_cmpswap_bits")
        .allowlist_function("jl_atomic_bool_cmpswap_bits")
        .allowlist_function("jl_atomic_new_bits")
        .allowlist_function("jl_atomic_store_bits")
        .allowlist_function("jl_atomic_swap_bits")
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
        .allowlist_function("jl_environ")
        .allowlist_function("jl_enter_threaded_region")
        .allowlist_function("jl_eval_string")
        .allowlist_function("jl_exit_threaded_region")
        .allowlist_function("jl_exception_occurred")
        .allowlist_function("jl_field_index")
        .allowlist_function("jl_gc_add_finalizer")
        .allowlist_function("jl_gc_add_ptr_finalizer")
        .allowlist_function("jl_gc_alloc_typed")
        .allowlist_function("jl_gc_collect")
        .allowlist_function("jl_gc_enable")
        .allowlist_function("jl_gc_is_enabled")
        .allowlist_function("jl_gc_mark_queue_obj")
        .allowlist_function("jl_gc_mark_queue_objarray")
        .allowlist_function("jl_gc_queue_root")
        .allowlist_function("jl_gc_safepoint")
        .allowlist_function("jl_gc_schedule_foreign_sweepfunc")
        .allowlist_function("jl_gc_set_max_memory")
        .allowlist_function("jl_gensym")
        .allowlist_function("jl_get_binding_type")
        .allowlist_function("jl_get_current_task")
        .allowlist_function("jl_get_global")
        .allowlist_function("jl_get_libllvm")
        .allowlist_function("jl_get_kwsorter")
        .allowlist_function("jl_get_nth_field")
        .allowlist_function("jl_get_nth_field_noalloc")
        .allowlist_function("jl_get_ptls_states")
        .allowlist_function("jl_get_ARCH")
        .allowlist_function("jl_get_UNAME")
        .allowlist_function("jl_getallocationgranularity")
        .allowlist_function("jl_getpagesize")
        .allowlist_function("jl_git_branch")
        .allowlist_function("jl_git_commit")
        .allowlist_function("jl_has_free_typevars")
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
        .allowlist_function("jl_kwcall_func")
        .allowlist_function("jl_new_array")
        .allowlist_function("jl_new_datatype")
        .allowlist_function("jl_new_foreign_type")
        .allowlist_function("jl_new_module")
        .allowlist_function("jl_new_primitivetype")
        .allowlist_function("jl_new_struct_uninit")
        .allowlist_function("jl_new_structv")
        .allowlist_function("jl_new_typevar")
        .allowlist_function("jl_object_id")
        .allowlist_function("jl_pchar_to_array")
        .allowlist_function("jl_pchar_to_string")
        .allowlist_function("jl_process_events")
        .allowlist_function("jl_ptr_to_array")
        .allowlist_function("jl_ptr_to_array_1d")
        .allowlist_function("jl_reinit_foreign_type")
        .allowlist_function("jl_reshape_array")
        .allowlist_function("jl_set_const")
        .allowlist_function("jl_set_global")
        .allowlist_function("jl_set_nth_field")
        .allowlist_function("jl_stderr_obj")
        .allowlist_function("jl_stdout_obj")
        .allowlist_function("jl_subtype")
        .allowlist_function("jl_symbol")
        .allowlist_function("jl_symbol_n")
        .allowlist_function("jl_tagged_gensym")
        .allowlist_function("jl_throw")
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
        .allowlist_function("jlrs_catch_wrapper")
        .allowlist_function("jlrs_lock")
        .allowlist_function("jlrs_unlock")
        .allowlist_function("jlrs_array_data_owner_offset")
        .allowlist_function("jlrs_gc_queue_multiroot")
        .allowlist_function("jl_setjmp")
        .allowlist_function("jl_excstack_state")
        .allowlist_function("jl_enter_handler")
        .allowlist_function("jl_eh_restore_state")
        .allowlist_function("jl_current_exception")
        .allowlist_function("jl_restore_excstack")
        .allowlist_function("jl_get_pgcstack")
        .allowlist_type("jl_binding_t")
        .allowlist_type("jl_callptr_t")
        .allowlist_type("jl_code_instance_t")
        .allowlist_type("jl_datatype_t")
        .allowlist_type("jl_expr_t")
        .allowlist_type("jl_fielddesc16_t")
        .allowlist_type("jl_fielddesc32_t")
        .allowlist_type("jl_fielddesc8_t")
        .allowlist_type("jl_fptr_sparam_t")
        .allowlist_type("jl_method_match_t")
        .allowlist_type("jl_methtable_t")
        .allowlist_type("jl_opaque_closure_t")
        .allowlist_type("jl_options_t")
        .allowlist_type("jl_taggedvalue_t")
        .allowlist_type("jl_task_t")
        .allowlist_type("jl_typemap_t")
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
        .allowlist_var("jl_kwcall_func")
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
        .allowlist_var("jl_options")
        .allowlist_var("jl_pair_type")
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
        .opaque_type("uv_mutex_t")
        .opaque_type("uv_cond_t");

    #[cfg(feature = "julia-1-10")]
    {
        builder = builder.allowlist_var("jl_binding_type");
    }

    #[cfg(not(feature = "julia-1-10"))]
    {
        builder = builder.allowlist_function("jl_binding_type");
    }

    let bindings = builder.generate().expect("Unable to generate bindings");

    let mut bindings_bytes = Vec::new();
    bindings
        .write(Box::new(&mut bindings_bytes))
        .expect("Couldn't write to vec");

    let bindings_str = String::from_utf8(bindings_bytes).unwrap();
    fix_bindings(&include_dir, &bindings_str, &out_path);
}
