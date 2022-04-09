#![cfg(feature = "sync-rt")]
use jlrs::julia::Julia;

#[test]
fn init_with_image() {
    if let Ok(julia_dir) = std::env::var("JULIA_DIR") {
        let bindir = format!("{}/bin", julia_dir);
        #[cfg(target_os = "windows")]
        let image_path = format!("{}/lib/julia/sys.dll", julia_dir);
        #[cfg(target_os = "linux")]
        let image_path = format!("{}/lib/julia/sys.so", julia_dir);
        unsafe {
            assert!(Julia::init_with_image(&bindir, &image_path).is_ok());
            assert!(Julia::init_with_image(&bindir, &image_path).is_err());
        }
    } else {
        println!("Skipping image test because JULIA_DIR environment variable is not set.");
    }
}
