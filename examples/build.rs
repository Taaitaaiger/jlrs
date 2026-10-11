use julia_metadata::lib_dir;

fn main() {
    let lib_dir = lib_dir();
    println!("cargo::rustc-link-search={}", lib_dir.display());
    println!("cargo::rustc-link-search={}/julia", lib_dir.display());

    println!("cargo::rustc-link-arg=-Wl,-rpath,{}", lib_dir.display());
    println!(
        "cargo::rustc-link-arg=-Wl,-rpath,{}/julia",
        lib_dir.display()
    );
}
