//! Wrapper for `SimpleVector`.

use crate::{
    error::{AccessError, JlrsResult},
    layout::{typecheck::Typecheck, valid_layout::ValidLayout},
    memory::{
        frame::Frame,
        global::Global,
        output::Output,
        scope::{private::PartialScopePriv, PartialScope},
    },
    private::Private,
    wrappers::ptr::value::Value,
};
use jl_sys::{
    jl_alloc_svec, jl_alloc_svec_uninit, jl_emptysvec, jl_gc_wb, jl_svec_data, jl_svec_t,
};
use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
    ptr::NonNull,
};

use super::{
    datatype::DataType, private::WrapperPriv, value::ValueRef, Ref, Root, Wrapper, WrapperRef,
};

/// Access and mutate the contents of a `SimpleVector`.
#[repr(transparent)]
pub struct SimpleVectorData<'scope, 'borrow, T = ValueRef<'scope, 'static>>(
    NonNull<jl_svec_t>,
    PhantomData<&'scope ()>,
    PhantomData<&'borrow [T]>,
)
where
    T: WrapperRef<'scope, 'static>;

impl<'scope, 'borrow, T: WrapperRef<'scope, 'static>> SimpleVectorData<'scope, 'borrow, T> {
    /// Returns the length of this `SimpleVector`.
    pub fn len(&self) -> usize {
        // Safety: the pointer points to valid data
        unsafe { self.0.as_ref().length }
    }

    /// Returns the contents of this `SimpleVector` as a slice.
    pub fn as_slice(&self) -> &'borrow [T] {
        // Safety: the C API function is called with valid data
        unsafe { std::slice::from_raw_parts(jl_svec_data(self.0.as_ptr()).cast(), self.len()) }
    }

    /// Set the element at `index` to `value`. This is only safe if the `SimpleVector` has just
    /// been allocated.
    ///
    /// Safety: you may only mutate a `SimpleVector` after creating it, they should generally be
    /// considered immutable.
    pub unsafe fn set(&mut self, index: usize, value: Option<T::Wrapper>) -> JlrsResult<()> {
        if index >= self.len() {
            Err(AccessError::OutOfBoundsSVec {
                idx: index,
                len: self.len(),
            })?
        }

        jl_svec_data(self.0.as_ptr())
            .cast::<Option<T::Wrapper>>()
            .add(index)
            .write(value);

        if let Some(value) = value {
            jl_gc_wb(self.0.as_ptr().cast(), value.unwrap(Private).cast());
        };

        Ok(())
    }
}

/// A `SimpleVector` is a fixed-size array that contains `ValueRef`s.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct SimpleVector<'scope>(NonNull<jl_svec_t>, PhantomData<&'scope ()>);

impl<'scope> SimpleVector<'scope> {
    /// Create a new `SimpleVector` that can hold `n` values.
    pub fn with_capacity<F>(frame: &mut F, n: usize) -> JlrsResult<Self>
    where
        F: Frame<'scope>,
    {
        // Safety: the allocated data is immediately rooted
        unsafe {
            let svec = NonNull::new_unchecked(jl_alloc_svec(n));
            frame.value(svec, Private)
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
        frame.value(svec, Private)
    }

    /// Access the contents of this `SimpleVector` as `ValueRef`.
    pub fn data<'borrow, 'current, F>(
        self,
        _: &'borrow mut F,
    ) -> SimpleVectorData<'scope, 'borrow, ValueRef<'scope, 'static>>
    where
        F: Frame<'current>,
    {
        SimpleVectorData(self.unwrap_non_null(Private), PhantomData, PhantomData)
    }

    /// Access the contents of this `SimpleVector` as `U`.
    ///
    /// This method returns a `JlrsError::AccessError` if `U` isn't correct for all elements.
    pub fn typed_data<'borrow, 'current, U, F>(
        self,
        _: &'borrow mut F,
    ) -> JlrsResult<SimpleVectorData<'scope, 'borrow, U>>
    where
        F: Frame<'current>,
        U: WrapperRef<'scope, 'static>,
    {
        if !self.is_typed::<U>() {
            Err(AccessError::InvalidLayout {
                value_type_str: String::from("this SimpleVector"),
            })?;
        }

        Ok(SimpleVectorData(
            self.unwrap_non_null(Private),
            PhantomData,
            PhantomData,
        ))
    }

    /// Access the contents of this `SimpleVector` as `U`.
    ///
    /// Safety: this method doesn't check if `U` is correct for all elements.
    pub unsafe fn typed_data_unchecked<'borrow, 'current, U, F>(
        self,
        _: &'borrow mut F,
    ) -> SimpleVectorData<'scope, 'borrow, U>
    where
        F: Frame<'current>,
        U: WrapperRef<'scope, 'static>,
    {
        SimpleVectorData(self.unwrap_non_null(Private), PhantomData, PhantomData)
    }

    /// Access the contents of this `SimpleVector` as `ValueRef`.
    ///
    /// Safety: the lifetime borrow is not restricted by a frame.
    pub unsafe fn unrestricted_data(
        self,
    ) -> SimpleVectorData<'scope, 'scope, ValueRef<'scope, 'static>> {
        SimpleVectorData(self.unwrap_non_null(Private), PhantomData, PhantomData)
    }

    /// Access the contents of this `SimpleVector` as `U`.
    ///
    /// This method returns `JlrsError::AccessError` if `U` isn't correct for all elements.
    ///
    /// Safety: the lifetime borrow is not restricted by a frame.
    pub unsafe fn unrestricted_typed_data<U>(
        self,
    ) -> JlrsResult<SimpleVectorData<'scope, 'scope, U>>
    where
        U: WrapperRef<'scope, 'static>,
    {
        if !self.is_typed::<U>() {
            Err(AccessError::InvalidLayout {
                value_type_str: String::from("this SimpleVector"),
            })?;
        }

        Ok(SimpleVectorData(
            self.unwrap_non_null(Private),
            PhantomData,
            PhantomData,
        ))
    }

    /// Access the contents of this `SimpleVector` as `U`.
    ///
    /// This method returns `JlrsError::AccessError` if `U` isn't correct for all elements.
    ///
    /// Safety: this method doesn't check if `U` is correct for all elements, the lifetime borrow
    /// is not restricted by a frame.
    pub unsafe fn unrestricted_typed_data_unchecked<U>(self) -> SimpleVectorData<'scope, 'scope, U>
    where
        U: WrapperRef<'scope, 'static>,
    {
        SimpleVectorData(self.unwrap_non_null(Private), PhantomData, PhantomData)
    }

    fn is_typed<U: ValidLayout>(self) -> bool {
        // Safety: the pointer points to valid data
        unsafe {
            let len = self.unwrap_non_null(Private).as_ref().length;
            let ptr = self.unwrap_non_null(Private).as_ptr();
            let slice = std::slice::from_raw_parts(jl_svec_data(ptr).cast::<ValueRef>(), len);

            for element in slice.iter().copied() {
                match element.value() {
                    Some(value) => {
                        if !U::valid_layout(value.datatype().as_value()) {
                            return false;
                        }
                    }
                    None => (),
                }
            }
        }

        true
    }

    /// Returns the length of this `SimpleVector`.
    pub fn len(&self) -> usize {
        // Safety: the pointer points to valid data
        unsafe { self.0.as_ref().length }
    }
}

impl<'scope> SimpleVector<'scope> {
    /// Use the `Output` to extend the lifetime of this data.
    pub fn root<'target>(self, output: Output<'target>) -> SimpleVector<'target> {
        // Safety: the pointer points to valid data
        unsafe {
            let ptr = self.unwrap_non_null(Private);
            output.set_root::<SimpleVector>(ptr);
            SimpleVector::wrap_non_null(ptr, Private)
        }
    }
}

impl<'base> SimpleVector<'base> {
    /// The empty `SimpleVector`.
    pub fn emptysvec(_: Global<'base>) -> Self {
        // Safety: global constant
        unsafe { Self::wrap(jl_emptysvec, Private) }
    }
}

// Safety: if the type is jl_simplevector_type the data is an SimpleVector
unsafe impl<'scope> Typecheck for SimpleVector<'scope> {
    fn typecheck(t: DataType) -> bool {
        // Safety: can only be called from a thread known to Julia
        t == DataType::simplevector_type(unsafe { Global::new() })
    }
}

impl<'scope> Debug for SimpleVector<'scope> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self.display_string() {
            Ok(s) => write!(f, "{}", s),
            Err(e) => write!(f, "<Cannot display value: {}>", e),
        }
    }
}

impl<'scope> WrapperPriv<'scope, '_> for SimpleVector<'scope> {
    type Wraps = jl_svec_t;
    const NAME: &'static str = "SimpleVector";

    // Safety: `inner` must not have been freed yet, the result must never be
    // used after the GC might have freed it.
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

impl_root!(SimpleVector, 1);

/// A reference to a [`SimpleVector`] that has not been explicitly rooted.
pub type SimpleVectorRef<'scope> = Ref<'scope, 'static, SimpleVector<'scope>>;

unsafe impl<'scope> ValidLayout for SimpleVectorRef<'scope> {
    fn valid_layout(v: Value) -> bool {
        if let Ok(dt) = v.cast::<DataType>() {
            dt.is::<SimpleVector>()
        } else {
            false
        }
    }

    const IS_REF: bool = true;
}

impl<'scope> SimpleVectorRef<'scope> {
    /// Root this reference to a SimpleVector in `scope`.
    ///
    /// Safety: self must point to valid data.
    pub unsafe fn root<'target, S>(self, scope: S) -> JlrsResult<SimpleVector<'target>>
    where
        S: PartialScope<'target>,
    {
        <SimpleVector as Root>::root(scope, self)
    }
}
