//! Everything related to errors.

use crate::wrappers::ptr::{array::dimensions::Dimensions, value::Value, ValueRef};
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// Alias that is used for most `Result`s in this crate.
pub type JlrsResult<T> = Result<T, Box<JlrsError>>;

/// This type alias is used to encode the result of a function call: `Ok` indicates the call was
/// successful and contains the function's result, while `Err` indicates an exception was thrown
/// and contains said exception.
pub type JuliaResult<'frame, 'data, V = Value<'frame, 'data>> = Result<V, Value<'frame, 'data>>;
pub type JuliaResultRef<'frame, 'data, V = ValueRef<'frame, 'data>> =
    Result<V, ValueRef<'frame, 'data>>;

pub static CANNOT_DISPLAY_TYPE: &'static str = "<Cannot display type>";
pub static CANNOT_DISPLAY_VALUE: &'static str = "<Cannot display value>";

/// All different errors.
#[derive(Debug)]
pub enum JlrsError {
    Other(Box<dyn Error + Send + Sync>),
    Exception {
        msg: String,
    },
    AlreadyInitialized,
    ConstAlreadyExists {
        name: String,
        module: String,
        value: String,
    },
    NotAUnionArray {
        elem_ty: String,
        inline: bool,
    },
    NotAType {
        type_str: String,
    },
    NotAFunction {
        name: String,
        ty: String,
    },
    NotANamedTuple {
        type_str: String,
    },
    NotUTF8,
    NotATypeLB {
        typevar_name: String,
    },
    NotATypeUB {
        typevar_name: String,
    },
    InvalidBody {
        body_type_name: String,
    },
    NotAKind {
        type_name: String,
    },
    GlobalNotFound {
        name: String,
        module: String,
    },
    IncludeNotFound {
        path: String,
    },
    IncludeError {
        path: String,
        msg: String,
    },
    NoSuchField {
        type_name: String,
        field_name: String,
    },
    ElementTypeError {
        element_type_str: String,
        value_type_str: String,
    },
    InvalidLayout {
        value_type_str: String,
    },
    NotAModule {
        name: String,
    },
    AllocError(AllocError),
    WrongType {
        value_type: String,
    },
    IllegalUnionTag {
        union_type: String,
        tag: usize,
    },
    NotInline {
        element_type: String,
    },
    NullFrame,
    Inline {
        element_type: String,
    },
    NotAPointerField {
        value_type: String,
        field_idx: usize,
        field_name: String,
        field_type: String,
    },
    OutOfBounds {
        idx: usize,
        n_fields: usize,
        value_type: String,
    },
    OutOfBoundsSVec {
        idx: usize,
        n_fields: usize,
    },
    InvalidIndex {
        idx: Dimensions,
        sz: Dimensions,
    },
    Immutable {
        value_type: String,
    },
    NotSubtype {
        value_type: String,
        field_type: String,
    },
    NotConcrete {
        value_type: String,
    },
    ArrayNotSupported,
    NamedTupleSizeMismatch {
        n_names: usize,
        n_values: usize,
    },
    ArraySizeMismatch {
        dim_size: usize,
        vec_size: usize,
    },
    NumThreadsVar {
        value: String,
    },
    MoreThreadsRequired,
    ThreadsVarRequired,
    UndefRef,
}

/// Create a new `JlrsError::Exception` and wrap it in a `JlrsResult::Err`.
pub fn exception<T>(exc: String) -> JlrsResult<T> {
    Err(JlrsError::Exception { msg: exc })?
}

impl JlrsError {
    pub fn other<E: Error + 'static + Send + Sync>(reason: E) -> Self {
        JlrsError::Other(Box::new(reason))
    }

    pub(crate) fn alloc_error(a: AllocError) -> Self {
        JlrsError::AllocError(a)
    }
}

impl Display for JlrsError {
    fn fmt(&self, formatter: &mut Formatter) -> FmtResult {
        match self {
            JlrsError::ArraySizeMismatch { dim_size, vec_size } => {
                write!(
                    formatter,
                    "The array has {} elements, but {} have been provided",
                    dim_size, vec_size
                )
            }
            JlrsError::Other(other) => write!(formatter, "An error occurred: {}", other),
            JlrsError::AlreadyInitialized => {
                write!(formatter, "The runtime was already initialized, it can only be initialized once in a process")
            }
            JlrsError::Exception { msg } => formatter.write_str(&msg),
            JlrsError::NotAUnionArray { elem_ty, inline } => {
                let inline = if *inline { "inline" } else { "as a value" };
                write!(
                    formatter,
                    "The type of the elements is {}, which is stored {}",
                    elem_ty, inline
                )
            }
            JlrsError::NotAFunction { name, ty } => {
                write!(formatter, "{} is not a function, but a {}", name, ty)
            }
            JlrsError::NotUTF8 => write!(formatter, "The string contains invalid characters"),
            JlrsError::ConstAlreadyExists {
                name,
                module,
                value,
            } => {
                write!(
                    formatter,
                    "The constant {} already exists in {} and has the following value: {}",
                    name, module, value
                )
            }
            JlrsError::GlobalNotFound { name, module } => {
                write!(
                    formatter,
                    "The global {} could not be found in module {}",
                    name, module
                )
            }
            JlrsError::NoSuchField {
                type_name,
                field_name,
            } => {
                write!(
                    formatter,
                    "The type {} has no field named {}",
                    type_name, field_name
                )
            }
            JlrsError::IncludeNotFound { path } => {
                write!(formatter, "The file {} could not be found", path)
            }
            JlrsError::IncludeError { path, msg } => write!(
                formatter,
                "The file {} could not be included successfully. Exception: {}",
                path, msg
            ),
            JlrsError::IllegalUnionTag { union_type, tag } => write!(
                formatter,
                "The tag is {} but this tag is not valid for the union {}",
                tag, union_type
            ),
            JlrsError::ElementTypeError {
                element_type_str,
                value_type_str,
            } => write!(
                formatter,
                "Cannot set array value: {} is not a subtype or variant of {}",
                value_type_str, element_type_str
            ),
            JlrsError::NullFrame => write!(
                formatter,
                "NullFrame cannot be used to root values or to create a nested scope"
            ),
            JlrsError::NotAPointerField {
                value_type,
                field_idx,
                field_name,
                field_type,
            } => {
                write!(
                    formatter,
                    "The field {} of type {} (index: {}) has type {} which is stored inline",
                    field_name, value_type, field_idx, field_type
                )
            }
            JlrsError::NotInline { element_type } => {
                write!(
                    formatter,
                    "The elements of this array have the type {} which is not stored inline",
                    element_type
                )
            }
            JlrsError::NotANamedTuple { type_str } => {
                write!(
                    formatter,
                    "The provided keywords are not a NamedTuple, but {}",
                    type_str
                )
            }
            JlrsError::NotATypeLB { typevar_name } => {
                write!(
                    formatter,
                    "The lower bound of {} is not a type",
                    typevar_name
                )
            }
            JlrsError::NotATypeUB { typevar_name } => {
                write!(
                    formatter,
                    "The upper bound of {} is not a type",
                    typevar_name
                )
            }
            JlrsError::InvalidLayout { value_type_str } => {
                write!(formatter, "The layout is invalid for {}", value_type_str)
            }
            JlrsError::NotAKind { type_name } => {
                write!(formatter, "The type {} is not a kind", type_name)
            }
            JlrsError::InvalidBody {
                body_type_name: body_ty,
            } => write!(
                formatter,
                "The body of a UnionAll must be a type or a TypeVar. Found: {}",
                body_ty
            ),
            JlrsError::Immutable { value_type } => {
                write!(formatter, "The type {} is immutable", value_type)
            }
            JlrsError::NotSubtype {
                value_type,
                field_type,
            } => {
                write!(
                    formatter,
                    "{} is not a subtype of {}",
                    value_type, field_type
                )
            }
            JlrsError::Inline { element_type } => {
                write!(
                    formatter,
                    "The elements of this array have the type {} which is stored inline",
                    element_type
                )
            }
            JlrsError::MoreThreadsRequired => write!(
                formatter,
                "The JULIA_NUM_THREADS environment variable must be set to a value larger than 1"
            ),
            JlrsError::NotAType { type_str } => {
                write!(formatter, "Expected a type, got: {}", type_str)
            }
            JlrsError::NotAModule { name } => write!(formatter, "{} is not a module", name),
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
            JlrsError::WrongType { value_type } => {
                write!(
                    formatter,
                    "The provided type is not valid for {}",
                    value_type
                )
            }
            JlrsError::OutOfBounds {
                idx,
                n_fields,
                value_type,
            } => write!(
                formatter,
                "Cannot access field at index {} because the type {} has {} fields",
                idx, value_type, n_fields
            ),
            JlrsError::OutOfBoundsSVec { idx, n_fields } => write!(
                formatter,
                "Cannot access index {} because this SimpleVector has length {}",
                idx, n_fields
            ),
            JlrsError::InvalidIndex { idx, sz } => write!(
                formatter,
                "Index {} is not valid for array with shape {}",
                idx, sz
            ),
            JlrsError::NotConcrete { value_type } => {
                write!(formatter, "{} is not a concrete type", value_type)
            }
            JlrsError::NamedTupleSizeMismatch { n_names, n_values } => {
                write!(
                    formatter,
                    "A named tuple must have an equal number of names and values, but {} name(s) and {} values(s) were given",
                    n_names,
                    n_values
                )
            }
            JlrsError::ArrayNotSupported => {
                write!(
                    formatter,
                    "Array types cannot be instantiated with `DataType::instantiate`, but must \
                    be created with one of the methods provided by `Array` and `TypedArray`",
                )
            }
            JlrsError::UndefRef => {
                write!(formatter, "An undefined reference cannot be rooted")
            }
            JlrsError::ThreadsVarRequired => {
                write!(formatter, "On Windows the JULIA_NUM_THREADS environment variable must be explicitly set to 3 or higher, or auto.")
            }
            JlrsError::NumThreadsVar { value } => {
                write!(formatter, "The `JULIA_NUM_THREADS` environment variable must be set to a value larger than 2 or auto, but its value is: {}", value)
            }
        }
    }
}

impl Error for JlrsError {}

impl Into<Box<JlrsError>> for Box<dyn Error + 'static + Send + Sync> {
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
