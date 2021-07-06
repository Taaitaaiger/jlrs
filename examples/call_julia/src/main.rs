use jlrs::prelude::*;

fn main() {
    let mut julia = unsafe { Julia::init().expect("Could not init Julia") };

    julia
        .include("MyModule.jl")
        .expect("Could not include file");
    let v = julia
        .scope(|global, frame| {
            let dim = Value::new(&mut *frame, 4isize)?;
            let iters = Value::new(&mut *frame, 1_000_000isize)?;

            unsafe {
                Module::main(global)
                    // the submodule doesn't have to be rooted because it's never reloaded.
                    .submodule_ref("MyModule")?
                    .wrapper_unchecked()
                    // the same holds true for the function: the module is never reloaded so it's
                    // globally rooted
                    .function_ref("complexfunc")?
                    .wrapper_unchecked()
                    // Call the function with the two arguments it takes
                    .call2(&mut *frame, dim, iters)?
                    // If you don't want to use the exception, it can be converted to a `JlrsError`
                    // In this case the error message will contain the message that calling
                    // `display` in Julia would show
                    .into_jlrs_result()?
                    // The function that was called returns a `Float64`, which can be unboxed as `f64`
                    .unbox::<f64>()
            }
        })
        .expect("Result is an error");

    println!("Result: {}", v);
}
