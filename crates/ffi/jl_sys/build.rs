use julia_metadata::{emit_metadata, enable_version_cfgs, link_julia};

fn main() {
    println!("cargo::rerun-if-env-changed=PATH");
    println!("cargo::rerun-if-env-changed=LD_LIBRARY_PATH");
    println!("cargo::rerun-if-env-changed=DYLD_LIBRARY_PATH");
    println!("cargo::rerun-if-env-changed=JLRS_JULIA_DIR");

    enable_version_cfgs();
    emit_metadata();

    link_julia();
}
