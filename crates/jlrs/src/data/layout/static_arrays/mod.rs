//! Support for StaticArrays
//!
//! The [StaticArrays] package for Julia provides statically sized array types.
//!
//! The types provided in this module can be used in functions exported with the `julia_module`
//! macro. All types can be passed by reference, mutable array types can be passed by mutable
//! reference. Be aware that Rust's aliasing rules apply: never alias a mutable reference.
//!
//! [StaticArrays]: https://https://juliaarrays.github.io/StaticArrays.jl/stable/

pub mod dims;
pub mod m_array;
pub mod m_matrix;
pub mod m_vector;
pub mod s_array;
pub mod s_matrix;
pub mod s_vector;

#[cfg(all(
    feature = "static-arrays",
    any(feature = "local-rt", feature = "async-rt", feature = "multi-rt")
))]
pub(crate) unsafe fn init_static_arrays() {
    let using = c"using StaticArrays";
    unsafe {
        jl_sys::jl_eval_string(using.as_ptr());
    }
}
