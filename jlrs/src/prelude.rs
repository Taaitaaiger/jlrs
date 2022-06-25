//! Reexports structs and traits you're likely to need.

#[cfg(feature = "ccall")]
pub use crate::ccall::CCall;
#[cfg(feature = "pyplot")]
pub use crate::pyplot::{AccessPlotsModule, PyPlot};
#[cfg(feature = "async-std-rt")]
pub use crate::runtime::async_rt::async_std_rt::*;
#[cfg(feature = "tokio-rt")]
pub use crate::runtime::async_rt::tokio_rt::*;
#[cfg(any(feature = "async-rt", feature = "sync-rt"))]
pub use crate::runtime::builder::RuntimeBuilder;
#[cfg(feature = "sync-rt")]
pub use crate::runtime::sync_rt::Julia;
#[cfg(feature = "async-rt")]
pub use crate::runtime::{async_rt::AsyncJulia, builder::AsyncRuntimeBuilder};
#[cfg(feature = "async")]
pub use crate::{
    async_util::task::{yield_task, AsyncTask, PersistentTask},
    call::CallAsync,
    memory::frame::AsyncGcFrame,
};
pub use crate::{
    call::{Call, ProvideKeywords},
    convert::into_jlrs_result::IntoJlrsResult,
    error::JlrsResult,
    memory::{
        frame::Frame,
        global::Global,
        scope::{PartialScope, Scope},
    },
    named_tuple,
    wrappers::{
        inline::{bool::Bool, char::Char, nothing::Nothing, tuple::*},
        ptr::{
            array::ArrayRef,
            array::TypedArrayRef,
            array::{Array, TypedArray},
            datatype::DataType,
            datatype::DataTypeRef,
            module::Module,
            module::ModuleRef,
            string::JuliaString,
            string::StringRef,
            symbol::Symbol,
            value::Value,
            value::ValueRef,
            /*Ref,*/ Wrapper,
        },
    },
};
#[cfg(feature = "async")]
pub use async_trait::async_trait;
#[cfg(feature = "jlrs-derive")]
pub use jlrs_derive::*;
