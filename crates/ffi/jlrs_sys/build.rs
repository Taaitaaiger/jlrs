use std::env;

use cfg_if::cfg_if;
use find_julia::{JuliaDir, Version, enable_version_cfgs};
use jlrs_compat::{
    MAX_MINOR_VERSION, MIN_MINOR_VERSION, STABLE_MAJOR_VERSION, STABLE_MINOR_VERSION,
};

fn main() {
    // Enable julia_1_x configs
    enable_version_cfgs(MIN_MINOR_VERSION, MAX_MINOR_VERSION);

    if building_docs() {
        let version = Version::new(STABLE_MAJOR_VERSION, STABLE_MINOR_VERSION, 0, false);
        version.emit_metadata_unchecked();
        return;
    }

    // Load julia_dir info from metadata by jl-sys, and re-emit version metadata
    let julia_dir = {
        let julia_dir = JuliaDir::from_detected().expect("Julia not detected by jl-sys");
        julia_dir.version().emit_metadata_unchecked();
        julia_dir
    };

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

    let target = interpret_binary_builder_target(julia_dir.is_binary_builder());
    compile_jlrs_cc(&julia_dir, target);
    set_flags(&julia_dir, target);
}

#[derive(Clone, Copy)]
enum BBTarget {
    Windows,
    WindowsI686,
    I686,
    Arm,
    AArch,
}

// Interpret `bb_target` to
fn interpret_binary_builder_target(is_binary_builder: bool) -> Option<BBTarget> {
    if is_binary_builder {
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
        }
    }

    None
}

fn set_flags(_julia_dir: &JuliaDir, _target: Option<BBTarget>) {
    // if julia_dir.is_binary_builder() {

    // } else {
    //     cfg_if! {
    //         if #[cfg(any(target_os = "linux", feature = "windows", feature = "macos"))] {
    //             println!("cargo::rustc-link-arg=-rdynamic");
    //         } else if #[cfg(any(all(target_os = "windows", target_env = "gnu"), feature = "windows"))] {
    //             let bin_dir = julia_dir.bin_dir();
    //             println!("cargo::rustc-link-search={}", bin_dir.display());

    //             println!("cargo::rustc-link-lib=openlibm");
    //             println!("cargo::rustc-link-lib=libuv-2");
    //             println!("cargo::rustc-link-arg=-Wl,--stack,8388608");
    //         } else {
    //             panic!("Unsupported platform")
    //         }
    //     }
    // }
}

#[allow(unused_variables)]
fn compile_jlrs_cc(julia_dir: &JuliaDir, target: Option<BBTarget>) {
    let include_dir = julia_dir.include_dir();
    let mut c = cc::Build::new();
    c.include(&include_dir)
        .cpp(false)
        .flag_if_supported("-fPIC");

    if julia_dir.is_binary_builder() {
        c.file("src/jlrs_cc/jlrs_cc_ext.c")
            .file("src/jlrs_cc/jlrs_cc_reexport.c")
            .file("src/jlrs_cc/jlrs_cc_hacks.c")
            .file("src/jlrs_cc/jlrs_cc_fast_tls.c");

        match target {
            Some(BBTarget::I686 | BBTarget::Arm | BBTarget::AArch) => {
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
                let julia_dll_a = format!("{}/libjulia.dll.a", julia_dir.lib_dir().display());
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
    if !julia_dir.is_binary_builder() {
        #[cfg(feature = "fast-tls")]
        c.define("JLRS_FAST_TLS", None);
    }

    c.compile("jlrs_cc");
}

fn building_docs() -> bool {
    if env::var("DOCS_RS").is_ok() {
        return true;
    }

    #[cfg(feature = "docs")]
    return true;

    #[cfg(not(feature = "docs"))]
    return false;
}
