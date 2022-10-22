//! Targets for methods that return Julia data.
//!
//! Many methods in jlrs return Julia data, these methods use targets to ensure the returned data
//! has the correct type and appropriate lifetimes.
//!
//! Targets implement two traits, [`Target`] and [`ExceptionTarget`]. `Target` is used in
//! combination with methods that return `Data` or a `Result`. `Data` is simply some Julia data,
//! while `Result` is a `Result` that contains Julia data in both its `Ok` and `Err` variants;
//! if an `Err` is returned it contains a caught exception. An `ExceptionTarget` is used in
//! combintation with methods that can throw an exception, but typically don't return Julia data
//! on success. If an `ExceptionTarget` does return Julia data on success, the data is guaranteed
//! to be globally rooted.
//!
//! Targets don't guarantee the returned data is rooted, this depends on what target has been
//! used. The following targets currently exist, the `'scope` lifetime indicates the lifetime of
//! the returned data:
//!
//! | Type                              | Rooting |
//! |-----------------------------------|---------|
//! | `(Async)GcFrame<'scope>`          | Yes     |
//! | `&mut (Async)GcFrame<'scope>`     | Yes     |
//! | `Output<'scope>`                  | Yes     |
//! | `&'scope mut Output<'_>`          | Yes     |
//! | `Global<'scope>`                  | No      |
//! | `&<T: (Exception)Target<'scope>>` | No      |
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

use std::{future::Future, marker::PhantomData};

use crate::prelude::{JlrsResult, Value, Wrapper};

use self::private::{ExceptionTargetPriv, TargetPriv};

#[cfg(feature = "async")]
use self::frame::AsyncGcFrame;

use self::{frame::GcFrame, global::Global, output::Output};

pub mod frame;
pub mod global;
pub mod output;
pub mod target_type;

/// Trait implemented by all targets.
///
/// Whenever a function in jlrs returns new Julia data, it will take a target which implements
/// this trait. Every target implements [`TargetType`], which defines the type that is returned.
/// These functions return either `TargetType::Data` or `TargetType::Result`, the first is used
/// when exceptions aren't caught, while the second is used when they are caught.
///
/// For more information see the [module-level] docs
///  
/// [module-level]: self
pub trait Target<'target, 'data, W = Value<'target, 'data>>: TargetPriv<'target, 'data, W>
where
    W: Wrapper<'target, 'data>,
{
    /// Returns a new `Global`.
    fn global(&self) -> Global<'target> {
        unsafe { Global::new() }
    }

    /// Convert `self` to an `ExtendedTarget`.
    fn into_extended_target<'borrow, 'current>(
        self,
        frame: &'borrow mut GcFrame<'current>,
    ) -> ExtendedTarget<'target, 'current, 'borrow, 'data, Self, W> {
        ExtendedTarget {
            target: self,
            frame,
            _target_marker: PhantomData,
            _data_marker: PhantomData,
            _wrapper_marker: PhantomData,
        }
    }

    /// Convert `self` to an `ExtendedAsyncTarget`.
    #[cfg(feature = "async")]
    fn into_extended_async_target<'borrow, 'current>(
        self,
        frame: &'borrow mut AsyncGcFrame<'current>,
    ) -> ExtendedAsyncTarget<'target, 'current, 'borrow, 'data, Self, W> {
        ExtendedAsyncTarget {
            target: self,
            frame,
            _target_marker: PhantomData,
            _data_marker: PhantomData,
            _wrapper_marker: PhantomData,
        }
    }
}

/// Trait implemented by all targets.
///
/// This trait is similar to [`Target`], it's used with methods that don't return Julia data
/// on success, but do return a caught exception on failure.
///
/// For more information see the [module-level] docs
///  
/// [module-level]: self
pub trait ExceptionTarget<'target, 'data, W = ()>: ExceptionTargetPriv<'target, 'data, W> {
    /// Returns a new `Global`.
    fn global(&self) -> Global<'target> {
        unsafe { Global::new() }
    }

    /// Convert `self` to an `ExtendedExceptionTarget`.
    fn into_extended_exception_target<'borrow, 'current>(
        self,
        frame: &'borrow mut GcFrame<'current>,
    ) -> ExtendedExceptionTarget<'target, 'current, 'borrow, 'data, Self, W> {
        ExtendedExceptionTarget {
            target: self,
            frame,
            _target_marker: PhantomData,
            _data_marker: PhantomData,
            _wrapper_marker: PhantomData,
        }
    }
}

/// A `Target` that borrows a frame for temporary allocations.
pub struct ExtendedTarget<'target, 'current, 'borrow, 'data, T, W = Value<'target, 'data>>
where
    T: Target<'target, 'data, W>,
    W: Wrapper<'target, 'data>,
{
    pub(crate) target: T,
    pub(crate) frame: &'borrow mut GcFrame<'current>,
    pub(crate) _target_marker: PhantomData<&'target ()>,
    pub(crate) _data_marker: PhantomData<&'data ()>,
    pub(crate) _wrapper_marker: PhantomData<W>,
}

impl<'target, 'current, 'borrow, 'data, T, W>
    ExtendedTarget<'target, 'current, 'borrow, 'data, T, W>
where
    T: Target<'target, 'data, W>,
    W: Wrapper<'target, 'data>,
{
    /// Split the `ExtendedTarget` into its `Target` and `BorrowedFrame`
    pub fn split(self) -> (T, BorrowedFrame<'borrow, 'current, GcFrame<'current>>) {
        (self.target, BorrowedFrame(self.frame, PhantomData))
    }
}

/// An `ExceptionTarget` that borrows a frame for temporary allocations.
pub struct ExtendedExceptionTarget<'target, 'current, 'borrow, 'data, T, W = Value<'target, 'data>>
where
    T: ExceptionTarget<'target, 'data, W>,
{
    pub(crate) target: T,
    pub(crate) frame: &'borrow mut GcFrame<'current>,
    pub(crate) _target_marker: PhantomData<&'target ()>,
    pub(crate) _data_marker: PhantomData<&'data ()>,
    pub(crate) _wrapper_marker: PhantomData<W>,
}

impl<'target, 'current, 'borrow, 'data, T, W>
    ExtendedExceptionTarget<'target, 'current, 'borrow, 'data, T, W>
where
    T: ExceptionTarget<'target, 'data, W>,
{
    /// Split the `ExtendedExceptionTarget` into its `ExceptionTarget` and `BorrowedFrame`.
    pub fn split(self) -> (T, BorrowedFrame<'borrow, 'current, GcFrame<'current>>) {
        (self.target, BorrowedFrame(self.frame, PhantomData))
    }
}

#[cfg(feature = "async")]
/// A `Target` that borrows an async frame for temporary allocations.
pub struct ExtendedAsyncTarget<'target, 'current, 'borrow, 'data, T, W = Value<'target, 'data>>
where
    T: Target<'target, 'data, W>,
    W: Wrapper<'target, 'data>,
{
    pub(crate) target: T,
    pub(crate) frame: &'borrow mut AsyncGcFrame<'current>,
    pub(crate) _target_marker: PhantomData<&'target ()>,
    pub(crate) _data_marker: PhantomData<&'data ()>,
    pub(crate) _wrapper_marker: PhantomData<W>,
}

#[cfg(feature = "async")]
impl<'target, 'current, 'borrow, 'data, T, W>
    ExtendedAsyncTarget<'target, 'current, 'borrow, 'data, T, W>
where
    T: Target<'target, 'data, W>,
    W: Wrapper<'target, 'data>,
{
    /// Split the `ExtendedAsyncTarget` into its `Target` and `BorrowedFrame`
    pub fn split(self) -> (T, BorrowedFrame<'borrow, 'current, AsyncGcFrame<'current>>) {
        (self.target, BorrowedFrame(self.frame, PhantomData))
    }
}

// TODO: ExtendedExceptionTarget / ExtendedAsyncExceptionTarget
// TODO: Unify?

/// A frame that has been borrowed. A new scope must be created before it can be used as a target
/// again.
pub struct BorrowedFrame<'borrow, 'current, F: 'current>(&'borrow mut F, PhantomData<&'current ()>);

impl<'borrow, 'current> BorrowedFrame<'borrow, 'current, GcFrame<'current>> {
    /// Create a temporary scope by calling [`GcFrame::scope`].
    pub fn scope<T, F>(self, func: F) -> JlrsResult<T>
    where
        for<'inner> F: FnOnce(GcFrame<'inner>) -> JlrsResult<T>,
    {
        self.0.scope(func)
    }
}

#[cfg(feature = "async")]
impl<'borrow, 'current> BorrowedFrame<'borrow, 'current, AsyncGcFrame<'current>> {
    /// Create a temporary scope by calling [`GcFrame::scope`].
    pub fn scope<T, F>(self, func: F) -> JlrsResult<T>
    where
        for<'inner> F: FnOnce(GcFrame<'inner>) -> JlrsResult<T>,
    {
        self.0.scope(func)
    }

    /// Create a temporary scope by calling [`AsyncGcFrame::async_scope`].
    pub async fn async_scope<'nested, T, F, G>(self, func: F) -> JlrsResult<T>
    where
        'borrow: 'nested,
        T: 'current,
        G: Future<Output = JlrsResult<T>>,
        F: FnOnce(AsyncGcFrame<'nested>) -> G,
    {
        self.0.async_scope(func).await
    }

    /// Create a temporary scope by calling [`AsyncGcFrame::relaxed_async_scope`].
    pub async unsafe fn relaxed_async_scope<'nested, T, F, G>(self, func: F) -> JlrsResult<T>
    where
        'borrow: 'nested,
        T: 'nested,
        G: Future<Output = JlrsResult<T>>,
        F: FnOnce(AsyncGcFrame<'nested>) -> G,
    {
        self.0.relaxed_async_scope(func).await
    }
}

impl<'target, 'data, W> Target<'target, 'data, W> for GcFrame<'target> where
    W: Wrapper<'target, 'data>
{
}

impl<'target, 'data, W> Target<'target, 'data, W> for &mut GcFrame<'target> where
    W: Wrapper<'target, 'data>
{
}

#[cfg(feature = "async")]
impl<'target, 'data, W> Target<'target, 'data, W> for AsyncGcFrame<'target> where
    W: Wrapper<'target, 'data>
{
}

#[cfg(feature = "async")]
impl<'target, 'data, W> Target<'target, 'data, W> for &mut AsyncGcFrame<'target> where
    W: Wrapper<'target, 'data>
{
}

impl<'target, 'data, W> Target<'target, 'data, W> for Global<'target> where
    W: Wrapper<'target, 'data>
{
}

impl<'target, 'data, W> Target<'target, 'data, W> for Output<'target> where
    W: Wrapper<'target, 'data>
{
}

impl<'target, 'data, W> Target<'target, 'data, W> for &'target mut Output<'_> where
    W: Wrapper<'target, 'data>
{
}

impl<'target, 'data, W, T> Target<'target, 'data, W> for &T
where
    W: Wrapper<'target, 'data>,
    T: Target<'target, 'data, W>,
{
}

impl<'target, 'data, W> ExceptionTarget<'target, 'data, W> for GcFrame<'target> {}

impl<'target, 'data, W> ExceptionTarget<'target, 'data, W> for &mut GcFrame<'target> {}

#[cfg(feature = "async")]
impl<'target, 'data, W> ExceptionTarget<'target, 'data, W> for AsyncGcFrame<'target> {}

#[cfg(feature = "async")]
impl<'target, 'data, W> ExceptionTarget<'target, 'data, W> for &mut AsyncGcFrame<'target> {}

impl<'target, 'data, W> ExceptionTarget<'target, 'data, W> for Global<'target> {}

impl<'target, 'data, W> ExceptionTarget<'target, 'data, W> for Output<'target> {}

impl<'target, 'data, W> ExceptionTarget<'target, 'data, W> for &'target mut Output<'_> {}

impl<'target, 'data, W, T> ExceptionTarget<'target, 'data, W> for &T where
    T: ExceptionTarget<'target, 'data, W>
{
}

pub(crate) mod private {
    use std::ptr::NonNull;

    use jl_sys::jl_value_t;

    use crate::{
        prelude::{Value, ValueRef, Wrapper},
        private::Private,
        wrappers::ptr::{private::WrapperPriv, Ref},
    };

    use super::{
        global::Global,
        target_type::{ExceptionTargetType, TargetType},
        GcFrame, Output,
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

    impl<'target> TargetBase<'target> for Global<'target> {}

    impl<'target, T: TargetBase<'target>> TargetBase<'target> for &T {}

    pub trait TargetPriv<'target, 'data, W>:
        TargetBase<'target> + TargetType<'target, 'data, W>
    where
        W: Wrapper<'target, 'data>,
    {
        // Safety: the pointer must point to valid data.
        unsafe fn data_from_ptr(self, value: NonNull<W::Wraps>, _: Private) -> Self::Data;

        // Safety: the pointer must point to valid data.
        unsafe fn result_from_ptr(
            self,
            result: Result<NonNull<W::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> Self::Result;

        // Safety: the pointer must point to valid data.
        unsafe fn result_from_unrooted(
            self,
            result: Result<Ref<'target, 'data, W>, ValueRef<'target, 'data>>,
            _: Private,
        ) -> Self::Result {
            let result = match result {
                Ok(v) => Ok(NonNull::new_unchecked(v.ptr())),
                Err(e) => Err(NonNull::new_unchecked(e.ptr())),
            };

            self.result_from_ptr(result, Private)
        }

        // Safety: the pointer must point to valid data.
        unsafe fn result_from_rooted(
            self,
            result: Result<W, Value<'target, 'data>>,
            _: Private,
        ) -> Self::Result {
            let result = match result {
                Ok(v) => Ok(v.unwrap_non_null(Private)),
                Err(e) => Err(e.unwrap_non_null(Private)),
            };

            self.result_from_ptr(result, Private)
        }
    }

    impl<'target, 'data, W: Wrapper<'target, 'data>> TargetPriv<'target, 'data, W>
        for &mut GcFrame<'target>
    {
        // Safety: the pointer must point to valid data.
        unsafe fn data_from_ptr(self, value: NonNull<W::Wraps>, _: Private) -> Self::Data {
            self.root(value)
        }

        // Safety: the pointer must point to valid data.
        unsafe fn result_from_ptr(
            self,
            result: Result<NonNull<W::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> Self::Result {
            match result {
                Ok(t) => Ok(self.root(t)),
                Err(e) => Err(self.root(e)),
            }
        }
    }

    impl<'target, 'data, W: Wrapper<'target, 'data>> TargetPriv<'target, 'data, W>
        for GcFrame<'target>
    {
        // Safety: the pointer must point to valid data.
        unsafe fn data_from_ptr(self, value: NonNull<W::Wraps>, _: Private) -> Self::Data {
            self.root(value)
        }

        // Safety: the pointer must point to valid data.
        unsafe fn result_from_ptr(
            self,
            result: Result<NonNull<W::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> Self::Result {
            match result {
                Ok(t) => Ok(self.root(t)),
                Err(e) => Err(self.root(e)),
            }
        }
    }

    #[cfg(feature = "async")]
    impl<'target, 'data, W: Wrapper<'target, 'data>> TargetPriv<'target, 'data, W>
        for &mut AsyncGcFrame<'target>
    {
        // Safety: the pointer must point to valid data.
        unsafe fn data_from_ptr(self, value: NonNull<W::Wraps>, _: Private) -> Self::Data {
            self.root(value)
        }

        // Safety: the pointer must point to valid data.
        unsafe fn result_from_ptr(
            self,
            result: Result<NonNull<W::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> Self::Result {
            match result {
                Ok(t) => Ok(self.root(t)),
                Err(e) => Err(self.root(e)),
            }
        }
    }

    #[cfg(feature = "async")]
    impl<'target, 'data, W: Wrapper<'target, 'data>> TargetPriv<'target, 'data, W>
        for AsyncGcFrame<'target>
    {
        // Safety: the pointer must point to valid data.
        unsafe fn data_from_ptr(self, value: NonNull<W::Wraps>, _: Private) -> Self::Data {
            self.root(value)
        }

        // Safety: the pointer must point to valid data.
        unsafe fn result_from_ptr(
            self,
            result: Result<NonNull<W::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> Self::Result {
            match result {
                Ok(t) => Ok(self.root(t)),
                Err(e) => Err(self.root(e)),
            }
        }
    }

    impl<'target, 'data, W: Wrapper<'target, 'data>> TargetPriv<'target, 'data, W> for Output<'target> {
        // Safety: the pointer must point to valid data.
        unsafe fn data_from_ptr(self, value: NonNull<W::Wraps>, _: Private) -> Self::Data {
            self.consume(value)
        }

        // Safety: the pointer must point to valid data.
        unsafe fn result_from_ptr(
            self,
            result: Result<NonNull<W::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> Self::Result {
            match result {
                Ok(t) => Ok(self.consume(t)),
                Err(e) => Err(self.consume(e)),
            }
        }
    }

    impl<'target, 'data, W: Wrapper<'target, 'data>> TargetPriv<'target, 'data, W>
        for &'target mut Output<'_>
    {
        // Safety: the pointer must point to valid data.
        unsafe fn data_from_ptr(self, value: NonNull<W::Wraps>, _: Private) -> Self::Data {
            self.temporary(value)
        }

        // Safety: the pointer must point to valid data.
        unsafe fn result_from_ptr(
            self,
            result: Result<NonNull<W::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> Self::Result {
            match result {
                Ok(t) => Ok(self.temporary(t)),
                Err(e) => Err(self.temporary(e)),
            }
        }
    }

    impl<'target, 'data, W: Wrapper<'target, 'data>> TargetPriv<'target, 'data, W> for Global<'target> {
        // Safety: the pointer must point to valid data.
        unsafe fn data_from_ptr(self, value: NonNull<W::Wraps>, _: Private) -> Self::Data {
            Ref::wrap(value.as_ptr())
        }

        // Safety: the pointer must point to valid data.
        unsafe fn result_from_ptr(
            self,
            result: Result<NonNull<W::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> Self::Result {
            match result {
                Ok(t) => Ok(Ref::wrap(t.as_ptr())),
                Err(e) => Err(Ref::wrap(e.as_ptr())),
            }
        }
    }

    impl<'target, 'data, W: Wrapper<'target, 'data>, T: TargetPriv<'target, 'data, W>>
        TargetPriv<'target, 'data, W> for &T
    {
        // Safety: the pointer must point to valid data.
        unsafe fn data_from_ptr(self, value: NonNull<W::Wraps>, _: Private) -> Self::Data {
            Ref::wrap(value.as_ptr())
        }

        // Safety: the pointer must point to valid data.
        unsafe fn result_from_ptr(
            self,
            result: Result<NonNull<W::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> Self::Result {
            match result {
                Ok(t) => Ok(Ref::wrap(t.as_ptr())),
                Err(e) => Err(Ref::wrap(e.as_ptr())),
            }
        }
    }

    pub trait ExceptionTargetPriv<'target, 'data, W>:
        TargetBase<'target> + ExceptionTargetType<'target, 'data, W>
    {
        // Safety: the pointer must point to valid data.
        unsafe fn exception_from_ptr(
            self,
            result: Result<W, NonNull<jl_value_t>>,
            _: Private,
        ) -> Self::Exception;
    }

    impl<'target, 'data, W> ExceptionTargetPriv<'target, 'data, W> for &mut GcFrame<'target> {
        // Safety: the pointer must point to valid data.
        unsafe fn exception_from_ptr(
            self,
            result: Result<W, NonNull<jl_value_t>>,
            _: Private,
        ) -> Self::Exception {
            match result {
                Ok(t) => Ok(t),
                Err(e) => Err(self.root(e)),
            }
        }
    }

    impl<'target, 'data, W> ExceptionTargetPriv<'target, 'data, W> for GcFrame<'target> {
        // Safety: the pointer must point to valid data.
        unsafe fn exception_from_ptr(
            self,
            result: Result<W, NonNull<jl_value_t>>,
            _: Private,
        ) -> Self::Exception {
            match result {
                Ok(t) => Ok(t),
                Err(e) => Err(self.root(e)),
            }
        }
    }

    #[cfg(feature = "async")]
    impl<'target, 'data, W> ExceptionTargetPriv<'target, 'data, W> for &mut AsyncGcFrame<'target> {
        // Safety: the pointer must point to valid data.
        unsafe fn exception_from_ptr(
            self,
            result: Result<W, NonNull<jl_value_t>>,
            _: Private,
        ) -> Self::Exception {
            match result {
                Ok(t) => Ok(t),
                Err(e) => Err(self.root(e)),
            }
        }
    }

    #[cfg(feature = "async")]
    impl<'target, 'data, W> ExceptionTargetPriv<'target, 'data, W> for AsyncGcFrame<'target> {
        // Safety: the pointer must point to valid data.
        unsafe fn exception_from_ptr(
            self,
            result: Result<W, NonNull<jl_value_t>>,
            _: Private,
        ) -> Self::Exception {
            match result {
                Ok(t) => Ok(t),
                Err(e) => Err(self.root(e)),
            }
        }
    }

    impl<'target, 'data, W> ExceptionTargetPriv<'target, 'data, W> for Output<'target> {
        // Safety: the pointer must point to valid data.
        unsafe fn exception_from_ptr(
            self,
            result: Result<W, NonNull<jl_value_t>>,
            _: Private,
        ) -> Self::Exception {
            match result {
                Ok(t) => Ok(t),
                Err(e) => Err(self.consume(e)),
            }
        }
    }

    impl<'target, 'data, W> ExceptionTargetPriv<'target, 'data, W> for &'target mut Output<'_> {
        // Safety: the pointer must point to valid data.
        unsafe fn exception_from_ptr(
            self,
            result: Result<W, NonNull<jl_value_t>>,
            _: Private,
        ) -> Self::Exception {
            match result {
                Ok(t) => Ok(t),
                Err(e) => Err(self.temporary(e)),
            }
        }
    }

    impl<'target, 'data, W> ExceptionTargetPriv<'target, 'data, W> for Global<'target> {
        // Safety: the pointer must point to valid data.
        unsafe fn exception_from_ptr(
            self,
            result: Result<W, NonNull<jl_value_t>>,
            _: Private,
        ) -> Self::Exception {
            match result {
                Ok(t) => Ok(t),
                Err(e) => Err(Ref::wrap(e.as_ptr())),
            }
        }
    }

    impl<'target, 'data, W, T: ExceptionTargetPriv<'target, 'data, W>>
        ExceptionTargetPriv<'target, 'data, W> for &T
    {
        // Safety: the pointer must point to valid data.
        unsafe fn exception_from_ptr(
            self,
            result: Result<W, NonNull<jl_value_t>>,
            _: Private,
        ) -> Self::Exception {
            match result {
                Ok(t) => Ok(t),
                Err(e) => Err(Ref::wrap(e.as_ptr())),
            }
        }
    }
}
