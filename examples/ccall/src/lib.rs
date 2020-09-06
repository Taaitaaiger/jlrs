use jlrs::prelude::*;

// NB: This crate is called `ccall`, so the library is called `libccall`. The functions are 
// annotated with `no_mangle` to prevent name mangling and `extern "C"` to make them callable
// with the C ABI.

// Add two 32-bit signed integers, it can be called from Julia with:
// `ccall((:add, "libccall"), Int32, (Int32, Int32), a, b)` where `a` and `b` are `Int32`s.
#[no_mangle]
pub unsafe extern "C" fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Increment every element in an array of `f64`s, it can be called from Julia with:
// `ccall((:incr_array, "libccall"), Cvoid, (Array{Float64},), arr)`  where `arr` is an 
// `Array{Float64}`.
#[no_mangle]
pub unsafe extern "C" fn incr_array(a: TypedArray<f64>) {
    // We want to mutably borrow the array data, but don't need to protect any new values.
    let mut ccall = CCall::new(0);
    ccall
        .null_frame(|frame| {
            let mut data = a.inline_data_mut(frame)?;

            for x in data.as_mut_slice() {
                *x += 1.0;
            }

            Ok(())
        })
        .ok();
}
