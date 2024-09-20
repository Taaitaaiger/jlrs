//! Reexports structs and traits you're likely to need.

#[cfg(feature = "async")]
pub use async_trait::async_trait;
#[cfg(feature = "ccall")]
pub use jlrs_macros::julia_module;
pub use jlrs_macros::{encode_as_constant_bytes, julia_version};
#[cfg(feature = "jlrs-derive")]
pub use jlrs_macros::{
    CCallArg, CCallReturn, ConstructType, Enum, HasLayout, IntoJulia, IsBits, Typecheck, Unbox,
    ValidField, ValidLayout,
};

#[cfg(any(feature = "local-rt", feature = "async-rt", feature = "ccall"))]
pub use crate::memory::stack_frame::StackFrame;
#[cfg(any(feature = "async-rt", feature = "local-rt", feature = "multi-rt"))]
pub use crate::runtime::builder::Builder;
#[cfg(feature = "tokio-rt")]
pub use crate::runtime::executor::tokio_exec::*;
#[cfg(feature = "ccall")]
pub use crate::runtime::handle::ccall::CCall;
#[cfg(feature = "local-rt")]
pub use crate::runtime::sync_rt::{Julia, PendingJulia};
#[cfg(feature = "async")]
pub use crate::{
    async_util::task::{AsyncTask, PersistentTask},
    call::CallAsync,
    memory::target::frame::AsyncGcFrame,
};
pub use crate::{
    call::{Call, ProvideKeywords},
    convert::into_jlrs_result::IntoJlrsResult,
    data::{
        layout::{bool::Bool, char::Char, nothing::Nothing, tuple::*},
        managed::{
            array::{
                data::accessor::{Accessor, AccessorMut, AccessorMut1D},
                Array, ArrayData, ArrayRef, ArrayResult, ConstructTypedArray, Matrix, MatrixData,
                MatrixRef, MatrixResult, RankedArray, RankedArrayData, RankedArrayRef,
                RankedArrayResult, TypedArray, TypedArrayData, TypedArrayRef, TypedArrayResult,
                TypedMatrix, TypedMatrixData, TypedMatrixRef, TypedMatrixResult, TypedRankedArray,
                TypedRankedArrayData, TypedRankedArrayRef, TypedRankedArrayResult, TypedVector,
                TypedVectorData, TypedVectorRef, TypedVectorResult, Vector, VectorAny,
                VectorAnyData, VectorAnyRef, VectorAnyResult, VectorData, VectorRef, VectorResult,
            },
            datatype::{DataType, DataTypeData, DataTypeRef, DataTypeResult},
            module::{Module, ModuleData, ModuleRef, ModuleResult},
            string::{JuliaString, StringData, StringRef, StringResult},
            symbol::Symbol,
            value::Value,
            value::ValueData,
            value::ValueRef,
            value::ValueResult,
            /* Ref, */ Managed, ManagedRef,
        },
    },
    define_fast_array_key, define_fast_key,
    error::JlrsResult,
    memory::{
        scope::{LocalScope, Scope},
        target::{Target, TargetType},
    },
    named_tuple,
    runtime::handle::with_stack::WithStack,
};
#[cfg(feature = "async-rt")]
pub use crate::{runtime::builder::AsyncBuilder, runtime::handle::async_handle::AsyncHandle};
