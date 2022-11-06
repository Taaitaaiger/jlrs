use jlrs::prelude::*;
use std::path::PathBuf;

fn main() {
    // Julia must be initialized before it can be used.
    // This is safe because this we're not initializing Julia from another
    // thread and crate at the same time.
    let mut frame = StackFrame::new();
    let mut pending = unsafe { RuntimeBuilder::new().start().expect("Could not init Julia") };
    let mut julia = pending.instance(&mut frame);

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
        .scope(|mut frame| {
            let dim = Value::new(&mut frame, 4isize);
            let iters = Value::new(&mut frame, 1_000_000isize);

            unsafe {
                Module::main(&frame)
                    // the submodule doesn't have to be rooted because it's never reloaded.
                    .submodule(&frame, "MyModule")?
                    .wrapper_unchecked()
                    // the same holds true for the function: the module is never reloaded so it's
                    // globally rooted
                    .function(&frame, "complexfunc")?
                    .wrapper_unchecked()
                    // Call the function with the two arguments it takes
                    .call2(&mut frame, dim, iters)
                    // If you don't want to use the exception, it can be converted to a `JlrsError`
                    // In this case the error message will contain the message that calling
                    // `display` in Julia would show
                    .into_jlrs_result()?
                    // The function that was called returns a `Float64`, which can be unboxed as `f64`
                    .unbox::<f64>()
            }
        })
        .expect("Result is an error");

    println!("Result: {}", result);
}
