//! Wrapper types for Julia data
//!
//! There are two major "classes" of Julia types, pointer and inline types. This distinction is due
//! to how data of these types is stored when they're used as field types in Julia structs. The
//! fields of inline types are stored inline, while a pointer type is stored as a pointer.
//!
//! While most kinds of data in Julia are defined purely in Julia, some are defined in C. This
//! includes types like `Module`, `DataType`, and `Array`. They're mostly pointer types. Rather
//! than dealing with the raw pointers, jlrs provides wrappers for these builtin types, you can
//! find them, and more information about them in general, in the [`ptr`] module. The most
//! important of these wrappers is [`Value`], which is essentially the `Any` of Julia.
//!
//! In addition to these pointer wrappers, jlrs also provides a many inline wrappers. Examples
//! include the primitive types like `UInt8` and `Float64`, most of them are simply the
//! appropriate primitive type in Rust; `u8` and `f64` will work for these two types, but `Bool`
//! and `Char` have custom wrappers: [`Bool`] and [`Char`]. Tuples of up to 32 elements are
//! available in the [`tuple`] module.
//!
//! [`Value`]: crate::wrappers::ptr::value::Value
//! [`Char`]: crate::wrappers::inline::char::Char
//! [`Bool`]: crate::wrappers::inline::bool::Bool
//! [`tuple`]: crate::wrappers::inline::tuple

pub mod inline;
pub mod ptr;
