use std::path::PathBuf;

use jlrs::prelude::*;

fn main() {
    let mut julia = Builder::new().start_local().expect("Could not init Julia");

    // Include some custom code defined in MyModule.jl.
    // This is safe because the included code doesn't do any strange things.
    unsafe {
        let path = PathBuf::from("MyModule.jl");
        if path.exists() {
            julia.include(path).expect("Could not include file");
        } else {
            julia
                .include("examples/MyModule.jl")
                .expect("Could not include file");
        }
    }

    // Create a scope, the closure provided to this method can use a `GcFrame` to ensure Julia
    // data is not cleaned up by the GC while it's in use.
    let result = julia
        .with_stack(|mut stack| {
            stack.scope(|mut frame| {
                let dim = Value::new(&mut frame, 4isize);
                let iters = Value::new(&mut frame, 1_000_000isize);

                unsafe {
                    Module::main(&frame)
                        // the submodule doesn't have to be rooted because it's never reloaded.
                        .submodule(&frame, "MyModule")?
                        .as_managed()
                        // the same holds true for the function: the module is never reloaded so it's
                        // globally rooted
                        .global(&frame, "complexfunc")?
                        .as_managed()
                        // Call the function with the two arguments it takes
                        .call(&mut frame, [dim, iters])?
                        // The function that was called returns a `Float64`, which can be unboxed as `f64`
                        .unbox::<f64>()
                }
            })
        })
        .expect("Result is an error");

    println!("Result: {}", result);
}
