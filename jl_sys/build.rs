#![allow(unused_imports)]

use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

// Many thanks to this comment on Github:
// https://github.com/rust-lang/rust-bindgen/issues/687#issuecomment-450750547
#[derive(Debug)]
struct IgnoreMacros(HashSet<String>);

impl bindgen::callbacks::ParseCallbacks for IgnoreMacros {
    fn will_parse_macro(&self, name: &str) -> bindgen::callbacks::MacroParsingBehavior {
        if self.0.contains(name) {
            bindgen::callbacks::MacroParsingBehavior::Ignore
        } else {
            bindgen::callbacks::MacroParsingBehavior::Default
        }
    }
}

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

    let ignored_macros = IgnoreMacros(
        vec![
            "FP_INFINITE".into(),
            "FP_NAN".into(),
            "FP_NORMAL".into(),
            "FP_SUBNORMAL".into(),
            "FP_ZERO".into(),
            "IPPORT_RESERVED".into(),
        ]
        .into_iter()
        .collect(),
    );

    // Only generate bindings if it's used by Jlrs
    let bindings = bindgen::Builder::default()
        .clang_args(&flags)
        .header("wrapper.h")
        .size_t_is_usize(true)
        .parse_callbacks(Box::new(ignored_macros))
        // Blacklist things that require 128-bits primitive types
        .blacklist_function("__acoshl")
        .blacklist_function("__acosl")
        .blacklist_function("__asinhl")
        .blacklist_function("__asinl")
        .blacklist_function("__atan2l")
        .blacklist_function("__atanhl")
        .blacklist_function("__atanl")
        .blacklist_function("__cbrtl")
        .blacklist_function("__ceill")
        .blacklist_function("__copysignl")
        .blacklist_function("__coshl")
        .blacklist_function("__cosl")
        .blacklist_function("__dreml")
        .blacklist_function("__erfcl")
        .blacklist_function("__erfl")
        .blacklist_function("__exp2l")
        .blacklist_function("__expl")
        .blacklist_function("__expm1l")
        .blacklist_function("__fabsl")
        .blacklist_function("__fdiml")
        .blacklist_function("__finitel")
        .blacklist_function("__floorl")
        .blacklist_function("__fmal")
        .blacklist_function("__fmaxl")
        .blacklist_function("__fminl")
        .blacklist_function("__fmodl")
        .blacklist_function("__fpclassifyl")
        .blacklist_function("__frexpl")
        .blacklist_function("__gammal")
        .blacklist_function("__hypotl")
        .blacklist_function("__ilogbl")
        .blacklist_function("__iseqsigl")
        .blacklist_function("__isinfl")
        .blacklist_function("__isnanl")
        .blacklist_function("__issignalingl")
        .blacklist_function("__j0l")
        .blacklist_function("__j1l")
        .blacklist_function("__jnl")
        .blacklist_function("__ldexpl")
        .blacklist_function("__lgammal")
        .blacklist_function("__lgammal_r")
        .blacklist_function("__llrintl")
        .blacklist_function("__llroundl")
        .blacklist_function("__log10l")
        .blacklist_function("__log1pl")
        .blacklist_function("__log2l")
        .blacklist_function("__logbl")
        .blacklist_function("__logl")
        .blacklist_function("__lrintl")
        .blacklist_function("__lroundl")
        .blacklist_function("__modfl")
        .blacklist_function("__nanl")
        .blacklist_function("__nearbyintl")
        .blacklist_function("__nextafterl")
        .blacklist_function("__nexttoward")
        .blacklist_function("__nexttowardf")
        .blacklist_function("__nexttowardl")
        .blacklist_function("__powl")
        .blacklist_function("__remainderl")
        .blacklist_function("__remquol")
        .blacklist_function("__rintl")
        .blacklist_function("__roundl")
        .blacklist_function("__scalbl")
        .blacklist_function("__scalblnl")
        .blacklist_function("__scalbnl")
        .blacklist_function("__signbitl")
        .blacklist_function("__significandl")
        .blacklist_function("__sinhl")
        .blacklist_function("__sinl")
        .blacklist_function("__sqrtl")
        .blacklist_function("__tanhl")
        .blacklist_function("__tanl")
        .blacklist_function("__tgammal")
        .blacklist_function("__truncl")
        .blacklist_function("__y0l")
        .blacklist_function("__y1l")
        .blacklist_function("__ynl")
        .blacklist_function("acoshl")
        .blacklist_function("acosl")
        .blacklist_function("asinhl")
        .blacklist_function("asinl")
        .blacklist_function("atan2l")
        .blacklist_function("atanhl")
        .blacklist_function("atanl")
        .blacklist_function("cbrtl")
        .blacklist_function("ceill")
        .blacklist_function("copysignl")
        .blacklist_function("coshl")
        .blacklist_function("cosl")
        .blacklist_function("dreml")
        .blacklist_function("erfcl")
        .blacklist_function("erfl")
        .blacklist_function("exp2l")
        .blacklist_function("expl")
        .blacklist_function("expm1l")
        .blacklist_function("fabsl")
        .blacklist_function("fdiml")
        .blacklist_function("finitel")
        .blacklist_function("floorl")
        .blacklist_function("fmal")
        .blacklist_function("fmaxl")
        .blacklist_function("fminl")
        .blacklist_function("fmodl")
        .blacklist_function("frexpl")
        .blacklist_function("gammal")
        .blacklist_function("hypotl")
        .blacklist_function("ilogbl")
        .blacklist_function("isinfl")
        .blacklist_function("isnanl")
        .blacklist_function("j0l")
        .blacklist_function("j1l")
        .blacklist_function("jnl")
        .blacklist_function("ldexpl")
        .blacklist_function("lgammal")
        .blacklist_function("lgammal_r")
        .blacklist_function("llrintl")
        .blacklist_function("llroundl")
        .blacklist_function("log10l")
        .blacklist_function("log1pl")
        .blacklist_function("log2l")
        .blacklist_function("logbl")
        .blacklist_function("logl")
        .blacklist_function("lrintl")
        .blacklist_function("lroundl")
        .blacklist_function("modfl")
        .blacklist_function("nanl")
        .blacklist_function("nearbyintl")
        .blacklist_function("nextafterl")
        .blacklist_function("nexttoward")
        .blacklist_function("nexttowardf")
        .blacklist_function("nexttowardl")
        .blacklist_function("powl")
        .blacklist_function("remainderl")
        .blacklist_function("remquol")
        .blacklist_function("rintl")
        .blacklist_function("roundl")
        .blacklist_function("scalbl")
        .blacklist_function("scalblnl")
        .blacklist_function("scalbnl")
        .blacklist_function("significandl")
        .blacklist_function("sinhl")
        .blacklist_function("sinl")
        .blacklist_function("sqrtl")
        .blacklist_function("tanhl")
        .blacklist_function("tanl")
        .blacklist_function("tgammal")
        .blacklist_function("truncl")
        .blacklist_function("y0l")
        .blacklist_function("y1l")
        .blacklist_function("ynl")
        .blacklist_function("strtold")
        .blacklist_function("qecvt")
        .blacklist_function("qfcvt")
        .blacklist_function("qgcvt")
        .blacklist_function("qecvt_r")
        .blacklist_function("qfcvt_r")
        .blacklist_function("_strtold_r")
        .blacklist_function("__C_specific_handler")
        .blacklist_function("__CppXcptFilter")
        .blacklist_function("_XcptFilter")
        .blacklist_function("RtlInitializeSListHead")
        .blacklist_function("RtlFirstEntrySList")
        .blacklist_function("RtlInterlockedPopEntrySList")
        .blacklist_function("RtlInterlockedFlushSList")
        .blacklist_function("RtlQueryDepthSList")
        .blacklist_function("UnhandledExceptionFilter")
        .blacklist_function("AddVectoredExceptionHandler")
        .blacklist_function("AddVectoredContinueHandler")
        .blacklist_function("InitializeSListHead")
        .blacklist_function("InterlockedFlushSList")
        .blacklist_function("InterlockedPopEntrySList")
        .blacklist_function("InterlockedPushEntrySList")
        .blacklist_function("SetXStateFeaturesMask")
        .blacklist_function("LocateXStateFeature")
        .blacklist_function("GetXStateFeaturesMask")
        .blacklist_function("CopyContext")
        .blacklist_function("InitializeContext")
        .blacklist_function("RaiseFailFastException")
        .blacklist_function("SetThreadContext")
        .blacklist_function("GetThreadContext")
        .blacklist_function("QueryDepthSList")
        .blacklist_function("RtlCaptureContext")
        .blacklist_function("RtlRestoreContext")
        .blacklist_function("RtlVirtualUnwind")
        .blacklist_function("RtlUnwindEx")
        .blacklist_function("RtlInterlockedPushEntrySList")
        .blacklist_function("RtlInterlockedPushListSListEx")
        .blacklist_function("SetUnhandledExceptionFilter")
        .rustfmt_bindings(true)
        .generate()
        .expect("Unable to generate bindings");
        
    // Write the bindings to the $OUT_DIR/bindings.rs file.
    bindings
        .write_to_file(&out_path)
        .expect("Couldn't write bindings!");
}
