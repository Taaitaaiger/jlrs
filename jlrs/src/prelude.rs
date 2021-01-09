//! Reexports structs and traits you're likely to need.

pub use crate::error::{CallResult, JlrsError, JlrsResult};
pub use crate::frame::{DynamicFrame, NullFrame, StaticFrame};
pub use crate::global::Global;
pub use crate::traits::{Call, Frame, ValidLayout};
pub use crate::value::array::{
    Array, ArrayData, CopiedArray, InlineArrayDataMut, TypedArray, ValueArrayDataMut,
};
pub use crate::value::datatype::DataType;
pub use crate::value::module::Module;
pub use crate::value::string::JuliaString;
pub use crate::value::symbol::Symbol;
pub use crate::value::tuple::*;
pub use crate::value::type_var::TypeVar;
pub use crate::value::Value;
pub use crate::{named_tuple, CCall, Julia};

#[cfg(feature = "jlrs-derive")]
pub use jlrs_derive::{IntoJulia, JuliaStruct};

#[cfg(all(feature = "async", target_os = "linux"))]
pub use crate::frame::DynamicAsyncFrame;
#[cfg(all(feature = "async", target_os = "linux"))]
pub use crate::multitask::*;
#[cfg(all(feature = "async", target_os = "linux"))]
pub use crate::traits::multitask::*;
#[cfg(all(feature = "async", target_os = "linux"))]
pub use async_trait::async_trait;
