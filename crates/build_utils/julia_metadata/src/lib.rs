use std::{ffi::OsString, path::PathBuf};

include!(concat!(env!("OUT_DIR"), "/metadata.rs"));
// pub const MAJOR_VERSION: usize = {};
// pub const MIN_MINOR_VERSION: usize = {};
// pub const MAX_MINOR_VERSION: usize = {};
// pub const NIGHTLY_MINOR_VERSION: usize = {};
// pub const SELECTED_MINOR_VERSION: usize = {};
// pub const DOCS_BUILD: bool = {};
// pub const DEBUG_BUILD: bool = {};
// pub const BINARY_BUILDER_BUILD: bool = {};
// pub static JULIA_DIR: &[u8] = &[{}];

/// Enable the `julia_1_x` version configs from `min_version..=max_version`.
pub fn enable_version_cfgs() {
    let versions = (MIN_MINOR_VERSION..=NIGHTLY_MINOR_VERSION)
        .map(|minor| format!("julia_1_{minor}"))
        .collect::<Vec<_>>()
        .join(",");

    println!("cargo::rustc-check-cfg=cfg({versions})");
}

pub fn emit_metadata() {
    if DOCS_BUILD {
        let major = MAJOR_VERSION;
        let minor = SELECTED_MINOR_VERSION;

        println!("cargo::rustc-cfg=julia_{major}_{minor}");
    } else {
        emit_version_metadata(
            MAJOR_VERSION as _,
            SELECTED_MINOR_VERSION as _,
            SELECTED_PATCH_VERSION as _,
            MIN_MINOR_VERSION as _,
            MAX_MINOR_VERSION as _,
            NIGHTLY_MINOR_VERSION as _,
            true,
        );
    }
}

pub fn emit_detected_version_metadata(warn: bool) {
    emit_version_metadata(
        MAJOR_VERSION as _,
        SELECTED_MINOR_VERSION as _,
        SELECTED_PATCH_VERSION as _,
        MIN_MINOR_VERSION as _,
        MAX_MINOR_VERSION as _,
        NIGHTLY_MINOR_VERSION as _,
        warn,
    );
}

pub fn link_julia() {
    if DOCS_BUILD {
        return;
    }

    if !BINARY_BUILDER_BUILD {
        let lib_dir = lib_dir();
        println!("cargo::rustc-link-search={}", lib_dir.display());
        println!("cargo::rustc-link-search={}/julia", lib_dir.display());

        if DEBUG_BUILD {
            println!("cargo::rustc-link-lib=julia-debug");
        } else {
            println!("cargo::rustc-link-lib=julia");
        }
    }

    if WINDOWS_BUILD {
        println!("cargo::rustc-link-arg=-Wl,--stack,8388608");
    }
}

fn julia_dir() -> PathBuf {
    let julia_dir = unsafe { OsString::from_encoded_bytes_unchecked(JULIA_DIR.to_vec()) };
    PathBuf::from(julia_dir)
}

pub fn lib_dir() -> PathBuf {
    let mut julia_dir = julia_dir();
    julia_dir.push("lib");
    julia_dir
}

pub fn include_dir() -> PathBuf {
    let mut julia_dir = julia_dir();
    julia_dir.push("include/julia");
    julia_dir
}

fn emit_version_metadata(
    major: u32,
    minor: u32,
    patch: u32,
    min_minor_version: u32,
    max_minor_version: u32,
    nightly_minor_version: u32,
    warn: bool,
) {
    if major != 1 {
        panic!("Detected unsupported version of Julia {major}.{minor}.{patch}.");
    }

    if DEV_BUILD {
        if warn {
            println!(
                "cargo::warning=Detected development version of Julia {major}.{minor}.{patch}, \
                    bindings may not be up-to-date. Please report any issues you encounter at \
                    https://www.github.com/Taaitaaiger/jlrs/issues"
            );
        }
    }

    if minor > nightly_minor_version {
        if warn {
            println!(
                "cargo::warning=Detected too recent version of Julia {major}.{minor}.{patch}, \
                    assuming compatibility with 1.{nightly_minor_version}. Please report any issues you \
                    encounter at https://www.github.com/Taaitaaiger/jlrs/issues"
            );
        }

        emit_version_metadata(
            major,
            nightly_minor_version,
            0,
            min_minor_version,
            max_minor_version,
            nightly_minor_version,
            false,
        );
    } else if minor == nightly_minor_version && warn {
        println!(
            "cargo::warning=Detected nightly version of Julia {major}.{minor}.{patch}. \
                    Please report any issues you encounter at \
                    https://www.github.com/Taaitaaiger/jlrs/issues"
        );

        emit_version_metadata(
            major,
            nightly_minor_version,
            patch,
            min_minor_version,
            max_minor_version,
            nightly_minor_version,
            false,
        );
    } else if minor > max_minor_version {
        if warn {
            println!(
                "cargo::warning=Detected unsupported version of Julia {major}.{minor}.{patch}, \
                    assuming compatibility with 1.{max_minor_version}. Please report any issues you \
                    encounter at https://www.github.com/Taaitaaiger/jlrs/issues"
            );
        }

        emit_version_metadata(
            major,
            max_minor_version,
            0,
            min_minor_version,
            max_minor_version,
            nightly_minor_version,
            false,
        );
    } else if minor < min_minor_version {
        panic!(
            "Detected unsupported version of Julia {major}.{minor}.{patch}. Minimum supported version is 1.{min_minor_version}"
        );
    } else {
        println!("cargo::rustc-cfg=julia_{major}_{minor}");
    }
}
