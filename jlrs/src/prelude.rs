//! Reexports structs and traits you're likely to need.

pub use crate::convert::into_jlrs_result::IntoJlrsResult;
pub use crate::error::{CallResult, JlrsError, JlrsResult};
pub use crate::layout::valid_layout::ValidLayout;
pub use crate::memory::frame::{GcFrame, NullFrame};
pub use crate::memory::global::Global;
pub use crate::memory::traits::{
    frame::Frame,
    scope::{Scope, ScopeExt},
};
pub use crate::value::array::{
    Array, ArrayData, CopiedArray, InlineArrayDataMut, TypedArray, ValueArrayDataMut,
};
pub use crate::value::datatype::DataType;
pub use crate::value::module::Module;
pub use crate::value::string::JuliaString;
pub use crate::value::symbol::Symbol;
pub use crate::value::traits::call::Call;
pub use crate::value::tuple::*;
pub use crate::value::type_var::TypeVar;
pub use crate::value::Value;
pub use crate::{named_tuple, CCall, Julia};

#[cfg(feature = "jlrs-derive")]
pub use jlrs_derive::{IntoJulia, JuliaStruct};

#[cfg(all(feature = "async", target_os = "linux"))]
pub use crate::memory::frame::AsyncGcFrame;
#[cfg(all(feature = "async", target_os = "linux"))]
pub use crate::memory::traits::as_unrooted::AsUnrooted;
#[cfg(all(feature = "async", target_os = "linux"))]
pub use crate::multitask::julia_task::JuliaTask;
#[cfg(all(feature = "async", target_os = "linux"))]
pub use crate::multitask::AsyncJulia;
#[cfg(all(feature = "async", target_os = "linux"))]
pub use async_trait::async_trait;
