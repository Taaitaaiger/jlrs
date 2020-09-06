This example provides a few functions that can be called from Julia and a script that calls them.

The library type is set in `Cargo.toml` to `cdylib`, `dylib` is also a valid choice. In order to build the library, run `cargo build`. Because this example uses jlrs, the `JULIA_DIR` environment variable (and in the case of Windows, `CYGWIN_DIR`) must be set, and the library must be available on the library search path.

When the library has been built, you can execute `callrust.jl` with `LD_LIBRARY_PATH=target/debug/ julia callrust.jl` and should see the following output:

`1 + 2 = 3`
`Before increment: [1.0 2.0 3.0; 4.0 5.0 6.0; 7.0 8.0 9.0]`
`After increment: [2.0 3.0 4.0; 5.0 6.0 7.0; 8.0 9.0 10.0]`
