# Examples

This crate contains several examples that show how to use jlrs, both embedding and ccall examples are available. The embedding examples can be called with `cargo run --example $name`. The ccall examples, `ccall` and `ccall_throw_exceptions`, are libraries that can be built with `cargo build --example $name`. After building the ccall examples they can be called by adding '$REPO_ROOT/target/debug/examples' to the `LD_LIBRARY_PATH` environment variable on Linux or the `PATH` environment variable on Windows and calling `julia $name.jl`.
