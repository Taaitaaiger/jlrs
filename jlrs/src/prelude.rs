//! Reexports structs and traits you're likely to need.

pub use crate::call::{Call, CallExt};
pub use crate::convert::into_jlrs_result::IntoJlrsResult;
pub use crate::error::{JlrsResult, JuliaResult};
pub use crate::layout::{typecheck::Nothing, valid_layout::ValidLayout};
pub use crate::memory::frame::{GcFrame, NullFrame};
pub use crate::memory::global::Global;
pub use crate::memory::traits::{
    frame::Frame,
    scope::{Scope, ScopeExt},
};
pub use crate::wrappers::inline::tuple::*;
pub use crate::wrappers::ptr::array::dimensions::Dims;
pub use crate::wrappers::ptr::array::{Array, TypedArray};
pub use crate::wrappers::ptr::datatype::DataType;
pub use crate::wrappers::ptr::module::Module;
pub use crate::wrappers::ptr::string::JuliaString;
pub use crate::wrappers::ptr::symbol::Symbol;
pub use crate::wrappers::ptr::type_var::TypeVar;
pub use crate::wrappers::ptr::value::Value;
pub use crate::wrappers::ptr::Wrapper;
pub use crate::{named_tuple, CCall, Julia};

#[cfg(feature = "jlrs-derive")]
pub use jlrs_derive::{IntoJulia, JuliaStruct};

#[cfg(feature = "async")]
pub use crate::extensions::multitask::{
    as_unrooted::AsUnrooted, async_frame::AsyncGcFrame, async_task::AsyncTask,
    call_async::CallAsync, AsyncJulia,
};
#[cfg(feature = "async")]
pub use async_trait::async_trait;
