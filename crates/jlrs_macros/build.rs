use std::{env, path::PathBuf};

use find_julia::{JuliaDir, enable_version_cfgs};
use jlrs_compat::{DOCS_MINOR_VERSION, MAJOR_VERSION, MIN_MINOR_VERSION, NIGHTLY_MINOR_VERSION};

fn main() {
    enable_version_cfgs(MIN_MINOR_VERSION, NIGHTLY_MINOR_VERSION);

    let julia_dir = if building_docs() {
        JuliaDir::docs(MAJOR_VERSION, DOCS_MINOR_VERSION)
    } else {
        JuliaDir::from_detected().expect("Julia not detected by jl-sys")
    };

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("version.rs");
    julia_dir
        .write_version_file(&out_path, MIN_MINOR_VERSION, NIGHTLY_MINOR_VERSION)
        .expect("Unable to write version file");
    julia_dir.version().emit_metadata_unchecked();
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
