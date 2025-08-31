use find_julia::{enable_version_cfgs, JuliaDir, Version};
use jlrs_compat::*;

fn main() {
    println!("cargo::rerun-if-env-changed=PATH");
    println!("cargo::rerun-if-env-changed=LD_LIBRARY_PATH");
    println!("cargo::rerun-if-env-changed=DYLD_LIBRARY_PATH");
    println!("cargo::rerun-if-env-changed=JLRS_JULIA_DIR");

    enable_version_cfgs(MIN_MINOR_VERSION, MAX_MINOR_VERSION);

    if building_docs() {
        // Safety: The Julia version only affects what symbols are documented
        unsafe {
            let version = Version::new(
                STABLE_MAJOR_VERSION,
                STABLE_MINOR_VERSION,
                STABLE_PATCH_VERSION,
                false,
            );
            version.emit_metadata_unchecked();
        }
    } else {
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
