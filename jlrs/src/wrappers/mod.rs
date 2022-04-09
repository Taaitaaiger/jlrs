//! Wrapper types for Julia data
//!
//! Whenever the C API returns data owned by the garbage collector it's returned as a pointer. In
//! order to avoid dealing with these raw pointers, jlrs provides several pointer wrapper types.
//! Examples include [`Value`], [`Array`], and [`Module`]. A pointer wrapper can always be
//! converted to a `Value`. All pointer wrappers have a lifetime which ensures the data can't
//! escape the scope it has been rooted in.
//!
//! Methods in jlrs that return Julia data usually come in two flavors: the result is rooted, in
//! this case a pointer wrapper is returned, or it's left unrooted. In the latter case a [`Ref`]
//! is returned instead. It's always unsafe to convert a `Ref` to a pointer wrapper because jlrs
//! can't guarantee that the data hasn't been freed yet.
//!
//! In addition to pointer wrappers there are inline wrappers which provide a layout for several
//! Julia types. This includes primitive types like `u8`, custom [`Bool`] and [`Char`] types, and
//! generic tuples that can have up to 32 elements are available in the [`tuple`] module. Inline
//! wrappers for other types can be generated with the JlrsReflect package.
//!
//! [`Value`]: crate::wrappers::ptr::value::Value
//! [`Array`]: crate::wrappers::ptr::array::Array
//! [`Module`]: crate::wrappers::ptr::module::Module
//! [`Ref`]: crate::wrappers::ptr::Ref
//! [`Char`]: crate::wrappers::inline::char::Char
//! [`Bool`]: crate::wrappers::inline::bool::Bool
//! [`tuple`]: crate::wrappers::inline::tuple

pub mod inline;
pub mod ptr;
