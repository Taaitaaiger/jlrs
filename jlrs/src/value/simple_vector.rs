//! Support for values with the `Core.SimpleVector` (`SVec`) type.

use crate::error::{JlrsError, JlrsResult};
use crate::global::Global;
use crate::traits::{private::Internal, Cast, Frame};
use crate::value::Value;
use crate::{impl_julia_type, impl_julia_typecheck, impl_valid_layout};
use jl_sys::{
    jl_alloc_svec, jl_alloc_svec_uninit, jl_emptysvec, jl_gc_wb, jl_simplevector_type,
    jl_svec_data, jl_svec_t,
};
use std::marker::PhantomData;

/// A `SimpleVector` is a fixed-size array that contains `Value`s.
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct SimpleVector<'frame>(*mut jl_svec_t, PhantomData<&'frame ()>);

impl<'frame> SimpleVector<'frame> {
    pub(crate) unsafe fn wrap(svec: *mut jl_svec_t) -> Self {
        SimpleVector(svec, PhantomData)
    }

    #[doc(hidden)]
    pub unsafe fn ptr(self) -> *mut jl_svec_t {
        self.0
    }

    pub fn with_capacity<F>(frame: &mut F, n: usize) -> JlrsResult<Self>
    where
        F: Frame<'frame>,
    {
        unsafe {
            let svec = jl_alloc_svec(n);
            if let Err(err) = frame.protect(svec.cast(), Internal) {
                Err(JlrsError::AllocError(err))?
            };

            Ok(SimpleVector::wrap(svec))
        }
    }

    pub unsafe fn with_capacity_uninit<F>(frame: &mut F, n: usize) -> JlrsResult<Self>
    where
        F: Frame<'frame>,
    {
        let svec = jl_alloc_svec_uninit(n);
        if let Err(err) = frame.protect(svec.cast(), Internal) {
            Err(JlrsError::AllocError(err))?
        };

        Ok(SimpleVector::wrap(svec))
    }

    /// Returns the length of this `SimpleVector`.
    pub fn len(self) -> usize {
        unsafe { (&*self.ptr()).length }
    }

    /// Returns the data of this `SimpleVector`.
    pub fn data(self) -> &'frame [Value<'frame, 'static>] {
        unsafe { std::slice::from_raw_parts(jl_svec_data(self.ptr()).cast(), self.len()) }
    }

    pub unsafe fn set<'data>(
        self,
        index: usize,
        value: Value<'_, 'data>,
    ) -> JlrsResult<Value<'frame, 'data>> {
        if index >= self.len() {
            Err(JlrsError::OutOfBounds(index, self.len()))?;
        }

        let mut_slice = std::slice::from_raw_parts_mut(jl_svec_data(self.ptr()).cast(), self.len());
        mut_slice[index] = value;
        if value.ptr() != std::ptr::null_mut() {
            jl_gc_wb(self.ptr().cast(), value.ptr());
        }

        Ok(Value::wrap(value.ptr()))
    }

    pub fn as_value(self) -> Value<'frame, 'static> {
        self.into()
    }
}

impl<'base> SimpleVector<'base> {
    pub fn emptysvec(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_emptysvec) }
    }
}

impl<'frame> Into<Value<'frame, 'static>> for SimpleVector<'frame> {
    fn into(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap(self.ptr().cast()) }
    }
}

unsafe impl<'frame, 'data> Cast<'frame, 'data> for SimpleVector<'frame> {
    type Output = Self;
    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
        if value.is::<Self::Output>() {
            return unsafe { Ok(Self::cast_unchecked(value)) };
        }

        Err(JlrsError::NotAnSVec)?
    }

    unsafe fn cast_unchecked(value: Value<'frame, 'data>) -> Self::Output {
        Self::wrap(value.ptr().cast())
    }
}

impl_julia_typecheck!(SimpleVector<'frame>, jl_simplevector_type, 'frame);
impl_julia_type!(SimpleVector<'frame>, jl_simplevector_type, 'frame);
impl_valid_layout!(SimpleVector<'frame>, 'frame);
