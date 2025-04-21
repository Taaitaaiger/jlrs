#[cfg(not(feature = "yggdrasil"))]
use std::path::PathBuf;
#[cfg(not(feature = "yggdrasil"))]
use std::process::Command;
#[cfg(target_os = "windows")]
use std::str::FromStr;
use std::{env, io::Read};
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "freebsd"))]
use std::{ffi::OsStr, os::unix::prelude::OsStrExt};

use cfg_if::cfg_if;

fn main() {
    #[cfg(feature = "docs")]
    {
        // TODO: depend on activated feature
        println!("cargo::metadata=version=1.12.0");
        println!("cargo::rustc-cfg=feature=\"julia-1-12\"");
        return;
    }

    if env::var("DOCS_RS").is_ok() {
        // TODO: depend on activated feature
        println!("cargo::metadata=version=1.12.0");
        println!("cargo::rustc-cfg=feature=\"julia-1-12\"");
        return;
    }

    println!("cargo::rerun-if-changed=src/jlrs_cc/jlrs_cc_ext.c");
    println!("cargo::rerun-if-changed=src/jlrs_cc/jlrs_version.c");
    println!("cargo::rerun-if-changed=src/jlrs_cc/jlrs_cc_hacks.c");
    println!("cargo::rerun-if-changed=src/jlrs_cc/jlrs_cc_reexport.c");
    println!("cargo::rerun-if-changed=src/jlrs_cc/jlrs_cc_fast_tls.c");
    println!("cargo::rerun-if-changed=src/jlrs_cc/jlrs_cc_ext.h");
    println!("cargo::rerun-if-changed=src/jlrs_cc/jlrs_cc_hacks.h");
    println!("cargo::rerun-if-changed=src/jlrs_cc/jlrs_cc_reexport.h");
    println!("cargo::rerun-if-changed=src/jlrs_cc/jlrs_cc.h");
    println!("cargo::rerun-if-changed=src/jlrs_cc/jlrs_cc_fast_tls.h");
    println!("cargo::rerun-if-env-changed=JULIA_DIR");
    println!("cargo::rerun-if-env-changed=PATH");
    println!("cargo::rerun-if-env-changed=LD_LIBRARY_PATH");
    println!("cargo::rerun-if-env-changed=DYLD_LIBRARY_PATH");

    detect_julia_version();

    let julia_dir = find_julia_dir()
        .expect("JULIA_DIR is not set and no installed version of Julia can be found");

    let julia_dir = julia_dir
        .as_os_str()
        .to_str()
        .expect("path contains invalid characters");

    let target = interpret_target();
    compile_jlrs_cc(&julia_dir, target);
    set_flags(&julia_dir, target);
}

fn detect_julia_version() {
    let mut julia_dir = find_julia_dir()
        .expect("JULIA_DIR is not set and no installed version of Julia can be found");

    julia_dir.push("include/julia/julia_version.h");

    let mut julia_version_file =
        std::fs::File::open(julia_dir).expect("Cannot find julia_version.h");

    let mut buf = Vec::new();
    julia_version_file
        .read_to_end(&mut buf)
        .expect("Cannot read julia_version.h");

    let julia_version_content = String::from_utf8(buf).expect("Not UTF8");
    let mut major = -1;
    let mut minor = -1;
    let mut patch = -1;
    let mut is_dev = false;

    for line in julia_version_content.lines() {
        if let Some(m) = line.strip_prefix("#define JULIA_VERSION_STRING ") {
            is_dev = m.contains("DEV");
            continue;
        }
        if let Some(m) = line.strip_prefix("#define JULIA_VERSION_MAJOR ") {
            major = m.parse().unwrap();
            continue;
        }
        if let Some(m) = line.strip_prefix("#define JULIA_VERSION_MINOR ") {
            minor = m.parse().unwrap();
            continue;
        }
        if let Some(m) = line.strip_prefix("#define JULIA_VERSION_PATCH ") {
            patch = m.parse().unwrap();
            continue;
        }
    }

    assert!(major != -1 && minor != -1 && patch != -1);
    if is_dev {
        println!("cargo::warning=Detected development version of Julia {major}.{minor}.{patch}, bindings may not be up-to-date. \
        Please report any issues you encounter at https://www.github.com/Taaitaaiger/jlrs/issues");
    }

    println!("cargo::metadata=version={major}.{minor}.{patch}");
    println!("cargo::rustc-cfg=feature=\"julia-{}-{}\"", major, minor);
}

#[cfg(feature = "yggdrasil")]
fn find_julia_dir() -> Option<PathBuf> {
    let mut path = PathBuf::from(env::var_os("WORKSPACE")?);
    path.push("destdir");

    return Some(path);
}

#[cfg(not(feature = "yggdrasil"))]
fn find_julia_dir() -> Option<PathBuf> {
    if let Some(path) = env::var_os("JULIA_DIR") {
        return Some(PathBuf::from(path));
    }

    cfg_if! {
        if #[cfg(any(target_os = "linux", target_os = "macos", target_os = "freebsd"))] {
            let out = Command::new("which").arg("julia").output().ok()?.stdout;
            let mut julia_path = PathBuf::from(OsStr::from_bytes(out.as_ref()));

            if !julia_path.pop() {
                return None;
            }

            if !julia_path.pop() {
                return None;
            }

            Some(julia_path)
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

            Some(julia_path)
        } else {
            unimplemented!("Julia detection not implemented for this platform, try setting JULIA_DIR")
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
enum Target {
    Windows,
    WindowsI686,
    I686,
}

#[cfg(not(feature = "yggdrasil"))]
fn interpret_target() -> Option<Target> {
    return None;
}

#[cfg(feature = "yggdrasil")]
fn interpret_target() -> Option<Target> {
    if let Ok(target) = env::var("target") {
        if target.contains("w64") {
            if target.contains("i686") {
                return Some(Target::WindowsI686);
            }
            return Some(Target::Windows);
        }

        if target.contains("i686") || target.contains("arm") {
            return Some(Target::I686);
        }
    }

    None
}

#[cfg(feature = "no-link")]
fn set_flags(julia_dir: &str, target: Option<Target>) {
    if let Some(Target::WindowsI686) = target {
        // Linking is necessary until raw dylib linkage is supported for this target
        println!("cargo::rustc-link-lib=julia");
        println!("cargo::rustc-link-lib=uv-2");
    }
}

#[cfg(not(feature = "no-link"))]
fn set_flags(julia_dir: &str, _tgt: Option<Target>) {
    cfg_if! {
        if #[cfg(all(target_os = "linux", not(any(feature = "windows", feature = "macos"))))] {
            println!("cargo::rustc-link-arg=-rdynamic");
            println!("cargo::rustc-link-search={}/lib", julia_dir);

            cfg_if! {
                if #[cfg(feature = "debug")] {
                    println!("cargo::rustc-link-lib=julia-debug");
                } else {
                    println!("cargo::rustc-link-lib=julia");
                }
            }
        } else if #[cfg(any(target_os = "macos", target_os = "freebsd", feature = "macos"))] {
            println!("cargo::rustc-link-search={}/lib", &julia_dir);

            cfg_if! {
                if #[cfg(feature = "debug")] {
                    println!("cargo::rustc-link-lib=julia-debug");
                } else {
                    println!("cargo::rustc-link-lib=julia");
                }
            }
        } else if #[cfg(all(target_os = "windows", target_env = "msvc"))] {
            println!("cargo::rustc-link-search={}/bin", &julia_dir);
            println!("cargo::rustc-link-search={}/lib", &julia_dir);
        } else if #[cfg(any(all(target_os = "windows", target_env = "gnu"), feature = "windows"))] {
            println!("cargo::rustc-link-search={}/bin", &julia_dir);

            cfg_if! {
                if #[cfg(feature = "debug")] {
                    println!("cargo::rustc-link-lib=julia-debug");
                } else {
                    println!("cargo::rustc-link-lib=julia");
                }
            }

            println!("cargo::rustc-link-lib=openlibm");
            println!("cargo::rustc-link-lib=libuv-2");
            println!("cargo::rustc-link-arg=-Wl,--stack,8388608");
        } else {
            panic!("Unsupported platform")
        }
    }
}

#[allow(unused_variables)]
fn compile_jlrs_cc(julia_dir: &str, target: Option<Target>) {
    let include_dir = format!("{}/include/julia/", julia_dir);

    let mut c = cc::Build::new();
    c.include(&include_dir)
        .cpp(false)
        .flag_if_supported("-fPIC");

    cfg_if! {
        if #[cfg(feature = "yggdrasil")] {
            c.file("src/jlrs_cc/jlrs_cc_ext.c")
                .file("src/jlrs_cc/jlrs_cc_reexport.c")
                .file("src/jlrs_cc/jlrs_cc_hacks.c")
                .file("src/jlrs_cc/jlrs_cc_fast_tls.c");

            match target {
                Some(Target::I686) => {
                    c.no_default_flags(true)
                        .flag("-O3")
                        .flag("-fPIC");
                }
                Some(Target::Windows) => {
                    c.flag("-mwindows");
                }
                Some(Target::WindowsI686) => {
                    c.no_default_flags(true)
                        .flag("-O3")
                        .flag("-mwindows");
                }
                _ => ()
            }
        } else {
            #[cfg(feature = "i686")]
            c.flag("-march=pentium4");

            cfg_if! {
                if #[cfg(target_env = "msvc")] {
                    let julia_dll_a = format!("{}/lib/libjulia.dll.a", julia_dir);
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
    }

    #[cfg(feature = "lto")]
    c.flag("-flto=thin");

    // Enable fast (i.e. local-exec) TLS. Only enable this feature if you're embedding Julia in a
    // Rust application.
    #[cfg(all(feature = "fast-tls", not(feature = "yggdrasil")))]
    c.define("JLRS_FAST_TLS", None);

    c.compile("jlrs_cc");
}
