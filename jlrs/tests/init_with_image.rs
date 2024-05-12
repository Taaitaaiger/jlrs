#![cfg(feature = "local-rt")]
mod tests {
    use jlrs::runtime::builder::Builder;

    #[test]
    fn init_with_image() {
        if let Ok(julia_dir) = std::env::var("JULIA_DIR") {
            let bindir = format!("{}/bin", julia_dir);
            #[cfg(target_os = "windows")]
            let image_path = format!("{}/lib/julia/sys.dll", julia_dir);
            #[cfg(target_os = "linux")]
            let image_path = format!("{}/lib/julia/sys.so", julia_dir);
            #[cfg(target_os = "macos")]
            let image_path = format!("{}/lib/julia/sys.dylib", julia_dir);

            unsafe {
                assert!(Builder::new()
                    .image(bindir, image_path)
                    .ok()
                    .unwrap()
                    .start_local()
                    .is_ok())
            }
        } else {
            println!("Skipping image test because JULIA_DIR environment variable is not set.");
        }
    }
}
