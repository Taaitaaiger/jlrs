use std::env;

use cfg_if::cfg_if;
use julia_metadata::{
    BINARY_BUILDER_BUILD, emit_detected_version_metadata, enable_version_cfgs, include_dir,
};

fn main() {
    enable_version_cfgs();
    emit_detected_version_metadata(false);

    println!("cargo::rerun-if-changed=src/jlrs_cc/jlrs_cc_ext.c");
    println!("cargo::rerun-if-changed=src/jlrs_cc/jlrs_cc_hacks.c");
    println!("cargo::rerun-if-changed=src/jlrs_cc/jlrs_cc_reexport.c");
    println!("cargo::rerun-if-changed=src/jlrs_cc/jlrs_cc_fast_tls.c");
    println!("cargo::rerun-if-changed=src/jlrs_cc/jlrs_cc_ext.h");
    println!("cargo::rerun-if-changed=src/jlrs_cc/jlrs_cc_hacks.h");
    println!("cargo::rerun-if-changed=src/jlrs_cc/jlrs_cc_reexport.h");
    println!("cargo::rerun-if-changed=src/jlrs_cc/jlrs_cc.h");
    println!("cargo::rerun-if-changed=src/jlrs_cc/jlrs_cc_fast_tls.h");

    println!("cargo::rerun-if-env-changed=JLRS_JULIA_DIR");
    println!("cargo::rerun-if-env-changed=PATH");
    println!("cargo::rerun-if-env-changed=LD_LIBRARY_PATH");
    println!("cargo::rerun-if-env-changed=DYLD_LIBRARY_PATH");

    compile_jlrs_cc();
}

#[derive(Clone, Copy)]
enum BBTarget {
    Windows,
    WindowsI686,
    I686,
    Arm,
    Riscv,
    AArch,
}

// Interpret `bb_target` to
fn interpret_binary_builder_target() -> Option<BBTarget> {
    if BINARY_BUILDER_BUILD {
        if let Ok(target) = env::var("bb_target") {
            if target.contains("w64") {
                if target.contains("i686") {
                    return Some(BBTarget::WindowsI686);
                }
                return Some(BBTarget::Windows);
            }

            if target.contains("i686") {
                return Some(BBTarget::I686);
            }

            if target.contains("aarch") {
                return Some(BBTarget::AArch);
            }

            if target.contains("arm") {
                return Some(BBTarget::Arm);
            }

            if target.contains("riscv") {
                return Some(BBTarget::Riscv);
            }
        }
    }

    None
}

#[allow(unused_variables)]
fn compile_jlrs_cc() {
    let include_dir = include_dir();
    let mut c = cc::Build::new();
    c.include(&include_dir)
        .cpp(false)
        .flag_if_supported("-fPIC")
        .flag_if_supported("-Wno-nullability-completeness");

    if BINARY_BUILDER_BUILD {
        c.file("src/jlrs_cc/jlrs_cc_ext.c")
            .file("src/jlrs_cc/jlrs_cc_reexport.c")
            .file("src/jlrs_cc/jlrs_cc_hacks.c")
            .file("src/jlrs_cc/jlrs_cc_fast_tls.c");

        let target = interpret_binary_builder_target();
        match target {
            Some(BBTarget::I686 | BBTarget::Arm | BBTarget::AArch | BBTarget::Riscv) => {
                c.no_default_flags(true).flag("-O3").flag("-fPIC");
            }
            Some(BBTarget::Windows) => {
                c.flag("-mwindows");
            }
            Some(BBTarget::WindowsI686) => {
                c.no_default_flags(true).flag("-O3").flag("-mwindows");
            }
            _ => (),
        }
    } else {
        #[cfg(feature = "i686")]
        c.flag("-march=pentium4");

        cfg_if! {
            if #[cfg(target_env = "msvc")] {
                let julia_dll_a = format!("{}/libjulia.dll.a", ::julia_metadata::lib_dir().display());
                c.file("src/jlrs_cc/jlrs_cc_ext.cc")
                    .file("src/jlrs_cc/jlrs_cc_reexport.cc")
                    .file("src/jlrs_cc/jlrs_cc_hacks.cc")
                    .file("src/jlrs_cc/jlrs_cc_fast_tls.cc")
                    .cpp(true)
                    .flag("/std:c++20")
                    .object(&julia_dll_a);
            } else {
                c
                    .file("src/jlrs_cc/jlrs_cc_ext.c")
                    .file("src/jlrs_cc/jlrs_cc_reexport.c")
                    .file("src/jlrs_cc/jlrs_cc_hacks.c")
                    .file("src/jlrs_cc/jlrs_cc_fast_tls.c");
            }
        }
    }

    #[cfg(feature = "lto")]
    c.flag("-flto=thin");

    // Enable fast (i.e. local-exec) TLS. Only enable this feature if you're embedding Julia in a
    // Rust application.
    if !BINARY_BUILDER_BUILD {
        #[cfg(feature = "fast-tls")]
        c.define("JLRS_FAST_TLS", None);
    }

    c.link_lib_modifier("+whole-archive");
    c.compile("jlrs_cc");
}
