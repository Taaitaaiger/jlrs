#![cfg(feature = "sync-rt")]
use jlrs::runtime::builder::RuntimeBuilder;

#[test]
fn init_with_image() {
    if let Ok(julia_dir) = std::env::var("JULIA_DIR") {
        let bindir = format!("{}/bin", julia_dir);
        let image_path = format!("{}/lib/julia/system.so", julia_dir);

        unsafe {
            assert!(RuntimeBuilder::new()
                .image(bindir, image_path)
                .start()
                .is_err())
        }
    } else {
        println!("Skipping image test because JULIA_DIR environment variable is not set.");
    }
}
