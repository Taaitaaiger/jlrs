//! Support for values with the `Core.MethodTable` type.
//!
//! The documentation for this module has been slightly adapted from the comments for this struct
//! in [`julia.h`]
//!
//! [`julia.h`]: https://github.com/JuliaLang/julia/blob/96786e22ccabfdafd073122abb1fb69cea921e17/src/julia.h#L535

use super::{array::Array, module::Module, symbol::Symbol, Value};
use crate::convert::cast::Cast;
use crate::error::{JlrsError, JlrsResult};
use crate::{impl_julia_type, impl_julia_typecheck, impl_valid_layout};
use jl_sys::{jl_methtable_t, jl_methtable_type};
use std::marker::PhantomData;

/// contains the TypeMap for one Type
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct MethodTable<'frame>(*mut jl_methtable_t, PhantomData<&'frame ()>);

impl<'frame> MethodTable<'frame> {
    pub(crate) unsafe fn wrap(method_table: *mut jl_methtable_t) -> Self {
        MethodTable(method_table, PhantomData)
    }

    #[doc(hidden)]
    pub unsafe fn ptr(self) -> *mut jl_methtable_t {
        self.0
    }

    /// Sometimes a hack used by serialization to handle kwsorter
    pub fn name(self) -> Symbol<'frame> {
        unsafe { Symbol::wrap((&*self.ptr()).name) }
    }

    /// The `defs` field.
    pub fn defs(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap((&*self.ptr()).defs) }
    }

    /// The `cache` field.
    pub fn cache(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap((&*self.ptr()).cache) }
    }

    /// Max # of non-vararg arguments in a signature
    pub fn max_args(self) -> isize {
        unsafe { (&*self.ptr()).max_args }
    }

    /// Keyword argument sorter function
    pub fn kwsorter(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap((&*self.ptr()).kwsorter) }
    }

    /// Used for incremental serialization to locate original binding
    pub fn module(self) -> Module<'frame> {
        unsafe { Module::wrap((&*self.ptr()).module) }
    }

    /// The `backedges` field.
    pub fn backedges(self) -> Array<'frame, 'static> {
        unsafe { Array::wrap((&*self.ptr()).backedges) }
    }

    /// 0, or 1 to skip splitting typemap on first (function) argument
    pub fn offs(self) -> u8 {
        unsafe { (&*self.ptr()).offs }
    }

    // Whether this accepts adding new methods
    pub fn frozen(self) -> u8 {
        unsafe { (&*self.ptr()).frozen }
    }

    /// Convert `self` to a `Value`.
    pub fn as_value(self) -> Value<'frame, 'static> {
        self.into()
    }
}

impl<'frame> Into<Value<'frame, 'static>> for MethodTable<'frame> {
    fn into(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap(self.ptr().cast()) }
    }
}

unsafe impl<'frame, 'data> Cast<'frame, 'data> for MethodTable<'frame> {
    type Output = Self;
    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
        if value.is::<Self::Output>() {
            return unsafe { Ok(Self::cast_unchecked(value)) };
        }

        Err(JlrsError::NotAMethTable)?
    }

    unsafe fn cast_unchecked(value: Value<'frame, 'data>) -> Self::Output {
        Self::wrap(value.ptr().cast())
    }
}

impl_julia_typecheck!(MethodTable<'frame>, jl_methtable_type, 'frame);
impl_julia_type!(MethodTable<'frame>, jl_methtable_type, 'frame);
impl_valid_layout!(MethodTable<'frame>, 'frame);
