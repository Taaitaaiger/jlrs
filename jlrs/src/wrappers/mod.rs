//! Wrapper types for Julia data
//!
//! Whenever the C API returns data owned by the garbage collector it's returned as a pointer.
//! jlrs provides several types that wrap these pointers, relevant functionality from the Julia C
//! API is available through methods and traits that these types implement. Examples include
//! [`Value`], [`Array`], and [`Module`]. A pointer wrapper can always be converted to a `Value`,
//! all of them have a lifetime which ensures the data can't escape the scope it has been rooted
//! in.
//!
//! In addition to pointer wrappers there are inline wrappers which provide a compatible layout
//! for the contents of Julia data with some specific type. These types can be converted to a
//! `Value` with [`Value::new`], and back to Rust with [`Value::unbox`]. Examples of inline
//! wrappers are primitive types like `u8`, the custom [`Bool`] and [`Char`] types, and generic
//! tuples that can have up to 32 elements which are available in the [`tuple`] module. Inline
//! wrappers for other types can be generated with the JlrsReflect.jl package.
//!
//! [`Value`]: crate::wrappers::ptr::value::Value
//! [`Value::new`]: crate::wrappers::ptr::value::Value::new
//! [`Value::unbox`]: crate::wrappers::ptr::value::Value::unbox
//! [`Array`]: crate::wrappers::ptr::array::Array
//! [`Module`]: crate::wrappers::ptr::module::Module
//! [`Ref`]: crate::wrappers::ptr::Ref
//! [`Char`]: crate::wrappers::inline::char::Char
//! [`Bool`]: crate::wrappers::inline::bool::Bool
//! [`tuple`]: crate::wrappers::inline::tuple

pub mod foreign;
pub mod inline;
pub mod ptr;
pub mod tracked;
