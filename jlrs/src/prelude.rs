//! Reexports structs and traits you're likely to need.

pub use crate::{
    convert::into_jlrs_result::IntoJlrsResult,
    error::{JlrsResult, JuliaResult},
    layout::typecheck::Nothing,
    memory::{
        frame::{Frame, GcFrame, NullFrame},
        global::Global,
        scope::{Scope, ScopeExt},
    },
    named_tuple,
    wrappers::{
        inline::tuple::*,
        ptr::{
            array::{dimensions::Dims, Array, TypedArray},
            call::{Call, CallExt, UnsafeCall, UnsafeCallExt},
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
pub use jlrs_derive::{IntoJulia, Typecheck, Unbox, ValidLayout};

#[cfg(feature = "async")]
pub use crate::extensions::multitask::{
    as_unrooted::AsUnrooted, async_frame::AsyncGcFrame, async_task::AsyncTask,
    call_async::CallAsync, AsyncJulia,
};
#[cfg(feature = "async")]
pub use async_trait::async_trait;
