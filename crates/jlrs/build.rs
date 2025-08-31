use std::env;

use find_julia::{JuliaDir, Version, enable_version_cfgs};
use jlrs_compat::{
    MAX_MINOR_VERSION, MIN_MINOR_VERSION, STABLE_MAJOR_VERSION, STABLE_MINOR_VERSION,
};

fn main() {
    enable_version_cfgs(MIN_MINOR_VERSION, MAX_MINOR_VERSION);

    if building_docs() {
        let version = Version::new(STABLE_MAJOR_VERSION, STABLE_MINOR_VERSION, 0, false);
        version.emit_metadata_unchecked();
        return;
    }

    JuliaDir::from_detected()
        .expect("Julia not detected by jl-sys")
        .version()
        .emit_metadata_unchecked();
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
