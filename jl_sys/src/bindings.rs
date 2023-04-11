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

bindings_for!(
    bindings_1_6_64,
    "julia-1-6",
    "64"
);
bindings_for!(
    bindings_1_6_32,
    "julia-1-6",
    "32"
);

bindings_for!(
    bindings_1_7_64,
    "julia-1-7",
    "64"
);
bindings_for!(
    bindings_1_7_32,
    "julia-1-7",
    "32"
);

bindings_for!(
    bindings_1_8_64,
    "julia-1-8",
    "64"
);
bindings_for!(
    bindings_1_8_32,
    "julia-1-8",
    "32"
);

bindings_for!(
    bindings_1_9_64,
    "julia-1-9",
    "64"
);
bindings_for!(
    bindings_1_9_32,
    "julia-1-9",
    "32"
);

bindings_for!(
    bindings_nightly_64,
    "julia-1-10",
    "64"
);
