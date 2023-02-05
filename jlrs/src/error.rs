//! Everything related to errors.

use std::error::Error as StdErr;

use thiserror::Error;

use crate::data::managed::{
    array::dimensions::Dimensions,
    value::{Value, ValueRef},
};

pub(crate) static CANNOT_DISPLAY_TYPE: &'static str = "<Cannot display type>";
pub(crate) static CANNOT_DISPLAY_VALUE: &'static str = "<Cannot display value>";

/// Alias that is used for most `Result`s in this crate.
pub type JlrsResult<T> = Result<T, Box<JlrsError>>;

/// Rooted Julia result or exception.
///
/// Some functions from the Julia C API can throw exceptions. Many methods provided by jlrs will
/// catch these exceptions and return a `JuliaResult`, the `Err` variant contains the exception.
pub type JuliaResult<'frame, 'data, V = Value<'frame, 'data>> = Result<V, Value<'frame, 'data>>;

/// Potentially unrooted Julia result or exception.
///
/// This type alias is similar to [`JuliaResult`], but can contain unrooted data.
pub type JuliaResultRef<'frame, 'data, V = ValueRef<'frame, 'data>> =
    Result<V, ValueRef<'frame, 'data>>;

/// Runtime errors.
#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("runtime can only be initialized once")]
    AlreadyInitialized,
    #[error("channel closed")]
    ChannelClosed,
    #[error("channel full")]
    ChannelFull,
}

/// IO errors.
#[derive(Debug, Error)]
pub enum IOError {
    #[error("path does not exist: {path}")]
    NotFound { path: String },
}

/// Type errors.
#[derive(Debug, Error)]
pub enum TypeError {
    #[error("expected a Function, {name} is a {ty}")]
    NotAFunction { name: String, ty: String },
    #[error("expected a NamedTuple, got a {ty}")]
    NotANamedTuple { ty: String },
    #[error("expected a Module, {name} is a {ty}")]
    NotAModule { name: String, ty: String },
    #[error("{element_type} is not a {value_type}")]
    IncompatibleType {
        element_type: String,
        value_type: String,
    },
    #[error("{value_type} is not subtype of {field_type}")]
    NotASubtype {
        value_type: String,
        field_type: String,
    },
    #[error("{value} is not a {field_type}")]
    NotA { value: String, field_type: String },
    #[error("{value_type} is immutable")]
    Immutable { value_type: String },
}

/// Array layout errors.
#[derive(Debug, Error)]
pub enum ArrayLayoutError {
    #[error("element type is {element_type}, which is not a bits union")]
    NotUnion { element_type: String },
    #[error("element type is {element_type}, which is not stored inline")]
    NotInline { element_type: String },
    #[error("element type is {element_type}, which has pointer fields")]
    NotBits { element_type: String },
    #[error("element type is {element_type}, which is stored inline")]
    NotPointer { element_type: String },
    #[error("rank is {found}, not {provided}")]
    RankMismatch { found: isize, provided: isize },
}

/// Data access errors.
#[derive(Debug, Error)]
pub enum AccessError {
    #[error("{type_name} has no field named {field_name}")]
    NoSuchField {
        type_name: String,
        field_name: String,
    },
    #[error("layout is invalid for {value_type}")]
    InvalidLayout { value_type: String },
    #[error("no value named {name} in {module}")]
    GlobalNotFound { name: String, module: String },
    #[error("the current value is locked")]
    Locked,
    #[error("{tag} is not a valid tag for {union_type}")]
    IllegalUnionTag { union_type: String, tag: usize },
    #[error("field {field_name} of type {value_type} is not stored as a pointer")]
    NotAPointerField {
        value_type: String,
        field_name: String,
    },
    #[error("Data is already borrowed")]
    BorrowError,
    #[error("field at index {idx} does not exist: {value_type} has {n_fields} fields")]
    OutOfBoundsField {
        idx: usize,
        n_fields: usize,
        value_type: String,
    },
    #[error("index {idx} is out-of-bounds for SimpleVector of length {len}")]
    OutOfBoundsSVec { idx: usize, len: usize },
    #[error("index {idx} is invalid for array with shape {sz}")]
    InvalidIndex { idx: Dimensions, sz: Dimensions },
    #[error("arrays can only be accessed with n-dimensional indices")]
    ArrayNeedsNumericalIndex,
    #[error("fields cannot be accessed with n-dimensional indices")]
    FieldNeedsSimpleIndex,
    #[error("cannot access undefined reference")]
    UndefRef,
}

/// Data instantiation errors.
#[derive(Debug, Error)]
pub enum InstantiationError {
    #[error("cannot create array with DataType::instantiate")]
    ArrayNotSupported,
    #[error("NamedTuples must have an equal number of keys and values, got {n_names} keys and {n_values} values")]
    NamedTupleSizeMismatch { n_names: usize, n_values: usize },
    #[error("expected a shape for {vec_size} elements, got a shape for {dim_size} elements")]
    ArraySizeMismatch { dim_size: usize, vec_size: usize },
}

/// Julia exception converted to a string.
#[derive(Debug, Error)]
#[error("{msg}")]
pub struct Exception {
    msg: String,
}

impl Exception {
    /// Returns a reference to the error message.
    pub fn get_message(&self) -> &str {
        &self.msg
    }
}

/// All different errors.
#[derive(Debug, Error)]
pub enum JlrsError {
    #[error("Other: {0}")]
    Other(Box<dyn StdErr + 'static + Send + Sync>),
    #[error("Exception: {0}")]
    Exception(Exception),
    #[error("Runtime error: {0}")]
    RuntimeError(RuntimeError),
    #[error("Type error: {0}")]
    TypeError(TypeError),
    #[error("IO error: {0}")]
    IOError(IOError),
    #[error("Access error: {0}")]
    AccessError(AccessError),
    #[error("Instantiation error: {0}")]
    InstantiationError(InstantiationError),
    #[error("Array layout error: {0}")]
    ArrayLayoutError(ArrayLayoutError),
}

impl JlrsError {
    /// Convert an arbitrary error to `JlrsError::Other`.
    pub fn other<E: StdErr + 'static + Send + Sync>(reason: E) -> Self {
        JlrsError::Other(Box::new(reason))
    }

    /// Convert an error message to `JlrsError::Exception`.
    pub fn exception<S: Into<String>>(msg: S) -> Self {
        JlrsError::Exception(Exception { msg: msg.into() })
    }

    /// Convert an arbitrary error to `Err(JlrsError::Other)`.
    pub fn other_error<T, E: StdErr + 'static + Send + Sync>(reason: E) -> Result<T, Self> {
        Err(Self::other(reason))
    }

    /// Convert an error message to `Err(JlrsError::Exception)`.
    pub fn exception_error<T>(msg: String) -> Result<T, Self> {
        Err(JlrsError::exception(msg))
    }
}

macro_rules! impl_from {
    ($type:ident) => {
        impl From<$type> for JlrsError {
            fn from(e: $type) -> Self {
                JlrsError::$type(e)
            }
        }

        impl From<$type> for Box<JlrsError> {
            fn from(e: $type) -> Self {
                Box::new(JlrsError::from(e))
            }
        }
    };
}

impl_from!(RuntimeError);
impl_from!(TypeError);
impl_from!(IOError);
impl_from!(AccessError);
impl_from!(InstantiationError);
impl_from!(ArrayLayoutError);
