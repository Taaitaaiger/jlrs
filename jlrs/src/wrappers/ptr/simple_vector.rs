//! Wrapper for `SimpleVector`.

use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
    ptr::NonNull,
};

use jl_sys::{
    jl_alloc_svec,
    jl_alloc_svec_uninit,
    jl_emptysvec,
    jl_gc_wb,
    jl_svec_data,
    jl_svec_t,
};

use super::{
    datatype::DataType,
    private::WrapperPriv,
    value::ValueRef,
    Ref,
    Wrapper,
    WrapperRef,
    WrapperType,
};
use crate::{
    error::{AccessError, JlrsResult},
    layout::{
        typecheck::Typecheck,
        valid_layout::{ValidField, ValidLayout},
    },
    memory::target::{unrooted::Unrooted, Target},
    private::Private,
    wrappers::ptr::value::Value,
};

/// Access and mutate the content of a `SimpleVector`.
#[repr(transparent)]
pub struct SimpleVectorContent<'scope, 'borrow, T = ValueRef<'scope, 'static>>(
    NonNull<jl_svec_t>,
    PhantomData<&'scope ()>,
    PhantomData<&'borrow [Option<T>]>,
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
    pub fn as_slice(&self) -> &'borrow [Option<T>] {
        // Safety: the C API function is called with valid data
        unsafe { std::slice::from_raw_parts(jl_svec_data(self.0.as_ptr()).cast(), self.len()) }
    }

    /// Set the element at `index` to `value`. This is only safe if the `SimpleVector` has just
    /// been allocated.
    ///
    /// Safety: you may only mutate a `SimpleVector` after creating it, they should generally be
    /// considered immutable.
    pub unsafe fn set(
        &mut self,
        index: usize,
        value: Option<WrapperType<'_, 'scope, 'static, T>>,
    ) -> JlrsResult<()> {
        if index >= self.len() {
            Err(AccessError::OutOfBoundsSVec {
                idx: index,
                len: self.len(),
            })?
        }

        jl_svec_data(self.0.as_ptr())
            .cast::<Option<<T::Wrapper as Wrapper<'scope, 'static>>::TypeConstructor<'_, 'static>>>(
            )
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
        Option<U>: ValidField,
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

    fn is_typed<U>(self) -> bool
    where
        U: WrapperRef<'scope, 'static>,
        Option<U>: ValidField,
    {
        // Safety: the pointer points to valid data
        unsafe {
            let len = self.unwrap_non_null(Private).as_ref().length;
            let ptr = self.unwrap_non_null(Private).as_ptr();
            let slice =
                std::slice::from_raw_parts(jl_svec_data(ptr).cast::<Option<ValueRef>>(), len);

            for element in slice.iter().copied() {
                match element {
                    Some(value) => {
                        let value = value.value();
                        if !Option::<U>::valid_field(value.datatype().as_value()) {
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

impl<'base> SimpleVector<'base> {
    /// The empty `SimpleVector`.
    pub fn emptysvec<T: Target<'base>>(_: &T) -> Self {
        // Safety: global constant
        unsafe { Self::wrap_non_null(NonNull::new_unchecked(jl_emptysvec), Private) }
    }
}

// Safety: if the type is jl_simplevector_type the data is an SimpleVector
unsafe impl<'scope> Typecheck for SimpleVector<'scope> {
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

impl<'scope> WrapperPriv<'scope, '_> for SimpleVector<'scope> {
    type Wraps = jl_svec_t;
    type TypeConstructorPriv<'target, 'da> = SimpleVector<'target>;
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

unsafe impl<'scope> ValidField for Option<SimpleVectorRef<'scope>> {
    fn valid_field(v: Value) -> bool {
        if let Ok(dt) = v.cast::<DataType>() {
            dt.is::<SimpleVector>()
        } else {
            false
        }
    }
}

impl<'scope> SimpleVectorRef<'scope> {
    /// Root this reference to a SimpleVector in `scope`.
    ///
    /// Safety: self must point to valid data.
    pub unsafe fn root<'target, T>(self, target: T) -> SimpleVectorData<'target, T>
    where
        T: Target<'target>,
    {
        target.data_from_ptr(self.ptr(), Private)
    }
}

use crate::memory::target::target_type::TargetType;

/// `SimpleVector` or `SimpleVectorRef`, depending on the target type `T`.
pub type SimpleVectorData<'target, T> =
    <T as TargetType<'target>>::Data<'static, SimpleVector<'target>>;

/// `JuliaResult<SimpleVector>` or `JuliaResultRef<SimpleVectorRef>`, depending on the target type
/// `T`.
pub type SimpleVectorResult<'target, T> =
    <T as TargetType<'target>>::Result<'static, SimpleVector<'target>>;
