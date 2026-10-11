fn main() {
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
