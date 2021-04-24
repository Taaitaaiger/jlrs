This example shows how a long-running function in Rust can be run on a separate thread without blocking Julia in combination with `ccall`.

The library type is set in `Cargo.toml` to `cdylib`, `dylib` is also a valid choice. In order to build the library, run `cargo build`. Because this example uses jlrs, the `JULIA_DIR` environment variable must be set, and the library must be available on the library search path.

When the library has been built, you can execute `callrust.jl` with `LD_LIBRARY_PATH=target/debug/ julia callrust.jl` and should see "Still running" be printed several times before exiting.
