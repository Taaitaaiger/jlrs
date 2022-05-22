This example provides a few functions that can be called from Julia and a script that calls them.

The library type is set in `Cargo.toml` to `cdylib`, `dylib` is also a valid choice. You can build the library by calling `cargo build`. You must ensure the prerequisites to use jlrs have been met, see the [readme of jlrs](https://github.com/Taaitaaiger/jlrs/blob/master/README.md) for more information. 

When the library has been built, you can run `callrust.jl` on Linux by calling `LD_LIBRARY_PATH=target/debug/:$LD_LIBRARY_PATH julia callrust.jl` and should see the following output:

`1 + 2 = 3`
`Before increment: [1.0 2.0 3.0; 4.0 5.0 6.0; 7.0 8.0 9.0]`
`After increment: [2.0 3.0 4.0; 5.0 6.0 7.0; 8.0 9.0 10.0]`
