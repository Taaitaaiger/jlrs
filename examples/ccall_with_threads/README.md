This example shows how a long-running function in Rust can be run on a separate thread without blocking Julia in combination with `ccall`.

The library type is set in `Cargo.toml` to `cdylib`, `dylib` is also a valid choice. You can build the library by calling `cargo build`. You must ensure the prerequisites to use jlrs have been met, see the [readme of jlrs](https://github.com/Taaitaaiger/jlrs/blob/master/README.md) for more information. 

When the library has been built, you can run `callrust.jl` on Linux by calling `LD_LIBRARY_PATH=target/debug/:$LD_LIBRARY_PATH julia callrust.jl` and should see "Still running" be printed several times before exiting.
