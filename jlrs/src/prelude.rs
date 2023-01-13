//! Reexports structs and traits you're likely to need.

#[cfg(feature = "async")]
pub use async_trait::async_trait;
#[cfg(feature = "ccall")]
pub use jlrs_macros::julia_module;
pub use jlrs_macros::julia_version;
#[cfg(feature = "jlrs-derive")]
pub use jlrs_macros::{
    CCallArg, ConstructType, IntoJulia, Typecheck, Unbox, ValidField, ValidLayout,
};

#[cfg(feature = "ccall")]
pub use crate::ccall::CCall;
#[cfg(any(feature = "sync-rt", feature = "ccall"))]
pub use crate::memory::stack_frame::StackFrame;
#[cfg(feature = "pyplot")]
pub use crate::pyplot::{AccessPlotsModule, PyPlot};
#[cfg(feature = "async-std-rt")]
pub use crate::runtime::async_rt::async_std_rt::*;
#[cfg(feature = "tokio-rt")]
pub use crate::runtime::async_rt::tokio_rt::*;
#[cfg(any(feature = "async-rt", feature = "sync-rt"))]
pub use crate::runtime::builder::RuntimeBuilder;
#[cfg(feature = "sync-rt")]
pub use crate::runtime::sync_rt::{Julia, PendingJulia};
#[cfg(feature = "async-rt")]
pub use crate::runtime::{async_rt::AsyncJulia, builder::AsyncRuntimeBuilder};
#[cfg(feature = "async")]
pub use crate::{
    async_util::{
        affinity::{Affinity, DispatchAny, DispatchMain, DispatchWorker},
        task::{yield_task, AsyncTask, PersistentTask},
    },
    call::CallAsync,
    memory::target::frame::AsyncGcFrame,
};
pub use crate::{
    call::{Call, ProvideKeywords},
    convert::into_jlrs_result::IntoJlrsResult,
    data::{
        layout::{bool::Bool, char::Char, nothing::Nothing, tuple::*},
        managed::{
            array::ArrayRef,
            array::TypedArrayRef,
            array::{tracked::TrackArray, Array, TypedArray},
            datatype::DataType,
            datatype::DataTypeRef,
            module::Module,
            module::ModuleRef,
            string::JuliaString,
            string::StringRef,
            symbol::Symbol,
            value::Value,
            value::ValueRef,
            /* Ref, */ Managed,
        },
    },
    error::JlrsResult,
    memory::target::{target_type::TargetType, Target},
    named_tuple,
};
