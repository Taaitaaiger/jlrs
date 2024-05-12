//! Managed type for `SimpleVector`.

use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
    ptr::{null_mut, NonNull},
};

use jl_sys::{
    jl_alloc_svec, jl_alloc_svec_uninit, jl_emptysvec, jl_simplevector_type, jl_svec_copy,
    jl_svec_t, jlrs_svec_data, jlrs_svec_len, jlrs_svecref, jlrs_svecset,
};

use super::{datatype::DataType, private::ManagedPriv, AtomicSlice, Managed, ManagedData, Ref};
use crate::{
    data::{
        layout::valid_layout::{ValidField, ValidLayout},
        managed::value::Value,
        types::typecheck::Typecheck,
    },
    error::{AccessError, JlrsResult},
    memory::target::{unrooted::Unrooted, Target, TargetResult},
    private::Private,
};

/// Access the content of a `SimpleVector`.
#[repr(transparent)]
pub struct SimpleVectorAccessor<'borrow, T = Value<'borrow, 'static>>(
    SimpleVector<'borrow>,
    PhantomData<&'borrow AtomicSlice<'borrow, 'static, T, SimpleVector<'borrow>>>,
)
where
    T: Managed<'borrow, 'static>;

impl<'borrow, T> SimpleVectorAccessor<'borrow, T>
where
    T: Managed<'borrow, 'static>,
{
    /// Returns the length of this `SimpleVector`.
    #[inline]
    pub fn len(&self) -> usize {
        // Safety: the pointer points to valid data
        unsafe { jlrs_svec_len(self.0.unwrap(Private)) }
    }

    /// Get the element at `index`. Returns `None` if the field is undefined or out-of-bounds.
    pub fn get<'target, Tgt>(
        &self,
        target: Tgt,
        index: usize,
    ) -> Option<ManagedData<'target, 'static, Tgt, T::InScope<'target>>>
    where
        Tgt: Target<'target>,
    {
        if index >= self.len() {
            return None;
        }

        unsafe {
            let v = jlrs_svecref(self.0.unwrap(Private).cast(), index);
            if v.is_null() {
                None
            } else {
                let v = Value::wrap_non_null(NonNull::new_unchecked(v), Private)
                    .cast_unchecked::<T>()
                    .root(target);

                Some(v)
            }
        }
    }

    /// Set the element at `index` to `value`. This is only safe if the `SimpleVector` has just
    /// been allocated.
    ///
    /// Safety: you should only mutate a `SimpleVector` after creating it, they should generally be
    /// considered immutable.
    pub unsafe fn set(&self, index: usize, value: Option<T::InScope<'_>>) -> JlrsResult<()> {
        if index >= self.len() {
            Err(AccessError::OutOfBoundsSVec {
                idx: index,
                len: self.len(),
            })?
        }

        let v = match value {
            Some(v) => v.unwrap(Private).cast(),
            None => null_mut(),
        };

        jlrs_svecset(self.0.unwrap(Private).cast(), index as _, v);
        Ok(())
    }

    /// Returns the content of this `SimpleVector` as an `AtomicSlice`.
    #[inline]
    pub fn as_atomic_slice<'b>(
        &'b self,
    ) -> AtomicSlice<'b, 'static, T::InScope<'b>, SimpleVector<'b>> {
        // Safety: the C API function is called with valid data
        let slice = unsafe {
            let data = jlrs_svec_data(self.0.unwrap(Private)).cast();
            std::slice::from_raw_parts(data, self.len())
        };

        AtomicSlice::new(self.0, slice)
    }
}

/// A `SimpleVector` is a fixed-size array that contains `ValueRef`s.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct SimpleVector<'scope>(NonNull<jl_svec_t>, PhantomData<&'scope ()>);

impl<'scope> SimpleVector<'scope> {
    /// Create a new `SimpleVector` that can hold `n` values.
    #[inline]
    pub fn with_capacity<Tgt>(target: Tgt, n: usize) -> SimpleVectorData<'scope, Tgt>
    where
        Tgt: Target<'scope>,
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
    #[inline]
    pub unsafe fn with_capacity_uninit<Tgt>(target: Tgt, n: usize) -> SimpleVectorData<'scope, Tgt>
    where
        Tgt: Target<'scope>,
    {
        let svec = NonNull::new_unchecked(jl_alloc_svec_uninit(n));
        target.data_from_ptr(svec, Private)
    }

    /// Copy an existing `SimpleVector`.
    #[inline]
    pub fn copy<'target, Tgt>(self, target: Tgt) -> SimpleVectorData<'target, Tgt>
    where
        Tgt: Target<'target>,
    {
        unsafe {
            let svec = jl_svec_copy(self.unwrap(Private));
            let svec = SimpleVector::wrap_non_null(NonNull::new_unchecked(svec), Private);
            svec.root(target)
        }
    }

    /// Immutably access the contents of this `SimpleVector`.
    #[inline]
    pub fn data<'borrow>(&'borrow self) -> SimpleVectorAccessor<'borrow> {
        SimpleVectorAccessor(*self, PhantomData)
    }

    /// Access the contents of this `SimpleVector` as `U`.
    ///
    /// Safety: this method doesn't check if `U` is correct for all elements.
    #[inline]
    pub unsafe fn typed_data_unchecked<'borrow, U>(
        &'borrow self,
    ) -> SimpleVectorAccessor<'borrow, U>
    where
        U: Managed<'borrow, 'static>,
    {
        SimpleVectorAccessor(*self, PhantomData)
    }

    /// Returns the length of this `SimpleVector`.
    #[inline]
    pub fn len(&self) -> usize {
        // Safety: the pointer points to valid data
        unsafe { jlrs_svec_len(self.0.as_ptr()) }
    }
}

impl<'base> SimpleVector<'base> {
    /// The empty `SimpleVector`.
    #[inline]
    pub fn emptysvec<Tgt: Target<'base>>(_: &Tgt) -> Self {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_emptysvec), Private) }
    }
}

// Safety: if the type is jl_simplevector_type the data is an SimpleVector
unsafe impl<'scope> Typecheck for SimpleVector<'scope> {
    #[inline]
    fn typecheck(t: DataType) -> bool {
        // Safety: can only be called from a thread known to Julia
        t == DataType::simplevector_type(unsafe { &Unrooted::new() })
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

impl<'scope> ManagedPriv<'scope, '_> for SimpleVector<'scope> {
    type Wraps = jl_svec_t;
    type WithLifetimes<'target, 'da> = SimpleVector<'target>;
    const NAME: &'static str = "SimpleVector";

    // Safety: `inner` must not have been freed yet, the result must never be
    // used after the GC might have freed it.
    #[inline]
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    #[inline]
    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

impl_construct_type_managed!(SimpleVector, 1, jl_simplevector_type);

/// A reference to a [`SimpleVector`] that has not been explicitly rooted.
pub type SimpleVectorRef<'scope> = Ref<'scope, 'static, SimpleVector<'scope>>;

/// A [`SimpleVectorRef`] with static lifetimes. This is a useful shorthand for signatures of
/// `ccall`able functions that return a [`SimpleVector`].
pub type SimpleVectorRet = Ref<'static, 'static, SimpleVector<'static>>;

unsafe impl<'scope> ValidLayout for SimpleVectorRef<'scope> {
    #[inline]
    fn valid_layout(v: Value) -> bool {
        if v.is::<DataType>() {
            let dt = unsafe { v.cast_unchecked::<DataType>() };
            dt.is::<SimpleVector>()
        } else {
            false
        }
    }

    #[inline]
    fn type_object<'target, Tgt: Target<'target>>(target: &Tgt) -> Value<'target, 'static> {
        DataType::simplevector_type(target).as_value()
    }

    const IS_REF: bool = true;
}

unsafe impl<'scope> ValidField for Option<SimpleVectorRef<'scope>> {
    #[inline]
    fn valid_field(v: Value) -> bool {
        if v.is::<DataType>() {
            let dt = unsafe { v.cast_unchecked::<DataType>() };
            dt.is::<SimpleVector>()
        } else {
            false
        }
    }
}

use crate::memory::target::TargetType;

/// `SimpleVector` or `SimpleVectorRef`, depending on the target type `Tgt`.
pub type SimpleVectorData<'target, Tgt> =
    <Tgt as TargetType<'target>>::Data<'static, SimpleVector<'target>>;

/// `JuliaResult<SimpleVector>` or `JuliaResultRef<SimpleVectorRef>`, depending on the target type
/// `Tgt`.
pub type SimpleVectorResult<'target, Tgt> =
    TargetResult<'target, 'static, SimpleVector<'target>, Tgt>;

impl_ccall_arg_managed!(SimpleVector, 1);
impl_into_typed!(SimpleVector);
