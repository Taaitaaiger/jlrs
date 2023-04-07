#[rustfmt::skip]
macro_rules! bindings_for {
    ($bindings:tt, $version:literal, $os:literal, $pointer_width:literal) => {
        #[cfg(all(
            not(feature = "use-bindgen"),
            feature = $version,
            target_os = $os,
            target_pointer_width = $pointer_width
        ))]
        mod $bindings;
        #[cfg(all(
            not(feature = "use-bindgen"),
            feature = $version,
            target_os = $os,
            target_pointer_width = $pointer_width
        ))]
        pub use $bindings::*;
    };
}

bindings_for!(
    bindings_1_6_x86_64_unknown_linux_gnu,
    "julia-1-6",
    "linux",
    "64"
);
bindings_for!(
    bindings_1_6_i686_unknown_linux_gnu,
    "julia-1-6",
    "linux",
    "32"
);
bindings_for!(
    bindings_1_6_x86_64_pc_windows_gnu,
    "julia-1-6",
    "windows",
    "64"
);
bindings_for!(
    bindings_1_6_x86_64_apple_darwin,
    "julia-1-6",
    "macos",
    "64"
);

bindings_for!(
    bindings_1_7_x86_64_unknown_linux_gnu,
    "julia-1-7",
    "linux",
    "64"
);

bindings_for!(
    bindings_1_7_i686_unknown_linux_gnu,
    "julia-1-7",
    "linux",
    "32"
);

bindings_for!(
    bindings_1_7_x86_64_pc_windows_gnu,
    "julia-1-7",
    "windows",
    "64"
);
bindings_for!(
    bindings_1_7_x86_64_apple_darwin,
    "julia-1-7",
    "macos",
    "64"
);

bindings_for!(
    bindings_1_8_x86_64_unknown_linux_gnu,
    "julia-1-8",
    "linux",
    "64"
);
bindings_for!(
    bindings_1_8_i686_unknown_linux_gnu,
    "julia-1-8",
    "linux",
    "32"
);
bindings_for!(
    bindings_1_8_x86_64_pc_windows_gnu,
    "julia-1-8",
    "windows",
    "64"
);
bindings_for!(
    bindings_1_8_x86_64_apple_darwin,
    "julia-1-8",
    "macos",
    "64"
);

bindings_for!(
    bindings_1_9_x86_64_unknown_linux_gnu,
    "julia-1-9",
    "linux",
    "64"
);
bindings_for!(
    bindings_1_9_i686_unknown_linux_gnu,
    "julia-1-9",
    "linux",
    "32"
);
bindings_for!(
    bindings_1_9_x86_64_pc_windows_gnu,
    "julia-1-9",
    "windows",
    "64"
);
bindings_for!(
    bindings_1_9_x86_64_apple_darwin,
    "julia-1-9",
    "macos",
    "64"
);

bindings_for!(
    bindings_nightly_x86_64_unknown_linux_gnu,
    "julia-1-10",
    "linux",
    "64"
);