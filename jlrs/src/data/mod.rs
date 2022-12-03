//! Managed types for Julia data
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
//! [`Value`]: crate::data::managed::value::Value
//! [`Value::new`]: crate::data::managed::value::Value::new
//! [`Value::unbox`]: crate::data::managed::value::Value::unbox
//! [`Array`]: crate::data::managed::array::Array
//! [`Module`]: crate::data::managed::module::Module
//! [`Ref`]: crate::data::managed::Ref
//! [`Char`]: crate::data::layout::char::Char
//! [`Bool`]: crate::data::layout::bool::Bool
//! [`tuple`]: crate::data::layout::tuple

pub mod layout;
pub mod managed;
