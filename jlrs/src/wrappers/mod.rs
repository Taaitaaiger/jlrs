//! Wrapper types
//!
//! Working with the Julia C API involves many pointers. Many of these pointers point to data
//! owned by the garbage collector that have to remain reachable while they're in use to
//! prevent the data from being freed, see the [`memory`] module for more information. Rather than
//! working with these pointers directly, jlrs provides wrapper types with lifetimes to ensure
//! this data can only be used while it can be reached by the garbage collector.
//!
//! The most important of these wrappers is [`Value`], which is essentially a pointer to some
//! opaque Julia data. When a Julia function is called from jlrs, its arguments must be
//! [`Value`]s and it returns one, too. Although a [`Value`] is opaque, we can figure out what it
//! contains because all [`Value`]s are guaranteed to be preceded by a header which contains their
//! [`DataType`], which is also a wrapper type. Two other important wrapper types are [`Array`],
//! which is the type that backs n-dimensional Julia arrays, and [`Module`] which provides access
//! to Julia modules and their contents.
//!
//! All these wrapper types wrap pointer types. When the corresponding Julia type is used in a
//! struct, it's stored as a pointer. All pointer types are valid [`Value`]s because they retain
//! their header. The wrappers for these types, and other builtin pointer types, can be found in
//! the [`ptr`] submodule.
//!
//! When you use the wrappers provided by jlrs it's guaranteed that the garbage collector can
//! reach them while they can be used from Rust. This is ensured because jlrs generally roots
//! values when they're created before providing the wrapper. Most of the wrapper types have
//! pointer fields themselves, and while these pointer fields can be used as [`Value`]s there's
//! one major issue: mutability. Due to mutability it's possible to change what data a pointer
//! field points to, and this can cause the previous value to become unreachable. As a result,
//! it can't be guaranteed that the data won't be freed while using it. Due to this issue, jlrs
//! provides [`Ref`]. Unlike the wrapper types, a [`Ref`] doesn't guarantee that it points to data
//! that is still reachable.
//!
//! Finally, there are several wrappers for types that are stored inline in Julia structs rather
//! than as pointers. These include [`Char`] and [`Bool`], but also tuples of various sizes and
//! bits-unions. These can be found in the [`inline`] submodule.
//!
//! [`Value`]: crate::wrappers::ptr::value::Value
//! [`Array`]: crate::wrappers::ptr::array::Array
//! [`DataType`]: crate::wrappers::ptr::datatype::DataType
//! [`Module`]: crate::wrappers::ptr::module::Module
//! [`Ref`]: crate::wrappers::ptr::Ref
//! [`Char`]: crate::wrappers::inline::char::Char
//! [`Bool`]: crate::wrappers::inline::bool::Bool
//! [`memory`]: crate::memory

pub mod inline;
pub mod ptr;
