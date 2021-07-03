//! Reexports structs and traits you're likely to need.

pub use crate::{
    convert::{as_unrooted::AsUnrooted, into_jlrs_result::IntoJlrsResult},
    error::JlrsResult,
    layout::typecheck::Nothing,
    memory::{
        frame::{Frame, GcFrame, NullFrame},
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
    CCall, Julia,
};

#[cfg(feature = "jlrs-derive")]
pub use jlrs_derive::*;

#[cfg(feature = "async")]
pub use crate::extensions::multitask::{
    async_frame::AsyncGcFrame, async_task::AsyncTask, call_async::CallAsync, AsyncJulia,
};
#[cfg(feature = "async")]
pub use async_trait::async_trait;
