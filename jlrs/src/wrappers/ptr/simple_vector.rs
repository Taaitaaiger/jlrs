//! Wrapper for `SimpleVector`.

use crate::wrappers::ptr::value::Value;
use crate::{
    error::{JlrsError, JlrsResult},
    memory::{frame::Frame, global::Global},
};
use crate::{layout::typecheck::Typecheck, memory::output::Output};
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
use crate::wrappers::ptr::Ref;

/// A `SimpleVector` is a fixed-size array that contains `Value`s.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct SimpleVector<'scope, T = Value<'scope, 'static>>(
    NonNull<jl_svec_t>,
    PhantomData<&'scope ()>,
    PhantomData<T>,
)
where
    T: Wrapper<'scope, 'static>;

impl<'scope, T: Wrapper<'scope, 'static>> SimpleVector<'scope, T> {
    /// Create a new `SimpleVector` that can hold `n` values.
    pub fn with_capacity<F>(frame: &mut F, n: usize) -> JlrsResult<Self>
    where
        F: Frame<'scope>,
    {
        unsafe {
            let svec = NonNull::new_unchecked(jl_alloc_svec(n));
            let v = frame
                .push_root(svec, Private)
                .map_err(JlrsError::alloc_error)?;
            Ok(v)
        }
    }

    /// Create a new `SimpleVector` that can hold `n` values without initializing its contents.
    ///
    /// Safety: The contents must be set before calling Julia again, the contents must never be
    /// accessed before all elements are set.
    pub unsafe fn with_capacity_uninit<F>(frame: &mut F, n: usize) -> JlrsResult<Self>
    where
        F: Frame<'scope>,
    {
        let svec = NonNull::new_unchecked(jl_alloc_svec_uninit(n));
        let v = frame
            .push_root(svec, Private)
            .map_err(JlrsError::alloc_error)?;
        Ok(v)
    }

    /// Returns the length of this `SimpleVector`.
    pub fn len(self) -> usize {
        unsafe { self.unwrap_non_null(Private).as_ref().length }
    }

    /// Returns the data of this `SimpleVector`.
    ///
    /// Safety: the type `T` must be the type of all elements in the simple vector.
    pub unsafe fn data(self) -> &'scope [Ref<'scope, 'static, T>] {
        std::slice::from_raw_parts(jl_svec_data(self.unwrap(Private)).cast(), self.len())
    }

    /// Returns the data of this `SimpleVector`.
    ///
    /// Safety: the type `T` must be the type of all elements in the simple vector.
    pub unsafe fn data_unchecked(self) -> &'scope [T] {
        std::slice::from_raw_parts(jl_svec_data(self.unwrap(Private)).cast(), self.len())
    }

    /// Set the element at `index` to `value`. This is only safe if the `SimpleVector` has just
    /// been allocated.
    ///
    /// Safety: you may only mutate a `SimpleVector` after creating it, they should generally be
    /// considered immutable.
    pub unsafe fn set(self, index: usize, value: Option<T>) -> JlrsResult<Ref<'scope, 'static, T>> {
        if index >= self.len() {
            Err(JlrsError::OutOfBoundsSVec {
                idx: index,
                n_fields: self.len(),
            })?
        }

        jl_svec_data(self.unwrap(Private))
            .cast::<Option<T>>()
            .add(index)
            .write(value);

        if let Some(value) = value {
            jl_gc_wb(self.unwrap(Private).cast(), value.unwrap(Private).cast());
        }

        let ptr = value.map(|v| v.unwrap(Private)).unwrap_or(null_mut());
        Ok(Ref::wrap(ptr))
    }
}

impl<'scope> SimpleVector<'scope, Value<'scope, 'static>> {
    /// Use the `Output` to extend the lifetime of this data.
    pub fn root<'target>(self, output: Output<'target>) -> SimpleVector<'target> {
        unsafe {
            let ptr = self.unwrap_non_null(Private);
            output.set_root::<SimpleVector>(ptr);
            SimpleVector::wrap_non_null(ptr, Private)
        }
    }
}

impl<'base, T: Wrapper<'base, 'static>> SimpleVector<'base, T> {
    /// The empty `SimpleVector`.
    pub fn emptysvec(_: Global<'base>) -> Self {
        unsafe { Self::wrap(jl_emptysvec, Private) }
    }
}

unsafe impl<'scope, T: Wrapper<'scope, 'static>> Typecheck for SimpleVector<'scope, T> {
    fn typecheck(t: DataType) -> bool {
        unsafe { t.unwrap(Private) == jl_simplevector_type }
    }
}

unsafe impl<'scope, T: Wrapper<'scope, 'static>> ValidLayout for SimpleVector<'scope, T> {
    fn valid_layout(v: Value) -> bool {
        if let Ok(dt) = v.cast::<DataType>() {
            dt.is::<SimpleVector>()
        } else {
            false
        }
    }
}

impl<'scope, T: Wrapper<'scope, 'static>> Debug for SimpleVector<'scope, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self.display_string() {
            Ok(s) => write!(f, "{}", s),
            Err(e) => write!(f, "<Cannot display value: {}>", e),
        }
    }
}

impl<'scope, T: Wrapper<'scope, 'static>> WrapperPriv<'scope, '_> for SimpleVector<'scope, T> {
    type Wraps = jl_svec_t;
    const NAME: &'static str = "SimpleVector";

    #[inline(always)]
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData, PhantomData)
    }

    #[inline(always)]
    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

// TODO: T
impl_root!(SimpleVector, 1);
