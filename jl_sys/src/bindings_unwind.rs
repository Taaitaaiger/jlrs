#[rustfmt::skip]
macro_rules! bindings_for {
    ($bindings:tt, $version:literal, $pointer_width:literal) => {
        #[cfg(all(
            not(feature = "use-bindgen"),
            feature = $version,
            target_pointer_width = $pointer_width
        ))]
        mod $bindings;
        #[cfg(all(
            not(feature = "use-bindgen"),
            feature = $version,
            target_pointer_width = $pointer_width
        ))]
        pub use $bindings::*;
    };
}

bindings_for!(bindings_unwind_1_6_64, "julia-1-6", "64");
bindings_for!(bindings_unwind_1_6_32, "julia-1-6", "32");

bindings_for!(bindings_unwind_1_7_64, "julia-1-7", "64");
bindings_for!(bindings_unwind_1_7_32, "julia-1-7", "32");

bindings_for!(bindings_unwind_1_8_64, "julia-1-8", "64");
bindings_for!(bindings_unwind_1_8_32, "julia-1-8", "32");

bindings_for!(bindings_unwind_1_9_64, "julia-1-9", "64");
bindings_for!(bindings_unwind_1_9_32, "julia-1-9", "32");

bindings_for!(bindings_unwind_1_10_64, "julia-1-10", "64");
bindings_for!(bindings_unwind_1_10_32, "julia-1-10", "32");

bindings_for!(bindings_unwind_1_11_64, "julia-1-11", "64");
bindings_for!(bindings_unwind_1_11_32, "julia-1-11", "32");

#[cfg(target_os = "windows")]
mod bindings_unwind_ext_windows;

#[cfg(target_os = "windows")]
pub use bindings_unwind_ext_windows::*;
