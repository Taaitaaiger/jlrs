use std::env;

use find_julia::{JuliaDir, Version, enable_version_cfgs};
use jlrs_compat::{DOCS_MINOR_VERSION, MAJOR_VERSION, MIN_MINOR_VERSION, NIGHTLY_MINOR_VERSION};

fn main() {
    enable_version_cfgs(MIN_MINOR_VERSION, NIGHTLY_MINOR_VERSION);

    if building_docs() {
        let version = Version::new(MAJOR_VERSION, DOCS_MINOR_VERSION, 0, false);
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
