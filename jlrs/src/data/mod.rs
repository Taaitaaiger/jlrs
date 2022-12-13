//! Create and use Julia data.
//!
//! Julia data is data that is managed by Julia and is eventually freed by its GC after it has
//! become unreachable. Types that represent such data, for example [`Value`], [`Array`], and
//! [`Module`], can be found in the [`managed`] module. In general, if some type of Julia data is
//! defined by the C API, a compatible type is available which provides access to relevant
//! functionality from the C API through the methods and traits it implements.
//!
//! If you want to unbox or otherwise access the content of Julia data, you'll have to provide the
//! layout of that data. The [`layout`] module provides valid layouts for many types of primitive
//! data and tuples, layouts for other types of Julia data can be generated with JlrsReflect.jl.
//! The [`ForeignType`] trait is also provided by this module, it can be used to move Rust data to
//! Julia and make the GC responsible for freeing it.
//!
//! [`Value`]: crate::data::managed::value::Value
//! [`Array`]: crate::data::managed::array::Array
//! [`Module`]: crate::data::managed::module::Module
//! [`Ref`]: crate::data::managed::Ref
//! [`ForeignType`]: crate::data::layout::foreign::ForeignType

pub mod layout;
pub mod managed;
