#![cfg(feature = "sync-rt")]
use jlrs::{runtime::builder::RuntimeBuilder, memory::context::ContextFrame};

#[test]
fn init_with_image() {
    if let Ok(julia_dir) = std::env::var("JULIA_DIR") {
        let bindir = format!("{}/bin2", julia_dir);
        let image_path = format!("{}/lib/julia/sys.so", julia_dir);
        let base = ContextFrame::new();

        unsafe {
            assert!(RuntimeBuilder::new()
                .image(bindir, image_path)
                .start(&base)
                .is_err())
        }
    } else {
        println!("Skipping image test because JULIA_DIR environment variable is not set.");
    }
}
