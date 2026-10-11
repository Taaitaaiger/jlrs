use std::{env, path::PathBuf};

use find_julia::JuliaDir;

fn main() {
    let julia_dir = if building_docs() {
        JuliaDir::docs(jlrs_compat::MAJOR_VERSION, jlrs_compat::DOCS_MINOR_VERSION)
    } else {
        JuliaDir::find(windows_build(), debug_build())
            .expect("JLRS_JULIA_DIR is not set and no installed version of Julia can be found")
    };

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("version.rs");
    julia_dir
        .write_metadata_file(
            &out_path,
            jlrs_compat::MIN_MINOR_VERSION,
            jlrs_compat::MAX_MINOR_VERSION,
            jlrs_compat::NIGHTLY_MINOR_VERSION,
        )
        .expect("Unable to write version file");
}

fn debug_build() -> bool {
    #[cfg(feature = "debug")]
    return true;

    #[cfg(not(feature = "debug"))]
    return false;
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

fn windows_build() -> bool {
    #[cfg(any(target_os = "windows", feature = "windows"))]
    return true;

    #[cfg(not(any(target_os = "windows", feature = "windows")))]
    return false;
}
