use find_julia::{JuliaDir, Version, enable_version_cfgs};
use jlrs_compat::*;

fn main() {
    println!("cargo::rerun-if-env-changed=PATH");
    println!("cargo::rerun-if-env-changed=LD_LIBRARY_PATH");
    println!("cargo::rerun-if-env-changed=DYLD_LIBRARY_PATH");
    println!("cargo::rerun-if-env-changed=JLRS_JULIA_DIR");

    // Enable julia_1_x configs
    enable_version_cfgs(MIN_MINOR_VERSION, MAX_MINOR_VERSION);

    if building_docs() {
        // Don't link Julia when building the docs
        let version = Version::new(STABLE_MAJOR_VERSION, STABLE_MINOR_VERSION, 0, false);
        version.emit_metadata_unchecked();
    } else {
        // Detect active version of Julia, emit metadata, and link Julia.
        let julia_dir = JuliaDir::find()
            .expect("JLRS_JULIA_DIR is not set and no installed version of Julia can be found");

        julia_dir.emit_metadata(MIN_MINOR_VERSION, MAX_MINOR_VERSION);
        julia_dir.link();
    }
}

fn building_docs() -> bool {
    #[cfg(feature = "docs")]
    return true;

    #[cfg(not(feature = "docs"))]
    return false;
}
