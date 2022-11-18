//! Targets for methods that return Julia data.
//!
//! Many methods in jlrs return Julia data, these methods use targets to ensure the returned data
//! has the correct type and appropriate lifetimes.
//!
//! Targets implement the [`Target`] trait. This trait is used in
//! combination with methods that return `Data`, an `Exception` or a `Result`. `Data` is simply
//! some Julia data, `Exception` is a result that can contain Julia data in its `Err` variant,
//! and `Result` is a `Result` that contains Julia data in both its `Ok` and `Err` variants.
//! If an `Err` is returned it contains a caught exception. An `Exception` is used in
//! combination with methods that can throw an exception, but typically don't return Julia data
//! on success. If an `Exception` does contain Julia data on success, the data is guaranteed to be
//! globally rooted.
//!
//! Targets don't guarantee the returned data is rooted, this depends on what target has been
//! used. The following targets currently exist, the `'scope` lifetime indicates the lifetime of
//! the returned data:
//!
//! | Type                          | Rooting |
//! |-------------------------------|---------|
//! | `(Async)GcFrame<'scope>`      | Yes     |
//! | `&mut (Async)GcFrame<'scope>` | Yes     |
//! | `Output<'scope>`              | Yes     |
//! | `&'scope mut Output<'_>`      | Yes     |
//! | `ReusableSlot<'target>`       | Yes     |
//! | `&mut ReusableSlot<'target>`  | Yes     |
//! | `Global<'scope>`              | No      |
//! | `&<T: Target<'scope>>`        | No      |
//!
//!
//! The last row means that any target `T` can be used as a non-rooting target by using a
//! reference to that target. When a non-rooting target is used, Julia data is returned as a
//! [`Ref`] rather than a [`Wrapper`]. This is useful in cases where it can be guaranteed the
//! data is globally rooted, or if you don't care about the result. More information about these
//! target types can be found in the submodules that define them.
//!
//! Some targets can only be used to root a single value, methods that need to allocate temporary
//! data should use extended targets. An extended target can be split into a `BorrowedFrame` and
//! a target, the `BorrowedFrame` can be used to create a temporary scope and the target for the
//! data that is returned.
//!
//! [`Ref`]: crate::wrappers::ptr::Ref
//! [`Wrapper`]: crate::wrappers::ptr::Wrapper

use std::marker::PhantomData;

use self::{frame::BorrowedFrame, private::TargetPriv, reusable_slot::ReusableSlot};

#[cfg(feature = "async")]
use self::frame::AsyncGcFrame;

use self::{frame::GcFrame, global::Global, output::Output};

pub mod frame;
pub mod global;
pub mod output;
pub mod reusable_slot;
pub mod target_type;

/// Trait implemented by all targets.
///
/// Whenever a function in jlrs returns new Julia data, it will take a target which implements
/// this trait. Every target implements [`TargetType`], which defines the type that is returned.
/// These functions return either `TargetType::Data`, `TargetType::Exception` or
/// `TargetType::Result`, the first is used when exceptions aren't caught, while the second is
/// used when they are caught.
///
/// For more information see the [module-level] docs
///  
/// [module-level]: self
/// [`TargetType`]: crate::memory::target::target_type::TargetType
pub trait Target<'target>: TargetPriv<'target> {
    /// Returns a new `Global`.
    fn global(&self) -> Global<'target> {
        unsafe { Global::new() }
    }

    /// Convert `self` to an `ExtendedTarget`.
    fn into_extended_target<'borrow, 'current>(
        self,
        frame: &'borrow mut GcFrame<'current>,
    ) -> ExtendedTarget<'target, 'current, 'borrow, Self> {
        ExtendedTarget {
            target: self,
            frame,
            _target_marker: PhantomData,
        }
    }

    /// Convert `self` to an `ExtendedAsyncTarget`.
    #[cfg(feature = "async")]
    fn into_extended_async_target<'borrow, 'current>(
        self,
        frame: &'borrow mut AsyncGcFrame<'current>,
    ) -> ExtendedAsyncTarget<'target, 'current, 'borrow, Self> {
        ExtendedAsyncTarget {
            target: self,
            frame,
            _target_marker: PhantomData,
        }
    }
}

/// A `Target` that borrows a frame for temporary allocations.
pub struct ExtendedTarget<'target, 'current, 'borrow, T>
where
    T: Target<'target>,
{
    pub(crate) target: T,
    pub(crate) frame: &'borrow mut GcFrame<'current>,
    pub(crate) _target_marker: PhantomData<&'target ()>,
}

impl<'target, 'current, 'borrow, T> ExtendedTarget<'target, 'current, 'borrow, T>
where
    T: Target<'target>,
{
    /// Split the `ExtendedTarget` into its `Target` and `BorrowedFrame`
    pub fn split(self) -> (T, BorrowedFrame<'borrow, 'current, GcFrame<'current>>) {
        (self.target, BorrowedFrame(self.frame, PhantomData))
    }
}

#[cfg(feature = "async")]
/// A `Target` that borrows an async frame for temporary allocations.
pub struct ExtendedAsyncTarget<'target, 'current, 'borrow, T>
where
    T: Target<'target>,
{
    pub(crate) target: T,
    pub(crate) frame: &'borrow mut AsyncGcFrame<'current>,
    pub(crate) _target_marker: PhantomData<&'target ()>,
}

#[cfg(feature = "async")]
impl<'target, 'current, 'borrow, T> ExtendedAsyncTarget<'target, 'current, 'borrow, T>
where
    T: Target<'target>,
{
    /// Split the `ExtendedAsyncTarget` into its `Target` and `BorrowedFrame`
    pub fn split(self) -> (T, BorrowedFrame<'borrow, 'current, AsyncGcFrame<'current>>) {
        (self.target, BorrowedFrame(self.frame, PhantomData))
    }
}

impl<'target> Target<'target> for GcFrame<'target> {}

impl<'target> Target<'target> for &mut GcFrame<'target> {}

#[cfg(feature = "async")]
impl<'target> Target<'target> for AsyncGcFrame<'target> {}

#[cfg(feature = "async")]
impl<'target> Target<'target> for &mut AsyncGcFrame<'target> {}

impl<'target> Target<'target> for Global<'target> {}

impl<'target> Target<'target> for Output<'target> {}

impl<'target> Target<'target> for &'target mut Output<'_> {}

impl<'target> Target<'target> for ReusableSlot<'target> {}

impl<'target> Target<'target> for &mut ReusableSlot<'target> {}

impl<'target, 'data, T> Target<'target> for &T where T: Target<'target> {}

pub(crate) mod private {
    use std::ptr::NonNull;

    use jl_sys::jl_value_t;

    use crate::{
        private::Private,
        wrappers::ptr::{
            private::WrapperPriv,
            value::{Value, ValueRef},
            Ref, Wrapper,
        },
    };

    use super::{
        global::Global, reusable_slot::ReusableSlot, target_type::TargetType, GcFrame, Output,
    };

    #[cfg(feature = "async")]
    use super::AsyncGcFrame;

    pub trait TargetBase<'target>: Sized {}

    impl<'target> TargetBase<'target> for &mut GcFrame<'target> {}

    impl<'target> TargetBase<'target> for GcFrame<'target> {}

    #[cfg(feature = "async")]
    impl<'target> TargetBase<'target> for &mut AsyncGcFrame<'target> {}

    #[cfg(feature = "async")]
    impl<'target> TargetBase<'target> for AsyncGcFrame<'target> {}

    impl<'target> TargetBase<'target> for Output<'target> {}

    impl<'target> TargetBase<'target> for &'target mut Output<'_> {}

    impl<'target> TargetBase<'target> for ReusableSlot<'target> {}

    impl<'target> TargetBase<'target> for &mut ReusableSlot<'target> {}

    impl<'target> TargetBase<'target> for Global<'target> {}

    impl<'target, T: TargetBase<'target>> TargetBase<'target> for &T {}

    pub trait TargetPriv<'target>: TargetType<'target> {
        // Safety: the pointer must point to valid data.
        unsafe fn data_from_ptr<'data, T: Wrapper<'target, 'data>>(
            self,
            value: NonNull<T::Wraps>,
            _: Private,
        ) -> Self::Data<'data, T>;

        // Safety: the pointer must point to valid data.
        unsafe fn result_from_ptr<'data, T: Wrapper<'target, 'data>>(
            self,
            result: Result<NonNull<T::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> Self::Result<'data, T>;

        // Safety: the pointer must point to valid data.
        unsafe fn result_from_unrooted<'data, T: Wrapper<'target, 'data>>(
            self,
            result: Result<Ref<'target, 'data, T>, ValueRef<'target, 'data>>,
            _: Private,
        ) -> Self::Result<'data, T> {
            let result = match result {
                Ok(v) => Ok(v.ptr()),
                Err(e) => Err(e.ptr()),
            };

            self.result_from_ptr(result, Private)
        }

        // Safety: the pointer must point to valid data.
        unsafe fn result_from_rooted<'data, T: Wrapper<'target, 'data>>(
            self,
            result: Result<T, Value<'target, 'data>>,
            _: Private,
        ) -> Self::Result<'data, T> {
            let result = match result {
                Ok(v) => Ok(v.unwrap_non_null(Private)),
                Err(e) => Err(e.unwrap_non_null(Private)),
            };

            self.result_from_ptr(result, Private)
        }

        // Safety: the pointer must point to valid data.
        unsafe fn exception_from_ptr<'data, T>(
            self,
            result: Result<T, NonNull<jl_value_t>>,
            _: Private,
        ) -> Self::Exception<'data, T>;
    }

    impl<'target> TargetPriv<'target> for &mut GcFrame<'target> {
        // Safety: the pointer must point to valid data.
        unsafe fn data_from_ptr<'data, T: Wrapper<'target, 'data>>(
            self,
            value: NonNull<T::Wraps>,
            _: Private,
        ) -> Self::Data<'data, T> {
            self.root(value)
        }

        // Safety: the pointer must point to valid data.
        unsafe fn result_from_ptr<'data, T: Wrapper<'target, 'data>>(
            self,
            result: Result<NonNull<T::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> Self::Result<'data, T> {
            match result {
                Ok(t) => Ok(self.root(t)),
                Err(e) => Err(self.root(e)),
            }
        }

        // Safety: the pointer must point to valid data.
        unsafe fn exception_from_ptr<'data, T>(
            self,
            result: Result<T, NonNull<jl_value_t>>,
            _: Private,
        ) -> Self::Exception<'data, T> {
            match result {
                Ok(t) => Ok(t),
                Err(e) => Err(self.root(e)),
            }
        }
    }

    impl<'target> TargetPriv<'target> for GcFrame<'target> {
        // Safety: the pointer must point to valid data.
        unsafe fn data_from_ptr<'data, T: Wrapper<'target, 'data>>(
            self,
            value: NonNull<T::Wraps>,
            _: Private,
        ) -> Self::Data<'data, T> {
            self.root(value)
        }

        // Safety: the pointer must point to valid data.
        unsafe fn result_from_ptr<'data, T: Wrapper<'target, 'data>>(
            self,
            result: Result<NonNull<T::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> Self::Result<'data, T> {
            match result {
                Ok(t) => Ok(self.root(t)),
                Err(e) => Err(self.root(e)),
            }
        }

        // Safety: the pointer must point to valid data.
        unsafe fn exception_from_ptr<'data, T>(
            self,
            result: Result<T, NonNull<jl_value_t>>,
            _: Private,
        ) -> Self::Exception<'data, T> {
            match result {
                Ok(t) => Ok(t),
                Err(e) => Err(self.root(e)),
            }
        }
    }

    #[cfg(feature = "async")]
    impl<'target> TargetPriv<'target> for &mut AsyncGcFrame<'target> {
        // Safety: the pointer must point to valid data.
        unsafe fn data_from_ptr<'data, T: Wrapper<'target, 'data>>(
            self,
            value: NonNull<T::Wraps>,
            _: Private,
        ) -> Self::Data<'data, T> {
            self.root(value)
        }

        // Safety: the pointer must point to valid data.
        unsafe fn result_from_ptr<'data, T: Wrapper<'target, 'data>>(
            self,
            result: Result<NonNull<T::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> Self::Result<'data, T> {
            match result {
                Ok(t) => Ok(self.root(t)),
                Err(e) => Err(self.root(e)),
            }
        }

        // Safety: the pointer must point to valid data.
        unsafe fn exception_from_ptr<'data, T>(
            self,
            result: Result<T, NonNull<jl_value_t>>,
            _: Private,
        ) -> Self::Exception<'data, T> {
            match result {
                Ok(t) => Ok(t),
                Err(e) => Err(self.root(e)),
            }
        }
    }

    #[cfg(feature = "async")]
    impl<'target> TargetPriv<'target> for AsyncGcFrame<'target> {
        // Safety: the pointer must point to valid data.
        unsafe fn data_from_ptr<'data, T: Wrapper<'target, 'data>>(
            self,
            value: NonNull<T::Wraps>,
            _: Private,
        ) -> Self::Data<'data, T> {
            self.root(value)
        }

        // Safety: the pointer must point to valid data.
        unsafe fn result_from_ptr<'data, T: Wrapper<'target, 'data>>(
            self,
            result: Result<NonNull<T::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> Self::Result<'data, T> {
            match result {
                Ok(t) => Ok(self.root(t)),
                Err(e) => Err(self.root(e)),
            }
        }

        // Safety: the pointer must point to valid data.
        unsafe fn exception_from_ptr<'data, T>(
            self,
            result: Result<T, NonNull<jl_value_t>>,
            _: Private,
        ) -> Self::Exception<'data, T> {
            match result {
                Ok(t) => Ok(t),
                Err(e) => Err(self.root(e)),
            }
        }
    }

    impl<'target> TargetPriv<'target> for Output<'target> {
        // Safety: the pointer must point to valid data.
        unsafe fn data_from_ptr<'data, T: Wrapper<'target, 'data>>(
            self,
            value: NonNull<T::Wraps>,
            _: Private,
        ) -> Self::Data<'data, T> {
            self.consume(value)
        }

        // Safety: the pointer must point to valid data.
        unsafe fn result_from_ptr<'data, T: Wrapper<'target, 'data>>(
            self,
            result: Result<NonNull<T::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> Self::Result<'data, T> {
            match result {
                Ok(t) => Ok(self.consume(t)),
                Err(e) => Err(self.consume(e)),
            }
        }

        // Safety: the pointer must point to valid data.
        unsafe fn exception_from_ptr<'data, T>(
            self,
            result: Result<T, NonNull<jl_value_t>>,
            _: Private,
        ) -> Self::Exception<'data, T> {
            match result {
                Ok(t) => Ok(t),
                Err(e) => Err(self.consume(e)),
            }
        }
    }

    impl<'target> TargetPriv<'target> for &'target mut Output<'_> {
        // Safety: the pointer must point to valid data.
        unsafe fn data_from_ptr<'data, T: Wrapper<'target, 'data>>(
            self,
            value: NonNull<T::Wraps>,
            _: Private,
        ) -> Self::Data<'data, T> {
            self.temporary(value)
        }

        // Safety: the pointer must point to valid data.
        unsafe fn result_from_ptr<'data, T: Wrapper<'target, 'data>>(
            self,
            result: Result<NonNull<T::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> Self::Result<'data, T> {
            match result {
                Ok(t) => Ok(self.temporary(t)),
                Err(e) => Err(self.temporary(e)),
            }
        }

        // Safety: the pointer must point to valid data.
        unsafe fn exception_from_ptr<'data, T>(
            self,
            result: Result<T, NonNull<jl_value_t>>,
            _: Private,
        ) -> Self::Exception<'data, T> {
            match result {
                Ok(t) => Ok(t),
                Err(e) => Err(self.temporary(e)),
            }
        }
    }

    impl<'target> TargetPriv<'target> for ReusableSlot<'target> {
        // Safety: the pointer must point to valid data.
        unsafe fn data_from_ptr<'data, T: Wrapper<'target, 'data>>(
            self,
            value: NonNull<T::Wraps>,
            _: Private,
        ) -> Self::Data<'data, T> {
            self.consume(value)
        }

        // Safety: the pointer must point to valid data.
        unsafe fn result_from_ptr<'data, T: Wrapper<'target, 'data>>(
            self,
            result: Result<NonNull<T::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> Self::Result<'data, T> {
            match result {
                Ok(t) => Ok(self.consume(t)),
                Err(e) => Err(self.consume(e)),
            }
        }

        // Safety: the pointer must point to valid data.
        unsafe fn exception_from_ptr<'data, T>(
            self,
            result: Result<T, NonNull<jl_value_t>>,
            _: Private,
        ) -> Self::Exception<'data, T> {
            match result {
                Ok(t) => Ok(t),
                Err(e) => Err(self.consume(e)),
            }
        }
    }

    impl<'target> TargetPriv<'target> for &mut ReusableSlot<'target> {
        // Safety: the pointer must point to valid data.
        unsafe fn data_from_ptr<'data, T: Wrapper<'target, 'data>>(
            self,
            value: NonNull<T::Wraps>,
            _: Private,
        ) -> Self::Data<'data, T> {
            self.temporary(value)
        }

        // Safety: the pointer must point to valid data.
        unsafe fn result_from_ptr<'data, T: Wrapper<'target, 'data>>(
            self,
            result: Result<NonNull<T::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> Self::Result<'data, T> {
            match result {
                Ok(t) => Ok(self.temporary(t)),
                Err(e) => Err(self.temporary(e)),
            }
        }

        // Safety: the pointer must point to valid data.
        unsafe fn exception_from_ptr<'data, T>(
            self,
            result: Result<T, NonNull<jl_value_t>>,
            _: Private,
        ) -> Self::Exception<'data, T> {
            match result {
                Ok(t) => Ok(t),
                Err(e) => Err(self.temporary(e)),
            }
        }
    }

    impl<'target> TargetPriv<'target> for Global<'target> {
        // Safety: the pointer must point to valid data.
        unsafe fn data_from_ptr<'data, T: Wrapper<'target, 'data>>(
            self,
            value: NonNull<T::Wraps>,
            _: Private,
        ) -> Self::Data<'data, T> {
            Ref::wrap(value)
        }

        // Safety: the pointer must point to valid data.
        unsafe fn result_from_ptr<'data, T: Wrapper<'target, 'data>>(
            self,
            result: Result<NonNull<T::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> Self::Result<'data, T> {
            match result {
                Ok(t) => Ok(Ref::wrap(t)),
                Err(e) => Err(Ref::wrap(e)),
            }
        }

        // Safety: the pointer must point to valid data.
        unsafe fn exception_from_ptr<'data, T>(
            self,
            result: Result<T, NonNull<jl_value_t>>,
            _: Private,
        ) -> Self::Exception<'data, T> {
            match result {
                Ok(t) => Ok(t),
                Err(e) => Err(Ref::wrap(e)),
            }
        }
    }

    impl<'target, U: TargetPriv<'target>> TargetPriv<'target> for &U {
        // Safety: the pointer must point to valid data.
        unsafe fn data_from_ptr<'data, T: Wrapper<'target, 'data>>(
            self,
            value: NonNull<T::Wraps>,
            _: Private,
        ) -> Self::Data<'data, T> {
            Ref::wrap(value)
        }

        // Safety: the pointer must point to valid data.
        unsafe fn result_from_ptr<'data, T: Wrapper<'target, 'data>>(
            self,
            result: Result<NonNull<T::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> Self::Result<'data, T> {
            match result {
                Ok(t) => Ok(Ref::wrap(t)),
                Err(e) => Err(Ref::wrap(e)),
            }
        }

        // Safety: the pointer must point to valid data.
        unsafe fn exception_from_ptr<'data, T>(
            self,
            result: Result<T, NonNull<jl_value_t>>,
            _: Private,
        ) -> Self::Exception<'data, T> {
            match result {
                Ok(t) => Ok(t),
                Err(e) => Err(Ref::wrap(e)),
            }
        }
    }
}
