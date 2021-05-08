//! Support for values with the `Core.SimpleVector` (`SVec`) type.

use crate::private::Private;
use crate::value::Value;
use crate::{
    convert::cast::Cast,
    error::{JlrsError, JlrsResult},
    memory::{global::Global, traits::frame::Frame},
};
use crate::{impl_julia_typecheck, impl_valid_layout};
use jl_sys::{
    jl_alloc_svec, jl_alloc_svec_uninit, jl_emptysvec, jl_gc_wb, jl_simplevector_type,
    jl_svec_data, jl_svec_t,
};
use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
    ptr::{null_mut, NonNull},
};

use super::{
    traits::wrapper::Wrapper,
    wrapper_ref::{ValueRef, WrapperRef},
};

/// A `SimpleVector` is a fixed-size array that contains `Value`s.
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct SimpleVector<'frame, T = Value<'frame, 'static>>(
    NonNull<jl_svec_t>,
    PhantomData<&'frame ()>,
    PhantomData<T>,
)
where
    T: Wrapper<'frame, 'static>;

impl<'frame, T: Wrapper<'frame, 'static>> SimpleVector<'frame, T> {
    pub(crate) unsafe fn wrap(svec: *mut jl_svec_t) -> Self {
        debug_assert!(!svec.is_null());
        SimpleVector(NonNull::new_unchecked(svec), PhantomData, PhantomData)
    }

    #[doc(hidden)]
    pub unsafe fn inner(self) -> NonNull<jl_svec_t> {
        self.0
    }

    /// Create a new `SimpleVector` that can hold `n` values.
    pub fn with_capacity<F>(frame: &mut F, n: usize) -> JlrsResult<Self>
    where
        F: Frame<'frame>,
    {
        unsafe {
            let svec = jl_alloc_svec(n);
            if let Err(err) = frame.push_root(svec.cast(), Private) {
                Err(JlrsError::AllocError(err))?
            };

            Ok(SimpleVector::wrap(svec))
        }
    }

    /// Create a new `SimpleVector` that can hold `n` values without initializing its contents.
    /// The contents must be set before calling Julia again.
    pub unsafe fn with_capacity_uninit<F>(frame: &mut F, n: usize) -> JlrsResult<Self>
    where
        F: Frame<'frame>,
    {
        let svec = jl_alloc_svec_uninit(n);
        if let Err(err) = frame.push_root(svec.cast(), Private) {
            Err(JlrsError::AllocError(err))?
        };

        Ok(SimpleVector::wrap(svec))
    }

    /// Returns the length of this `SimpleVector`.
    pub fn len(self) -> usize {
        unsafe { (&*self.inner().as_ptr()).length }
    }

    /// Returns the data of this `SimpleVector`.
    pub fn data(self) -> &'frame [WrapperRef<'frame, 'static, T>] {
        unsafe {
            std::slice::from_raw_parts(jl_svec_data(self.inner().as_ptr()).cast(), self.len())
        }
    }

    pub unsafe fn set<'data>(
        self,
        index: usize,
        value: Option<Value<'_, 'data>>,
    ) -> JlrsResult<ValueRef<'frame, 'static>> {
        if index >= self.len() {
            Err(JlrsError::OutOfBounds(index, self.len()))?;
        }

        let mut_slice =
            std::slice::from_raw_parts_mut(jl_svec_data(self.inner().as_ptr()).cast(), self.len());
        mut_slice[index] = value;
        if let Some(value) = value {
            jl_gc_wb(self.inner().as_ptr().cast(), value.inner().as_ptr());
        }

        let ptr = value.map(|v| v.inner().as_ptr()).unwrap_or(null_mut());
        Ok(ValueRef::wrap(ptr))
    }

    /// Convert `self` to a `Value`.
    pub fn as_value(self) -> Value<'frame, 'static> {
        self.into()
    }
}

impl<'base, T: Wrapper<'base, 'static>> SimpleVector<'base, T> {
    pub fn emptysvec(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_emptysvec) }
    }
}

impl<'scope, T: Wrapper<'scope, 'static>> Debug for SimpleVector<'scope, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("SimpleVector").finish()
    }
}

impl<'frame, T: Wrapper<'frame, 'static>> Into<Value<'frame, 'static>> for SimpleVector<'frame, T> {
    fn into(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap_non_null(self.inner().cast()) }
    }
}

unsafe impl<'frame, 'data> Cast<'frame, 'data> for SimpleVector<'frame, Value<'frame, 'static>> {
    type Output = Self;
    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
        if value.is::<Self::Output>() {
            return unsafe { Ok(Self::cast_unchecked(value)) };
        }

        Err(JlrsError::NotAnSVec)?
    }

    unsafe fn cast_unchecked(value: Value<'frame, 'data>) -> Self::Output {
        Self::wrap(value.inner().as_ptr().cast())
    }
}

impl_julia_typecheck!(SimpleVector<'frame, Value<'frame, 'static>>, jl_simplevector_type, 'frame);

impl_valid_layout!(SimpleVector<'frame, Value<'frame, 'static>>, 'frame);
