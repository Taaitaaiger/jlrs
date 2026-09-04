mod util;
#[cfg(all(feature = "local-rt", feature = "static-arrays"))]
mod tests {
    use jlrs::prelude::*;

    use super::util::JULIA;

    fn use_static_arrays() {
        JULIA.with(|j| unsafe {
            j.borrow().using("StaticArrays").unwrap();
        });
    }

    fn include_error() {
        JULIA.with(|j| unsafe {
            let jlrs = j.borrow();
            assert!(jlrs.include("Cargo.toml").is_err());
        });
    }

    #[test]
    fn runtime_test() {
        use_static_arrays();

        //
        include_error();
    }
}
