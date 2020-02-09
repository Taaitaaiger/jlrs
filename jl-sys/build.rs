use std::env;
use std::path::PathBuf;

fn main() {
    let flags = match env::var("JL_PATH") {
        Ok(path) => {
            let jl_include_path = format!("-I{}/include/julia/", path);
            let jl_lib_path = format!("-L{}/lib/", path);
            println!("cargo:rustc-flags={}", &jl_lib_path);

            vec![jl_include_path, jl_lib_path]
        },
        _ => {
            vec![]
        }
    };

    println!("cargo:rustc-link-lib=julia");
    println!("cargo:rerun-if-changed=wrapper.h");

    // Only generate bindings if it's used by Jlrs
    let bindings = bindgen::Builder::default()
        .clang_args(&flags)
        .header("wrapper.h")
        // Initializing and stopping
        .whitelist_function("jl_init__threading")
        .whitelist_function("jl_is_initialized")
        .whitelist_function("jl_atexit_hook")
        // Gc
        .whitelist_function("jl_pgcstack")
        .whitelist_function("jl_get_ptls_states")
        .whitelist_function("jl_gc_queue_root")
        // boxing and unboxing primitives
        .whitelist_function("jl_box_bool")
        .whitelist_function("jl_box_char")
        .whitelist_function("jl_box_int8")
        .whitelist_function("jl_unbox_int8")
        .whitelist_function("jl_box_int16")
        .whitelist_function("jl_unbox_int16")
        .whitelist_function("jl_box_int32")
        .whitelist_function("jl_unbox_int32")
        .whitelist_function("jl_box_int64")
        .whitelist_function("jl_unbox_int64")
        .whitelist_function("jl_box_uint8")
        .whitelist_function("jl_unbox_uint8")
        .whitelist_function("jl_box_uint16")
        .whitelist_function("jl_unbox_uint16")
        .whitelist_function("jl_box_uint32")
        .whitelist_function("jl_unbox_uint32")
        .whitelist_function("jl_box_uint64")
        .whitelist_function("jl_unbox_uint64")
        .whitelist_function("jl_box_float32")
        .whitelist_function("jl_unbox_float32")
        .whitelist_function("jl_box_float64")
        .whitelist_function("jl_unbox_float64")
        // call functions
        .whitelist_function("jl_call0")
        .whitelist_function("jl_call1")
        .whitelist_function("jl_call2")
        .whitelist_function("jl_call3")
        .whitelist_function("jl_call")
        .whitelist_function("jl_exception_occurred")
        .whitelist_function("jl_eval_string")
        // symbols and globals
        .whitelist_function("jl_symbol_n")
        .whitelist_function("jl_get_global")
        .whitelist_function("jl_set_global")
        // n-dimensional arrays
        .whitelist_function("jl_apply_array_type")
        .whitelist_function("jl_array_eltype")
        .whitelist_function("jl_alloc_array_1d")
        .whitelist_function("jl_alloc_array_2d")
        .whitelist_function("jl_alloc_array_3d")
        .whitelist_function("jl_new_array")
        .whitelist_function("jl_ptr_to_array_1d")
        .whitelist_function("jl_ptr_to_array")
        // strings
        .whitelist_function("jl_pchar_to_string")
        .whitelist_function("jl_typeof_str")
        // modules
        .whitelist_var("jl_base_module")
        .whitelist_var("jl_core_module")
        .whitelist_var("jl_main_module")
        // types
        .whitelist_type("jl_value_t")
        .whitelist_type("jl_taggedvalue_t")
        .whitelist_type("jl_datatype_t")
        .whitelist_var("jl_bool_type")
        .whitelist_var("jl_char_type")
        .whitelist_var("jl_int8_type")
        .whitelist_var("jl_int16_type")
        .whitelist_var("jl_int32_type")
        .whitelist_var("jl_int64_type")
        .whitelist_var("jl_uint8_type")
        .whitelist_var("jl_uint16_type")
        .whitelist_var("jl_uint32_type")
        .whitelist_var("jl_uint64_type")
        .whitelist_var("jl_float32_type")
        .whitelist_var("jl_float64_type")
        .whitelist_var("jl_string_type")
        .whitelist_var("jl_datatype_type")
        .whitelist_var("jl_array_typename")
        .whitelist_var("jl_module_type")
        .rustfmt_bindings(true)
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
