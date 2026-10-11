use julia_metadata::{emit_detected_version_metadata, enable_version_cfgs, lib_dir};

fn main() {
    enable_version_cfgs();
    emit_detected_version_metadata(false);

    let lib_dir = lib_dir();
    println!("cargo::rustc-link-search={}", lib_dir.display());
    println!("cargo::rustc-link-search={}/julia", lib_dir.display());

    #[cfg(target_os = "macos")]
    {
        let lib_dir = julia_metadata::lib_dir();
        println!("cargo::rustc-link-arg=-Wl,-rpath,{}", lib_dir.display());
        println!(
            "cargo::rustc-link-arg=-Wl,-rpath,{}/julia",
            lib_dir.display()
        );
    }
}
