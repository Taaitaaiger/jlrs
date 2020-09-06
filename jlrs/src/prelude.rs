//! Reexports structs and traits you're likely to need.

pub use crate::error::{JlrsError, JlrsResult};
pub use crate::traits::{Frame, ValidLayout};
pub use crate::value::array::{
    Array, ArrayData, CopiedArray, InlineArrayDataMut, TypedArray, ValueArrayDataMut,
};
pub use crate::value::datatype::DataType;
pub use crate::value::module::Module;
pub use crate::value::string::JuliaString;
pub use crate::value::symbol::Symbol;
pub use crate::value::tuple::*;
pub use crate::value::{Value, Values};
pub use crate::{CCall, Julia};
pub use crate::global::Global;

#[cfg(feature = "jlrs-derive")]
pub use jlrs_derive::{IntoJulia, JuliaStruct};

#[cfg(feature = "async")]
pub use crate::multitask::*;
#[cfg(feature = "async")]
pub use async_trait::async_trait;
