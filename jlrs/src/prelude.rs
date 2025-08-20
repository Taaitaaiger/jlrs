//! Reexports structs and traits you're likely to need.

#[cfg(feature = "ccall")]
pub use jlrs_macros::julia_module;
pub use jlrs_macros::julia_version;
#[cfg(feature = "jlrs-derive")]
pub use jlrs_macros::{
    CCallArg, CCallReturn, ConstructType, Enum, ForeignType, HasLayout, IntoJulia, IsBits,
    OpaqueType, Typecheck, Unbox, ValidField, ValidLayout,
};

#[cfg(feature = "jlrs-derive")]
pub use crate::data::types::foreign_type::ForeignType as _;
#[cfg(feature = "async-rt")]
pub use crate::runtime::builder::AsyncBuilder;
#[cfg(any(feature = "async-rt", feature = "local-rt", feature = "multi-rt"))]
pub use crate::runtime::builder::Builder;
#[cfg(feature = "tokio-rt")]
pub use crate::runtime::executor::tokio_exec::*;
#[cfg(feature = "async")]
pub use crate::{
    async_util::task::{AsyncTask, PersistentTask},
    call::CallAsync,
    memory::{scope::AsyncScope, target::frame::AsyncGcFrame},
};
pub use crate::{
    call::{Call, ProvideKeywords},
    data::{
        layout::{bool::Bool, char::Char, nothing::Nothing},
        managed::{
            array::{
                data::accessor::{Accessor, AccessorMut, AccessorMut1D},
                Array, ArrayData, ArrayResult, ConstructTypedArray, Matrix, MatrixData,
                MatrixResult, RankedArray, RankedArrayData, RankedArrayResult, TypedArray,
                TypedArrayData, TypedArrayResult, TypedMatrix, TypedMatrixData, TypedMatrixResult,
                TypedRankedArray, TypedRankedArrayData, TypedRankedArrayResult, TypedVector,
                TypedVectorData, TypedVectorResult, Vector, VectorAny, VectorAnyData,
                VectorAnyResult, VectorData, VectorResult, WeakArray, WeakMatrix, WeakRankedArray,
                WeakTypedArray, WeakTypedMatrix, WeakTypedRankedArray, WeakTypedVector, WeakVector,
                WeakVectorAny,
            },
            datatype::{DataType, DataTypeData, DataTypeResult, WeakDataType},
            module::{Module, ModuleData, ModuleResult, WeakModule},
            string::{JuliaString, StringData, StringResult, WeakString},
            symbol::Symbol,
            value::{Value, ValueData, ValueResult, WeakValue},
            Managed, ManagedWeak,
        },
    },
    define_fast_array_key, define_fast_key,
    error::JlrsResult,
    memory::{
        scope::{LocalScope, LocalScopeExt, Scope},
        target::{Target, TargetType},
    },
    named_tuple,
    runtime::{handle::with_stack::WithStack, Runtime},
};
