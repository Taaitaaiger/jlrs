//! Everything related to errors.

use crate::value::{array::dimensions::Dimensions, Value};
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// Alias that is used for most `Result`s in this crate.
pub type JlrsResult<T> = Result<T, Box<JlrsError>>;

/// This type alias is used to encode the result of a function call: `Ok` indicates the call was
/// successful and contains the function's result, while `Err` indicates an exception was thrown
/// and contains said exception.
pub type JuliaResult<'frame, 'data, V = Value<'frame, 'data>> = Result<V, Value<'frame, 'data>>;

/// All different errors.
#[derive(Debug)]
pub enum JlrsError {
    Other(Box<dyn Error + Send + Sync>),
    Exception(String),
    AlreadyInitialized,
    ConstAlreadyExists(String),
    NotAnArray,
    NotAUnionArray,
    Nothing,
    NotAType,
    NotADataType,
    NotAMethod,
    NotAMethodInstance,
    NotACodeInstance,
    NotAWeakRef,
    NotATypeMapEntry,
    NotATypeMapLevel,
    NotAnExpr,
    NotATask,
    NotASymbol,
    NotAString,
    NotUnicode,
    NotAnSVec,
    NotAnSSAValue,
    NotAMethodMatch,
    NotATypeName,
    NotATypeVar,
    NotATypeLB(String),
    NotATypeUB(String),
    NotAUnion,
    InvalidBody(String),
    NotAKind(String),
    NotAUnionAll,
    FunctionNotFound(String),
    IncludeNotFound(String),
    IncludeError(String, String),
    NoSuchField(String),
    InvalidArrayType,
    InvalidLayout,
    InvalidCharacter,
    NotAModule(String),
    NotAMethTable,
    AllocError(AllocError),
    WrongType,
    NotInline,
    NullFrame,
    Inline,
    NotAPointerField(usize),
    ZeroDimension,
    OutOfBounds(usize, usize),
    InvalidIndex(Dimensions, Dimensions),
    Immutable,
    NotSubtype,
    NotConcrete(String),
    ArrayNotSupported,
    NamedTupleSizeMismatch(usize, usize),
    MoreThreadsRequired,
}

/// Create a new `JlrsError::Exception` and wrap it in a `JlrsResult::Err`.
pub fn exception<T>(exc: String) -> JlrsResult<T> {
    Err(JlrsError::Exception(exc))?
}

impl JlrsError {
    pub fn other<E: Error + Send + Sync + 'static>(reason: E) -> Self {
        JlrsError::Other(Box::new(reason))
    }

    pub(crate) fn alloc_error(a: AllocError) -> Self {
        JlrsError::AllocError(a)
    }
}

impl Display for JlrsError {
    fn fmt(&self, formatter: &mut Formatter) -> FmtResult {
        match self {
            JlrsError::Other(other) => write!(formatter, "An error occurred: {}", other),
            JlrsError::AlreadyInitialized => {
                write!(formatter, "The runtime was already initialized")
            }
            JlrsError::Exception(exc) => write!(formatter, "An exception was thrown: {}", exc),
            JlrsError::NotAnArray => write!(formatter, "This is not an array"),
            JlrsError::NotAUnionArray => write!(formatter, "This is not a union array"),
            JlrsError::NotAString => write!(formatter, "This is not a string"),
            JlrsError::NotUnicode => write!(formatter, "This string contains invalid characters"),
            JlrsError::Nothing => write!(formatter, "This value is Nothing"),
            JlrsError::ConstAlreadyExists(name) => {
                write!(formatter, "The constant {} already exists", name)
            }
            JlrsError::FunctionNotFound(func) => {
                write!(formatter, "The function {} could not be found", func)
            }
            JlrsError::NoSuchField(field) => {
                write!(formatter, "The field {} could not be found", field)
            }
            JlrsError::IncludeNotFound(inc) => {
                write!(formatter, "The file {} could not be found", inc)
            }
            JlrsError::IncludeError(inc, err_type) => write!(
                formatter,
                "The file {} could not be included successfully. Exception type: {}",
                inc, err_type
            ),
            JlrsError::InvalidArrayType => write!(formatter, "Invalid array type"),
            JlrsError::InvalidCharacter => write!(formatter, "Invalid character"),
            JlrsError::NullFrame => write!(
                formatter,
                "NullFrames don't support allocations or nesting another NullFrame"
            ),
            JlrsError::NotAPointerField(idx) => {
                write!(formatter, "The field at index {} is stored inline", idx)
            }
            JlrsError::NotInline => {
                write!(formatter, "The data of this array is not stored inline")
            }
            JlrsError::NotAMethTable => write!(formatter, "This is not a method table"),
            JlrsError::NotAMethodMatch => write!(formatter, "This is not a method match"),
            JlrsError::NotAnSVec => write!(formatter, "This is not a simple vector"),
            JlrsError::NotAnSSAValue => write!(formatter, "This is not an SSA value"),
            JlrsError::NotATypeName => write!(formatter, "This is not a typename"),
            JlrsError::NotATypeVar => write!(formatter, "This is not a type var"),
            JlrsError::NotATypeLB(tv) => {
                write!(formatter, "The lower bound of {} is not a type", tv)
            }
            JlrsError::NotATypeUB(tv) => {
                write!(formatter, "The upper bound of {} is not a type", tv)
            }
            JlrsError::NotAUnion => write!(formatter, "This is not a union"),
            JlrsError::NotAUnionAll => write!(formatter, "This is not a UnionAll"),
            JlrsError::InvalidLayout => write!(formatter, "The layout is invalid"),

            JlrsError::NotAMethodInstance => write!(formatter, "This is not a method instance"),
            JlrsError::NotAKind(kind) => write!(formatter, "The type {} is not a kind", kind),
            JlrsError::InvalidBody(body_ty) => write!(
                formatter,
                "The body of a UnionAll must be a type or a TypeVar. Found: {}",
                body_ty
            ),
            JlrsError::NotACodeInstance => write!(formatter, "This is not a code instance"),
            JlrsError::NotAWeakRef => write!(formatter, "This is not a weak ref"),
            JlrsError::Immutable => write!(formatter, "This value is immutable"),
            JlrsError::NotSubtype => {
                write!(formatter, "Value type is not a subtype of the field type")
            }
            JlrsError::NotATypeMapEntry => write!(formatter, "This is not a typemap entry"),
            JlrsError::NotATypeMapLevel => write!(formatter, "This is not a typemap level"),
            JlrsError::NotAnExpr => write!(formatter, "This is not an expr"),
            JlrsError::NotATask => write!(formatter, "This is not a task"),

            JlrsError::Inline => write!(formatter, "The data of this array is stored inline"),
            JlrsError::MoreThreadsRequired => write!(
                formatter,
                "The JULIA_NUM_THREADS environment variable must be set to a value larger than 1"
            ),
            JlrsError::NotADataType => write!(formatter, "This is not a datatype"),
            JlrsError::NotAType => write!(formatter, "This is not a type"),
            JlrsError::NotAMethod => write!(formatter, "This is not a method"),
            JlrsError::NotASymbol => write!(formatter, "This is not a symbol"),
            JlrsError::NotAModule(module) => write!(formatter, "{} is not a module", module),
            JlrsError::AllocError(AllocError::FrameOverflow(n, cap)) => write!(
                formatter,
                "The frame cannot handle more data. Tried to allocate: {}; capacity: {}",
                n, cap,
            ),
            JlrsError::AllocError(AllocError::StackOverflow(n, cap)) => write!(
                formatter,
                "The stack cannot handle more data. Tried to allocate: {}; capacity: {}",
                n, cap,
            ),
            JlrsError::WrongType => {
                write!(formatter, "Requested type does not match the found type")
            }
            JlrsError::ZeroDimension => {
                write!(formatter, "Cannot handle arrays with zero dimensions")
            }
            JlrsError::OutOfBounds(idx, sz) => write!(
                formatter,
                "Cannot access value at index {} because the number of values is {}",
                idx, sz
            ),
            JlrsError::InvalidIndex(idx, sz) => write!(
                formatter,
                "Index {} is not valid for array with shape {}",
                idx, sz
            ),
            JlrsError::NotConcrete(type_name) => {
                write!(formatter, "{} is not a concrete DataType", type_name)
            }
            JlrsError::NamedTupleSizeMismatch(names, values) => {
                write!(
                    formatter,
                    "A named tuple must have an equal number of names and values, but {} name(s) and {} values(s) were given", 
                    names,
                    values
                )
            }
            JlrsError::ArrayNotSupported => {
                write!(
                    formatter,
                    "Array types cannot be instantiated with `DataType::instantiate`, but must \
                    be created with `Value::new_array`, `Value::move_array`, or \
                    `Value::borrow_array`.",
                )
            }
        }
    }
}

impl Error for JlrsError {}

impl Into<Box<JlrsError>> for Box<dyn Error + Send + Sync + 'static> {
    fn into(self) -> Box<JlrsError> {
        Box::new(JlrsError::Other(self))
    }
}

/// Frames and data they protect have a memory cost. If the memory set aside for containing frames
/// or the frame itself is exhausted, this error is returned.
#[derive(Copy, Clone, Debug)]
pub enum AllocError {
    //            desired, cap
    StackOverflow(usize, usize),
    FrameOverflow(usize, usize),
}

impl Into<JlrsError> for AllocError {
    fn into(self) -> JlrsError {
        JlrsError::AllocError(self)
    }
}

impl Into<Box<JlrsError>> for AllocError {
    fn into(self) -> Box<JlrsError> {
        Box::new(self.into())
    }
}
