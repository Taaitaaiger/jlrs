//! Wrapper for `SimpleVector`.

use crate::{
    error::{AccessError, JlrsResult},
    layout::{typecheck::Typecheck, valid_layout::ValidLayout},
    memory::{target::global::Global, target::Target},
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

/// Access and mutate the content of a `SimpleVector`.
#[repr(transparent)]
pub struct SimpleVectorContent<'scope, 'borrow, T = ValueRef<'scope, 'static>>(
    NonNull<jl_svec_t>,
    PhantomData<&'scope ()>,
    PhantomData<&'borrow [T]>,
)
where
    T: WrapperRef<'scope, 'static>;

impl<'scope, 'borrow, T: WrapperRef<'scope, 'static>> SimpleVectorContent<'scope, 'borrow, T> {
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
    pub fn with_capacity<T>(target: T, n: usize) -> SimpleVectorData<'scope, T>
    where
        T: Target<'scope>,
    {
        // Safety: the allocated data is immediately rooted
        unsafe {
            let svec = NonNull::new_unchecked(jl_alloc_svec(n));
            target.data_from_ptr(svec, Private)
        }
    }

    /// Create a new `SimpleVector` that can hold `n` values without initializing its contents.
    ///
    /// Safety: The contents must be set before calling Julia again, the contents must never be
    /// accessed before all elements are set.
    pub unsafe fn with_capacity_uninit<T>(target: T, n: usize) -> SimpleVectorData<'scope, T>
    where
        T: Target<'scope>,
    {
        let svec = NonNull::new_unchecked(jl_alloc_svec_uninit(n));
        target.data_from_ptr(svec, Private)
    }

    /// Access the contents of this `SimpleVector` as `ValueRef`.
    // TODO: ledger
    // TODO: mut
    pub fn data<'borrow>(&'borrow self) -> SimpleVectorContent<'scope, 'borrow> {
        SimpleVectorContent(self.unwrap_non_null(Private), PhantomData, PhantomData)
    }

    /// Access the contents of this `SimpleVector` as `U`.
    ///
    /// This method returns a `JlrsError::AccessError` if `U` isn't correct for all elements.
    // TODO: ledger
    // TODO: mut
    pub fn typed_data<'borrow, U>(
        &'borrow self,
    ) -> JlrsResult<SimpleVectorContent<'scope, 'borrow, U>>
    where
        U: WrapperRef<'scope, 'static>,
    {
        if !self.is_typed::<U>() {
            Err(AccessError::InvalidLayout {
                value_type: String::from("this SimpleVector"),
            })?;
        }

        Ok(SimpleVectorContent(
            self.unwrap_non_null(Private),
            PhantomData,
            PhantomData,
        ))
    }

    /// Access the contents of this `SimpleVector` as `U`.
    ///
    /// Safety: this method doesn't check if `U` is correct for all elements.
    pub unsafe fn typed_data_unchecked<'borrow, U>(
        &'borrow self,
    ) -> SimpleVectorContent<'scope, 'borrow, U>
    where
        U: WrapperRef<'scope, 'static>,
    {
        SimpleVectorContent(self.unwrap_non_null(Private), PhantomData, PhantomData)
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
    /// Use the target to reroot this data.
    pub fn root<'target, T>(self, target: T) -> SimpleVectorData<'target, T>
    where
        T: Target<'target>,
    {
        // Safety: the data is valid.
        unsafe { target.data_from_ptr(self.unwrap_non_null(Private), Private) }
    }
}

impl<'base> SimpleVector<'base> {
    /// The empty `SimpleVector`.
    pub fn emptysvec<T: Target<'base>>(_: &T) -> Self {
        // Safety: global constant
        unsafe { Self::wrap(jl_emptysvec, Private) }
    }
}

// Safety: if the type is jl_simplevector_type the data is an SimpleVector
unsafe impl<'scope> Typecheck for SimpleVector<'scope> {
    fn typecheck(t: DataType) -> bool {
        // Safety: can only be called from a thread known to Julia
        t == DataType::simplevector_type(unsafe { &Global::new() })
    }
}

impl<'scope> Debug for SimpleVector<'scope> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let s = self
            .display_string()
            .unwrap_or(String::from("<Cannot display value>"));
        write!(f, "{}", s)
    }
}

impl<'scope> WrapperPriv<'scope, '_> for SimpleVector<'scope> {
    type Wraps = jl_svec_t;
    type StaticPriv = SimpleVector<'static>;
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
    pub unsafe fn root<'target, T>(self, target: T) -> JlrsResult<SimpleVectorData<'target, T>>
    where
        T: Target<'target>,
    {
        <SimpleVector as Root>::root(target, self)
    }
}

use crate::memory::target::target_type::TargetType;
pub type SimpleVectorData<'target, T> =
    <T as TargetType<'target>>::Data<'static, SimpleVector<'target>>;
pub type SimpleVectorResult<'target, T> =
    <T as TargetType<'target>>::Result<'static, SimpleVector<'target>>;
