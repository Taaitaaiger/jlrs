//! Create and use Julia data.
//!
//! There are three major sides to Julia data: it's owned by the Julia GC, it has a type, and it
//! has a layout.
//!
//! Let's start with the fact that Julia data is owned by the GC. Whenever the C API returns Julia
//! data it's returned as a pointer. The data it points to is freed when the GC determines that it
//! has become unreachable. How this works is explained in more detail in the [`memory`] module.
//!
//! Rather than returning such pointers directly, jlrs returns instances of managed types defined
//! in the [`managed`] module. Each managed type contains a pointer to Julia data and has a
//! lifetime annotation to ensure it can only be used while it is guaranteed that the GC will
//! leave the referenced data alone. Most functionality provided by jlrs is available through
//! methods and traits implemented by managed types.
//!
//! Managed types don't necessarily correspond to a single `DataType` in Julia. For example, the
//! internal pointer of a [`Value`] can point to Julia data of any type, and that of an [`Array`]
//! can point to any Julia array regardless of the type of its elements and its rank. The
//! [`types`] module provides traits to check whether certain type properties hold true, to define
//! new types in Julia whose instances contain Rust data, and to construct arbitrarily complex
//! Julia type objects from compatible Rust types.
//!
//! Finally, there's the layout of Julia data. As stated previously, the internal pointer of a
//! `Value` can point to Julia data of any type, the layout depends on the `DataType` of that
//! data. The [`layout`] module provides matching layouts for several primitive and non-primitive
//! Julia types. While it might seem odd that the type and layout of Julia data are considerered
//! to be separate aspects, there's a good reason for this distinction: while a type dictates the
//! layout, a layout doesn't necessarily fully define the type, and typically we're more
//! interested in the layout than the type.
//!
//! It's not possible to define custom managed types, but it is possible to implement layouts and
//! type constructors. Rather than manually implementing such types and the relevant traits, you
//! should use the `reflect` function from the `JlrsCore.Reflect` module to do so automatically.
//!
//! [`memory`]: crate::memory
//! [`Value`]: crate::data::managed::value::Value
//! [`Array`]: crate::data::managed::array::Array

pub mod layout;
pub mod managed;
pub mod static_data;
pub mod types;
