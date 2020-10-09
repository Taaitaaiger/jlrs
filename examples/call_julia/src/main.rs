use jlrs::prelude::*;

fn main() {
    let mut julia = unsafe {
        Julia::init(32).expect("Could not init Julia") 
    };

    julia.include("MyModule.jl").expect("Could not include file");
    let v = julia.dynamic_frame(|global, frame| {
        let dim = Value::new(frame, 4isize)?;
        let iters = Value::new(frame, 1_000_000isize)?;

        Module::main(global)
            .submodule("MyModule")?
            .function("complexfunc")?
            .call2(frame, dim, iters)?
            .expect("MyModule.complexfunc threw an error")
            .cast::<f64>()
    }).expect("Result is an error");

    println!("Result: {}", v);
}
