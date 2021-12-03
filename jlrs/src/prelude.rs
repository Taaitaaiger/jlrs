//! Reexports structs and traits you're likely to need.

pub use crate::{
    convert::{as_unrooted::AsUnrooted, into_jlrs_result::IntoJlrsResult},
    error::JlrsResult,
    layout::typecheck::Nothing,
    memory::{
        frame::{Frame, GcFrame},
        global::Global,
        scope::{Scope, ScopeExt},
    },
    named_tuple,
    wrappers::{
        inline::{bool::Bool, char::Char, tuple::*},
        ptr::{
            array::{dimensions::Dims, Array, TypedArray},
            call::{Call, CallExt},
            datatype::DataType,
            module::Module,
            string::JuliaString,
            symbol::Symbol,
            value::Value,
            Wrapper,
        },
    },
};

#[cfg(feature = "sync-rt")]
pub use crate::julia::Julia;

#[cfg(feature = "ccall")]
pub use crate::{ccall::CCall, memory::frame::NullFrame};

#[cfg(feature = "jlrs-derive")]
pub use jlrs_derive::*;

#[cfg(any(feature = "tokio-rt", feature = "async-std-rt"))]
pub use crate::extensions::multitask::AsyncJulia;
#[cfg(feature = "async")]
pub use crate::extensions::multitask::{
    async_frame::AsyncGcFrame,
    async_task::{AsyncTask, PersistentTask},
    call_async::CallAsync,
    yield_task,
};
#[cfg(feature = "async")]
pub use async_trait::async_trait;

#[cfg(feature = "pyplot")]
pub use crate::extensions::pyplot::{AccessPlotsModule, PyPlot};
