use find_julia::{JuliaDir, enable_version_cfgs};
use jlrs_compat::{MAX_MINOR_VERSION, MIN_MINOR_VERSION};

fn main() {
    unsafe {
        enable_version_cfgs(MIN_MINOR_VERSION, MAX_MINOR_VERSION);
        JuliaDir::from_detected()
            .expect("Julia not detected by jl-sys")
            .version()
            .emit_metadata_unchecked();
    }
}
