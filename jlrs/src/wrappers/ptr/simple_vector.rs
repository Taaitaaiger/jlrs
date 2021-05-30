//! Wrapper for `Core.SimpleVector`.

use crate::layout::typecheck::Typecheck;
use crate::wrappers::ptr::value::Value;
use crate::{
    error::{JlrsError, JlrsResult},
    memory::{global::Global, traits::frame::Frame},
};
use crate::{layout::valid_layout::ValidLayout, private::Private};
use jl_sys::{
    jl_alloc_svec, jl_alloc_svec_uninit, jl_emptysvec, jl_gc_wb, jl_simplevector_type,
    jl_svec_data, jl_svec_t,
};
use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
    ptr::{null_mut, NonNull},
};

use super::{datatype::DataType, private::Wrapper as WrapperPriv, Wrapper};
use crate::wrappers::ptr::{Ref, ValueRef};

/// A `SimpleVector` is a fixed-size array that contains `Value`s.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct SimpleVector<'frame, T = Value<'frame, 'static>>(
    NonNull<jl_svec_t>,
    PhantomData<&'frame ()>,
    PhantomData<T>,
)
where
    T: Wrapper<'frame, 'static>;

impl<'frame, T: Wrapper<'frame, 'static>> SimpleVector<'frame, T> {
    /// Create a new `SimpleVector` that can hold `n` values.
    pub fn with_capacity<F>(frame: &mut F, n: usize) -> JlrsResult<Self>
    where
        F: Frame<'frame>,
    {
        unsafe {
            let svec = NonNull::new_unchecked(jl_alloc_svec(n));
            if let Err(err) = frame.push_root(svec.cast(), Private) {
                Err(JlrsError::AllocError(err))?
            };

            Ok(SimpleVector::wrap_non_null(svec, Private))
        }
    }

    /// Create a new `SimpleVector` that can hold `n` values without initializing its contents.
    /// The contents must be set before calling Julia again, the contents must never be accessed
    /// before all elements are set.
    pub unsafe fn with_capacity_uninit<F>(frame: &mut F, n: usize) -> JlrsResult<Self>
    where
        F: Frame<'frame>,
    {
        let svec = NonNull::new_unchecked(jl_alloc_svec_uninit(n));
        if let Err(err) = frame.push_root(svec.cast(), Private) {
            Err(JlrsError::AllocError(err))?
        };

        Ok(SimpleVector::wrap_non_null(svec, Private))
    }

    /// Returns the length of this `SimpleVector`.
    pub fn len(self) -> usize {
        unsafe { self.unwrap_non_null(Private).as_ref().length }
    }

    /// Returns the data of this `SimpleVector`.
    ///
    /// Safety: the type `T` must be the type of all elements in the simple vector.
    pub unsafe fn data(self) -> &'frame [Ref<'frame, 'static, T>] {
        std::slice::from_raw_parts(jl_svec_data(self.unwrap(Private)).cast(), self.len())
    }

    /// Set the element at `index` to `value`. This is only safe if the `SimpleVector` has just
    /// been allocated.
    pub unsafe fn set<'data>(
        self,
        index: usize,
        value: Option<Value<'_, 'data>>,
    ) -> JlrsResult<ValueRef<'frame, 'static>> {
        if index >= self.len() {
            Err(JlrsError::OutOfBounds(index, self.len()))?;
        }

        jl_svec_data(self.unwrap(Private))
            .cast::<Option<Value>>()
            .add(index)
            .write(value);

        if let Some(value) = value {
            jl_gc_wb(self.unwrap(Private).cast(), value.unwrap(Private));
        }

        let ptr = value.map(|v| v.unwrap(Private)).unwrap_or(null_mut());
        Ok(ValueRef::wrap(ptr))
    }
}

impl<'base, T: Wrapper<'base, 'static>> SimpleVector<'base, T> {
    /// The empty `SimpleVector`.
    pub fn emptysvec(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_emptysvec, Private) }
    }
}

impl<'scope, T: Wrapper<'scope, 'static>> Debug for SimpleVector<'scope, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("SimpleVector").finish()
    }
}

unsafe impl<'scope, T: Wrapper<'scope, 'static>> Typecheck for SimpleVector<'scope, T> {
    unsafe fn typecheck(t: DataType) -> bool {
        t.unwrap(Private) == jl_simplevector_type
    }
}

unsafe impl<'frame, T: Wrapper<'frame, 'static>> ValidLayout for SimpleVector<'frame, T> {
    unsafe fn valid_layout(v: Value) -> bool {
        if let Ok(dt) = v.cast::<DataType>() {
            dt.is::<SimpleVector>()
        } else {
            false
        }
    }
}

impl<'scope, T: Wrapper<'scope, 'static>> WrapperPriv<'scope, '_> for SimpleVector<'scope, T> {
    type Internal = jl_svec_t;

    unsafe fn wrap_non_null(inner: NonNull<Self::Internal>, _: Private) -> Self {
        Self(inner, PhantomData, PhantomData)
    }

    unsafe fn unwrap_non_null(self, _: Private) -> NonNull<Self::Internal> {
        self.0
    }
}
