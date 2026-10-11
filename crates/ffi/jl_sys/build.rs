use julia_metadata::{emit_metadata, enable_version_cfgs, link_julia};

fn main() {
    println!("cargo::rerun-if-env-changed=PATH");
    println!("cargo::rerun-if-env-changed=LD_LIBRARY_PATH");
    println!("cargo::rerun-if-env-changed=DYLD_LIBRARY_PATH");
    println!("cargo::rerun-if-env-changed=JLRS_JULIA_DIR");

    enable_version_cfgs();
    emit_metadata();

    #[cfg(target_os = "macos")]
    {
        let lib_dir = julia_metadata::lib_dir();
        println!("cargo::rustc-link-arg=-Wl,-rpath,{}", lib_dir.display());
        println!(
            "cargo::rustc-link-arg=-Wl,-rpath,{}/julia",
            lib_dir.display()
        );
    }

    link_julia();
}
