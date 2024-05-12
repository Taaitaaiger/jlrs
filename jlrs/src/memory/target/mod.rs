//! Frames, outputs and other targets.
//!
//! As explained in the [`memory`] module, functions that return Julia data take a target. All
//! targets implement the [`Target`] trait, this trait has a lifetime which encodes how long the
//! data created with this target can be used.
//!
//! There are two different kinds of target, rooting and non-rooting targets. A rooting target
//! guarantees the returned data is rooted while it can can be used, a non-rooting target doesn't
//! root the returned data at all. jlrs distinguishes between data that has been explicitly rooted
//! or not at the type level: rooted data is represented by types that implement the [`Managed`]
//! trait, while non-rooted data is represented as a [`Ref`].
//!
//! All targets define whether they are rooting or non-rooting targets by implementing
//! [`TargetType`]. This trait has a generic associated type: [`TargetType::Data`]. This type
//! is a [`Managed`] type if the target is a rooting target, and a [`Ref`] if it's non-rooting.
//! There are also the [`TargetResult`] and [`TargetException`] type aliases, which are `Result`s
//! that contain [`TargetType::Data`] in at least on of their variants.
//!
//! `Target::Data` is returned by functions that don't catch any exceptions. An example of such a
//! function is [`Value::new`], if you call that function with a rooting target it returns a
//! [`Value`], otherwise it returns a [`ValueRef`].
//!
//! `TargetResult` is used when exceptions are caught. An example is calling Julia functions
//! with the methods of the [`Call`] trait. These methods return a `Result`, the `Ok` variant
//! contains the same type as `Target::Data`, the `Err` variant is a `Value` or `ValueRef`
//! depending on the target.
//!
//! `TargetException` is used when exceptions are caught but the function doesn't need to return
//! Julia data on success. This is used by functions that call a function that can throw, but
//! don't return Julia data on success. Like `TargetResult` it's a `Result`, but can contain
//! arbitrary data in its `Ok` variant.
//!
//! All managed types provide type aliases for `Target::Data` and `TargetResult`, their names
//! are simply the name of the type itself and `Data` or `Result`. For example, `Value` provides
//! the aliases [`ValueData`] and [`ValueResult`]. It's generally significantly less verbose to
//! use these type aliases than expressing the return type with the associated type of the target,
//! and doing so clarifies what type of data is returned and whether you might need to handle a
//! caught exception or not.
//!
//! Rooting targets can be divided into three categories: frames, outputs, and reusable slots.
//! Frames form the backbone, they can have multiple slots that can hold one root; outputs and
//! reusable slots reserve a slot in a frame and target that slot. Every time a new scope is
//! created, it's provided with a new frame. Any data rooted in that frame remains rooted until
//! leaving the scope.
//!
//! There exist three kinds of scope: dynamic, local and async scopes. Dynamic scopes provide a
//! [`GcFrame`] which can grow to the necessary size, local scopes provide a statically-sized
//! [`LocalGcFrame`], and async scopes provide an [`AsyncGcFrame`] which is dynamically-sized like
//! a `GcFrame`. New dynamic scopes can only be created using a `GcFrame` or `AsyncGcFrame`, new
//! local scopes can be created using any target, and async scopes can only be created using an
//! `AsyncGcFrame`.
//!
//! A `GcFrame` lets you create  [`Output`]s and [`ReusableSlot`]s which are very similar. Both
//! target a reserved slot in that frame, they can be reused and consumed. When they're taken by
//!  value they're consumed, and both types return data that will remain rooted until you leave
//! the scope of the frame that roots them. They can also be taken by mutable reference, and here
//! they act differently. When a mutable reference to an output is used as a target, it returns
//! rooted data that inherits the lifetime of the reference. A reusable slot though returns data
//! that inherits the lifetime of the slot, to account for the fact that this data can become
//! unrooted while it is usable the data is returned as a `Ref` as if this target were an
//! unrooting target instead.
//!
//! A `LocalGcFrame` lets you create [`LocalOutput`]s and [`LocalReusableSlot`]s which behave
//! the same as their dynamic counterpart. The only difference is that these targets target a
//! local frame.
//!
//! There are effectively an infinite number of unrooting targets. Every rooting target can serve
//! as an unrooting target by providing an immutable reference. Sometimes this can lead to some
//! borrowing issues, for this purpose the `Unrooted` target exists which can be created by
//! calling [`Target::unrooted`].
//!
//! A full overview of all targets is provided below:
//!
//! | Type                                | Rooting   | Local | Async |
//! |-------------------------------------|-----------|-------|-------|
//! | `GcFrame<'scope>`                   | Yes       | No    | No    |
//! | `&mut GcFrame<'scope>`              | Yes       | No    | No    |
//! | `LocalGcFrame<'scope>`              | Yes       | Yes   | No    |
//! | `&mut LocalGcFrame<'scope>`         | Yes       | Yes   | No    |
//! | `AsyncGcFrame<'scope>`              | Yes       | No    | Yes   |
//! | `&mut AsyncGcFrame<'scope>`         | Yes       | No    | Yes   |
//! | `Output<'scope>`                    | Yes       | No    | No    |
//! | `&'scope mut Output<'_>`            | Yes       | No    | No    |
//! | `LocalOutput<'scope>`               | Yes       | Yes   | No    |
//! | `&'scope mut LocalOutput<'_>`       | Yes       | Yes   | No    |
//! | `ReusableSlot<'scope>`              | Yes       | No    | No    |
//! | `&'scope mut ReusableSlot<'_>`      | Partially | No    | No    |
//! | `LocalReusableSlot<'scope>`         | Yes       | Yes   | No    |
//! | `&'scope mut LocalReusableSlot<'_>` | Partially | Yes   | No    |
//! | `Unrooted<'scope>`                  | No        | No    | No    |
//! | `&Target<'scope>`                   | No        | No    | No    |
//!
//! [`Ref`]: crate::data::managed::Ref
//! [`Managed`]: crate::data::managed::Managed
//! [`memory`]: crate::memory
//! [`Call`]: crate::call::Call
//! [`Value`]: crate::data::managed::value::Value
//! [`Value::new`]: crate::data::managed::value::Value::new
//! [`ValueRef`]: crate::data::managed::value::ValueRef
//! [`ValueData`]: crate::data::managed::value::ValueData
//! [`ValueResult`]: crate::data::managed::value::ValueResult

use std::{marker::PhantomData, pin::Pin};

#[cfg(feature = "async")]
use self::frame::AsyncGcFrame;
use self::{
    frame::{BorrowedFrame, GcFrame, LocalFrame, LocalGcFrame, UnsizedLocalGcFrame},
    output::{LocalOutput, Output},
    private::TargetPriv,
    reusable_slot::{LocalReusableSlot, ReusableSlot},
    unrooted::Unrooted,
};
use super::scope::LocalScope;
#[cfg(feature = "multi-rt")]
#[cfg(not(any(feature = "julia-1-6", feature = "julia-1-7", feature = "julia-1-8")))]
use crate::runtime::handle::mt_handle::ActiveHandle;
use crate::{
    data::managed::Ref,
    prelude::{Managed, ValueData},
    runtime::handle::{weak_handle::WeakHandle, with_stack::StackHandle},
};

pub mod frame;
pub mod output;
pub mod reusable_slot;
pub mod unrooted;

/// Trait implemented by all targets.
///
/// For more information see the [module-level] docs.
///
/// [module-level]: self
pub trait Target<'target>: TargetPriv<'target> {
    /// Returns a new `Unrooted`.
    #[inline]
    fn unrooted(&self) -> Unrooted<'target> {
        unsafe { Unrooted::new() }
    }

    /// Create a new local scope and call `func`.
    ///
    /// The `LocalGcFrame` provided to `func` has capacity for `M` roots, `self` is propagated to
    /// the closure.
    #[inline]
    fn with_local_scope<T, F, const M: usize>(self, func: F) -> T
    where
        for<'inner> F: FnOnce(Self, LocalGcFrame<'inner, M>) -> T,
    {
        unsafe {
            let mut local_frame = LocalFrame::new();

            let pinned = local_frame.pin();
            let res = func(self, LocalGcFrame::new(&pinned));
            pinned.pop();
            res
        }
    }

    /// Convert `self` into an `ExtendedTarget`.
    #[inline]
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

    /// Convert `self` into an `ExtendedAsyncTarget`.
    #[cfg(feature = "async")]
    #[inline]
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

impl<'target, T, Tgt: Target<'target>> LocalScope<'target, T> for Tgt {}

/// A `Target` bundled with a [`GcFrame`].
pub struct ExtendedTarget<'target, 'current, 'borrow, Tgt>
where
    Tgt: Target<'target>,
{
    pub(crate) target: Tgt,
    pub(crate) frame: &'borrow mut GcFrame<'current>,
    pub(crate) _target_marker: PhantomData<&'target ()>,
}

impl<'target, 'current, 'borrow, Tgt> ExtendedTarget<'target, 'current, 'borrow, Tgt>
where
    Tgt: Target<'target>,
{
    /// Split the `ExtendedTarget` into its `Target` and a `BorrowedFrame`.
    #[inline]
    pub fn split(self) -> (Tgt, BorrowedFrame<'borrow, 'current, GcFrame<'current>>) {
        (self.target, BorrowedFrame(self.frame, PhantomData))
    }
}

#[cfg(feature = "async")]
/// A `Target` bundled with an [`AsyncGcFrame`].
pub struct ExtendedAsyncTarget<'target, 'current, 'borrow, Tgt>
where
    Tgt: Target<'target>,
{
    pub(crate) target: Tgt,
    pub(crate) frame: &'borrow mut AsyncGcFrame<'current>,
    pub(crate) _target_marker: PhantomData<&'target ()>,
}

#[cfg(feature = "async")]
impl<'target, 'current, 'borrow, Tgt> ExtendedAsyncTarget<'target, 'current, 'borrow, Tgt>
where
    Tgt: Target<'target>,
{
    /// Split the `ExtendedTarget` into its `Target` and a `BorrowedFrame`.
    #[inline]
    pub fn split(
        self,
    ) -> (
        Tgt,
        BorrowedFrame<'borrow, 'current, AsyncGcFrame<'current>>,
    ) {
        (self.target, BorrowedFrame(self.frame, PhantomData))
    }
}

impl<'target> Target<'target> for GcFrame<'target> {}

impl<'target, const N: usize> Target<'target> for LocalGcFrame<'target, N> {}

impl<'target> Target<'target> for UnsizedLocalGcFrame<'target> {}

impl<'target> Target<'target> for &mut GcFrame<'target> {}

impl<'target, const N: usize> Target<'target> for &mut LocalGcFrame<'target, N> {}

impl<'target> Target<'target> for &mut UnsizedLocalGcFrame<'target> {}

#[cfg(feature = "async")]
impl<'target> Target<'target> for AsyncGcFrame<'target> {}

#[cfg(feature = "async")]
impl<'target> Target<'target> for &mut AsyncGcFrame<'target> {}

impl<'target> Target<'target> for Unrooted<'target> {}

#[cfg(feature = "multi-rt")]
#[cfg(not(any(feature = "julia-1-6", feature = "julia-1-7", feature = "julia-1-8")))]
impl<'target> Target<'target> for ActiveHandle<'target> {}
impl<'target> Target<'target> for Pin<&'target mut WeakHandle> {}
impl<'target> Target<'target> for StackHandle<'target> {}

impl<'target> Target<'target> for Output<'target> {}

impl<'target> Target<'target> for LocalOutput<'target> {}

impl<'target> Target<'target> for &'target mut Output<'_> {}

impl<'target> Target<'target> for &'target mut LocalOutput<'_> {}

impl<'target> Target<'target> for ReusableSlot<'target> {}

impl<'target> Target<'target> for LocalReusableSlot<'target> {}

impl<'target> Target<'target> for &mut LocalReusableSlot<'target> {}

impl<'target> Target<'target> for &mut ReusableSlot<'target> {}

impl<'target, 'data, Tgt> Target<'target> for &Tgt where Tgt: Target<'target> {}

/// Defines the return types of a target, `Data`, `Exception`, and `Result`.
pub trait TargetType<'target>: Sized {
    /// Type returned by functions that don't catch Julia exceptions.
    ///
    /// For rooting targets, this type is `T`.
    /// For non-rooting targets, this type is [`Ref<'target, 'data, T>`].
    type Data<'data, T: Managed<'target, 'data>>;
}

pub type TargetResult<'scope, 'data, T, Tgt> =
    Result<<Tgt as TargetType<'scope>>::Data<'data, T>, ValueData<'scope, 'data, Tgt>>;

pub type TargetException<'scope, 'data, T, Tgt> = Result<T, ValueData<'scope, 'data, Tgt>>;

/// Extension trait for rooting targets.
pub trait RootingTarget<'target>: TargetType<'target> + Target<'target> {
    /// Convert data rooted with this target to an instance of the concrete type.
    fn into_concrete_type<'data, M: Managed<'target, 'data>>(t: Self::Data<'data, M>) -> M {
        // Safety: for rooting targets M == Self::Data<'data, M>
        unsafe { std::mem::transmute_copy(&t) }
    }

    /// Convert concrete data rooted with this target to an instance of the generic `Data` type.
    fn into_generic_type<'data, M: Managed<'target, 'data>>(t: M) -> Self::Data<'data, M> {
        // Safety: for rooting targets M == Self::Data<'data, M>
        unsafe { std::mem::transmute_copy(&t) }
    }
}

impl<'target> TargetType<'target> for &mut GcFrame<'target> {
    type Data<'data, T: Managed<'target, 'data>> = T;
}

impl<'target> RootingTarget<'target> for &mut GcFrame<'target> {}

impl<'target, const N: usize> TargetType<'target> for &mut LocalGcFrame<'target, N> {
    type Data<'data, T: Managed<'target, 'data>> = T;
}

impl<'target, const N: usize> RootingTarget<'target> for &mut LocalGcFrame<'target, N> {}

impl<'target> TargetType<'target> for &mut UnsizedLocalGcFrame<'target> {
    type Data<'data, T: Managed<'target, 'data>> = T;
}

impl<'target> RootingTarget<'target> for &mut UnsizedLocalGcFrame<'target> {}

impl<'target> TargetType<'target> for GcFrame<'target> {
    type Data<'data, T: Managed<'target, 'data>> = T;
}

impl<'target> RootingTarget<'target> for GcFrame<'target> {}

impl<'target, const N: usize> TargetType<'target> for LocalGcFrame<'target, N> {
    type Data<'data, T: Managed<'target, 'data>> = T;
}

impl<'target, const N: usize> RootingTarget<'target> for LocalGcFrame<'target, N> {}

impl<'target> TargetType<'target> for UnsizedLocalGcFrame<'target> {
    type Data<'data, T: Managed<'target, 'data>> = T;
}

impl<'target> RootingTarget<'target> for UnsizedLocalGcFrame<'target> {}

#[cfg(feature = "async")]
impl<'target> TargetType<'target> for &mut AsyncGcFrame<'target> {
    type Data<'data, T: Managed<'target, 'data>> = T;
}

#[cfg(feature = "async")]
impl<'target> TargetType<'target> for AsyncGcFrame<'target> {
    type Data<'data, T: Managed<'target, 'data>> = T;
}

#[cfg(feature = "async")]
impl<'target> RootingTarget<'target> for &mut AsyncGcFrame<'target> {}

#[cfg(feature = "async")]
impl<'target> RootingTarget<'target> for AsyncGcFrame<'target> {}

impl<'target> TargetType<'target> for Output<'target> {
    type Data<'data, T: Managed<'target, 'data>> = T;
}

impl<'target> RootingTarget<'target> for Output<'target> {}

impl<'target> TargetType<'target> for LocalOutput<'target> {
    type Data<'data, T: Managed<'target, 'data>> = T;
}

impl<'target> RootingTarget<'target> for LocalOutput<'target> {}

impl<'target> TargetType<'target> for &'target mut Output<'_> {
    type Data<'data, T: Managed<'target, 'data>> = T;
}

impl<'target> RootingTarget<'target> for &'target mut Output<'_> {}

impl<'target> TargetType<'target> for &'target mut LocalOutput<'_> {
    type Data<'data, T: Managed<'target, 'data>> = T;
}

impl<'target> RootingTarget<'target> for &'target mut LocalOutput<'_> {}

impl<'target> TargetType<'target> for ReusableSlot<'target> {
    type Data<'data, T: Managed<'target, 'data>> = T;
}

impl<'target> RootingTarget<'target> for ReusableSlot<'target> {}

impl<'target> TargetType<'target> for LocalReusableSlot<'target> {
    type Data<'data, T: Managed<'target, 'data>> = T;
}

impl<'target> RootingTarget<'target> for LocalReusableSlot<'target> {}

impl<'target> TargetType<'target> for &mut ReusableSlot<'target> {
    type Data<'data, T: Managed<'target, 'data>> = Ref<'target, 'data, T>;
}

impl<'target> TargetType<'target> for &mut LocalReusableSlot<'target> {
    type Data<'data, T: Managed<'target, 'data>> = Ref<'target, 'data, T>;
}

impl<'target> TargetType<'target> for Unrooted<'target> {
    type Data<'data, T: Managed<'target, 'data>> = Ref<'target, 'data, T>;
}

#[cfg(feature = "multi-rt")]
#[cfg(not(any(feature = "julia-1-6", feature = "julia-1-7", feature = "julia-1-8")))]
impl<'target> TargetType<'target> for ActiveHandle<'target> {
    type Data<'data, T: Managed<'target, 'data>> = Ref<'target, 'data, T>;
}

impl<'target> TargetType<'target> for Pin<&'target mut WeakHandle> {
    type Data<'data, T: Managed<'target, 'data>> = Ref<'target, 'data, T>;
}

impl<'target> TargetType<'target> for StackHandle<'target> {
    type Data<'data, T: Managed<'target, 'data>> = Ref<'target, 'data, T>;
}

impl<'target, U: TargetType<'target>> TargetType<'target> for &U {
    type Data<'data, T: Managed<'target, 'data>> = Ref<'target, 'data, T>;
}

pub(crate) mod private {
    use std::{pin::Pin, ptr::NonNull};

    use jl_sys::jl_value_t;

    #[cfg(feature = "async")]
    use super::AsyncGcFrame;
    use super::{
        frame::{LocalGcFrame, UnsizedLocalGcFrame},
        output::LocalOutput,
        reusable_slot::{LocalReusableSlot, ReusableSlot},
        unrooted::Unrooted,
        GcFrame, Output, TargetException, TargetResult, TargetType,
    };
    #[cfg(feature = "multi-rt")]
    #[cfg(not(any(feature = "julia-1-6", feature = "julia-1-7", feature = "julia-1-8")))]
    use crate::runtime::handle::mt_handle::ActiveHandle;
    use crate::{
        data::managed::{
            private::ManagedPriv,
            value::{Value, ValueRef},
            Managed, Ref,
        },
        private::Private,
        runtime::handle::{weak_handle::WeakHandle, with_stack::StackHandle},
    };

    pub trait TargetPriv<'target>: TargetType<'target> {
        // Safety: the pointer must point to valid data.
        unsafe fn data_from_ptr<'data, T: Managed<'target, 'data>>(
            self,
            value: NonNull<T::Wraps>,
            _: Private,
        ) -> Self::Data<'data, T>;

        // Safety: the pointer must point to valid data.
        unsafe fn result_from_ptr<'data, T: Managed<'target, 'data>>(
            self,
            result: Result<NonNull<T::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> TargetResult<'target, 'data, T, Self>;

        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn result_from_unrooted<'data, T: Managed<'target, 'data>>(
            self,
            result: Result<Ref<'target, 'data, T>, ValueRef<'target, 'data>>,
            _: Private,
        ) -> TargetResult<'target, 'data, T, Self> {
            let result = match result {
                Ok(v) => Ok(v.ptr()),
                Err(e) => Err(e.ptr()),
            };

            self.result_from_ptr(result, Private)
        }

        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn result_from_rooted<'data, T: Managed<'target, 'data>>(
            self,
            result: Result<T, Value<'target, 'data>>,
            _: Private,
        ) -> TargetResult<'target, 'data, T, Self> {
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
        ) -> TargetException<'target, 'data, T, Self>;
    }

    impl<'target> TargetPriv<'target> for &mut GcFrame<'target> {
        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn data_from_ptr<'data, T: Managed<'target, 'data>>(
            self,
            value: NonNull<T::Wraps>,
            _: Private,
        ) -> Self::Data<'data, T> {
            self.root(value)
        }

        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn result_from_ptr<'data, T: Managed<'target, 'data>>(
            self,
            result: Result<NonNull<T::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> TargetResult<'target, 'data, T, Self> {
            match result {
                Ok(t) => Ok(self.root(t)),
                Err(e) => Err(self.root(e)),
            }
        }

        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn exception_from_ptr<'data, T>(
            self,
            result: Result<T, NonNull<jl_value_t>>,
            _: Private,
        ) -> TargetException<'target, 'data, T, Self> {
            match result {
                Ok(t) => Ok(t),
                Err(e) => Err(self.root(e)),
            }
        }
    }

    impl<'target, const N: usize> TargetPriv<'target> for &mut LocalGcFrame<'target, N> {
        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn data_from_ptr<'data, T: Managed<'target, 'data>>(
            self,
            value: NonNull<T::Wraps>,
            _: Private,
        ) -> Self::Data<'data, T> {
            self.root(value)
        }

        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn result_from_ptr<'data, T: Managed<'target, 'data>>(
            self,
            result: Result<NonNull<T::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> TargetResult<'target, 'data, T, Self> {
            match result {
                Ok(t) => Ok(self.root(t)),
                Err(e) => Err(self.root(e)),
            }
        }

        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn exception_from_ptr<'data, T>(
            self,
            result: Result<T, NonNull<jl_value_t>>,
            _: Private,
        ) -> TargetException<'target, 'data, T, Self> {
            match result {
                Ok(t) => Ok(t),
                Err(e) => Err(self.root(e)),
            }
        }
    }

    impl<'target> TargetPriv<'target> for &mut UnsizedLocalGcFrame<'target> {
        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn data_from_ptr<'data, T: Managed<'target, 'data>>(
            self,
            value: NonNull<T::Wraps>,
            _: Private,
        ) -> Self::Data<'data, T> {
            self.root(value)
        }

        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn result_from_ptr<'data, T: Managed<'target, 'data>>(
            self,
            result: Result<NonNull<T::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> TargetResult<'target, 'data, T, Self> {
            match result {
                Ok(t) => Ok(self.root(t)),
                Err(e) => Err(self.root(e)),
            }
        }

        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn exception_from_ptr<'data, T>(
            self,
            result: Result<T, NonNull<jl_value_t>>,
            _: Private,
        ) -> TargetException<'target, 'data, T, Self> {
            match result {
                Ok(t) => Ok(t),
                Err(e) => Err(self.root(e)),
            }
        }
    }

    impl<'target> TargetPriv<'target> for GcFrame<'target> {
        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn data_from_ptr<'data, T: Managed<'target, 'data>>(
            self,
            value: NonNull<T::Wraps>,
            _: Private,
        ) -> Self::Data<'data, T> {
            self.root(value)
        }

        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn result_from_ptr<'data, T: Managed<'target, 'data>>(
            self,
            result: Result<NonNull<T::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> TargetResult<'target, 'data, T, Self> {
            match result {
                Ok(t) => Ok(self.root(t)),
                Err(e) => Err(self.root(e)),
            }
        }

        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn exception_from_ptr<'data, T>(
            self,
            result: Result<T, NonNull<jl_value_t>>,
            _: Private,
        ) -> TargetException<'target, 'data, T, Self> {
            match result {
                Ok(t) => Ok(t),
                Err(e) => Err(self.root(e)),
            }
        }
    }

    impl<'target, const N: usize> TargetPriv<'target> for LocalGcFrame<'target, N> {
        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn data_from_ptr<'data, T: Managed<'target, 'data>>(
            mut self,
            value: NonNull<T::Wraps>,
            _: Private,
        ) -> Self::Data<'data, T> {
            self.root(value)
        }

        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn result_from_ptr<'data, T: Managed<'target, 'data>>(
            mut self,
            result: Result<NonNull<T::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> TargetResult<'target, 'data, T, Self> {
            match result {
                Ok(t) => Ok(self.root(t)),
                Err(e) => Err(self.root(e)),
            }
        }

        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn exception_from_ptr<'data, T>(
            mut self,
            result: Result<T, NonNull<jl_value_t>>,
            _: Private,
        ) -> TargetException<'target, 'data, T, Self> {
            match result {
                Ok(t) => Ok(t),
                Err(e) => Err(self.root(e)),
            }
        }
    }

    impl<'target> TargetPriv<'target> for UnsizedLocalGcFrame<'target> {
        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn data_from_ptr<'data, T: Managed<'target, 'data>>(
            mut self,
            value: NonNull<T::Wraps>,
            _: Private,
        ) -> Self::Data<'data, T> {
            self.root(value)
        }

        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn result_from_ptr<'data, T: Managed<'target, 'data>>(
            mut self,
            result: Result<NonNull<T::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> TargetResult<'target, 'data, T, Self> {
            match result {
                Ok(t) => Ok(self.root(t)),
                Err(e) => Err(self.root(e)),
            }
        }

        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn exception_from_ptr<'data, T>(
            mut self,
            result: Result<T, NonNull<jl_value_t>>,
            _: Private,
        ) -> TargetException<'target, 'data, T, Self> {
            match result {
                Ok(t) => Ok(t),
                Err(e) => Err(self.root(e)),
            }
        }
    }

    #[cfg(feature = "async")]
    impl<'target> TargetPriv<'target> for &mut AsyncGcFrame<'target> {
        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn data_from_ptr<'data, T: Managed<'target, 'data>>(
            self,
            value: NonNull<T::Wraps>,
            _: Private,
        ) -> Self::Data<'data, T> {
            self.root(value)
        }

        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn result_from_ptr<'data, T: Managed<'target, 'data>>(
            self,
            result: Result<NonNull<T::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> TargetResult<'target, 'data, T, Self> {
            match result {
                Ok(t) => Ok(self.root(t)),
                Err(e) => Err(self.root(e)),
            }
        }

        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn exception_from_ptr<'data, T>(
            self,
            result: Result<T, NonNull<jl_value_t>>,
            _: Private,
        ) -> TargetException<'target, 'data, T, Self> {
            match result {
                Ok(t) => Ok(t),
                Err(e) => Err(self.root(e)),
            }
        }
    }

    #[cfg(feature = "async")]
    impl<'target> TargetPriv<'target> for AsyncGcFrame<'target> {
        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn data_from_ptr<'data, T: Managed<'target, 'data>>(
            self,
            value: NonNull<T::Wraps>,
            _: Private,
        ) -> Self::Data<'data, T> {
            self.root(value)
        }

        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn result_from_ptr<'data, T: Managed<'target, 'data>>(
            self,
            result: Result<NonNull<T::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> TargetResult<'target, 'data, T, Self> {
            match result {
                Ok(t) => Ok(self.root(t)),
                Err(e) => Err(self.root(e)),
            }
        }

        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn exception_from_ptr<'data, T>(
            self,
            result: Result<T, NonNull<jl_value_t>>,
            _: Private,
        ) -> TargetException<'target, 'data, T, Self> {
            match result {
                Ok(t) => Ok(t),
                Err(e) => Err(self.root(e)),
            }
        }
    }

    impl<'target> TargetPriv<'target> for Output<'target> {
        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn data_from_ptr<'data, T: Managed<'target, 'data>>(
            self,
            value: NonNull<T::Wraps>,
            _: Private,
        ) -> Self::Data<'data, T> {
            self.consume(value)
        }

        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn result_from_ptr<'data, T: Managed<'target, 'data>>(
            self,
            result: Result<NonNull<T::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> TargetResult<'target, 'data, T, Self> {
            match result {
                Ok(t) => Ok(self.consume(t)),
                Err(e) => Err(self.consume(e)),
            }
        }

        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn exception_from_ptr<'data, T>(
            self,
            result: Result<T, NonNull<jl_value_t>>,
            _: Private,
        ) -> TargetException<'target, 'data, T, Self> {
            match result {
                Ok(t) => Ok(t),
                Err(e) => Err(self.consume(e)),
            }
        }
    }

    impl<'target> TargetPriv<'target> for LocalOutput<'target> {
        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn data_from_ptr<'data, T: Managed<'target, 'data>>(
            self,
            value: NonNull<T::Wraps>,
            _: Private,
        ) -> Self::Data<'data, T> {
            self.consume(value)
        }

        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn result_from_ptr<'data, T: Managed<'target, 'data>>(
            self,
            result: Result<NonNull<T::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> TargetResult<'target, 'data, T, Self> {
            match result {
                Ok(t) => Ok(self.consume(t)),
                Err(e) => Err(self.consume(e)),
            }
        }

        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn exception_from_ptr<'data, T>(
            self,
            result: Result<T, NonNull<jl_value_t>>,
            _: Private,
        ) -> TargetException<'target, 'data, T, Self> {
            match result {
                Ok(t) => Ok(t),
                Err(e) => Err(self.consume(e)),
            }
        }
    }

    impl<'target> TargetPriv<'target> for &'target mut Output<'_> {
        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn data_from_ptr<'data, T: Managed<'target, 'data>>(
            self,
            value: NonNull<T::Wraps>,
            _: Private,
        ) -> Self::Data<'data, T> {
            self.temporary(value)
        }

        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn result_from_ptr<'data, T: Managed<'target, 'data>>(
            self,
            result: Result<NonNull<T::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> TargetResult<'target, 'data, T, Self> {
            match result {
                Ok(t) => Ok(self.temporary(t)),
                Err(e) => Err(self.temporary(e)),
            }
        }

        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn exception_from_ptr<'data, T>(
            self,
            result: Result<T, NonNull<jl_value_t>>,
            _: Private,
        ) -> TargetException<'target, 'data, T, Self> {
            match result {
                Ok(t) => Ok(t),
                Err(e) => Err(self.temporary(e)),
            }
        }
    }

    impl<'target> TargetPriv<'target> for &'target mut LocalOutput<'_> {
        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn data_from_ptr<'data, T: Managed<'target, 'data>>(
            self,
            value: NonNull<T::Wraps>,
            _: Private,
        ) -> Self::Data<'data, T> {
            self.temporary(value)
        }

        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn result_from_ptr<'data, T: Managed<'target, 'data>>(
            self,
            result: Result<NonNull<T::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> TargetResult<'target, 'data, T, Self> {
            match result {
                Ok(t) => Ok(self.temporary(t)),
                Err(e) => Err(self.temporary(e)),
            }
        }

        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn exception_from_ptr<'data, T>(
            self,
            result: Result<T, NonNull<jl_value_t>>,
            _: Private,
        ) -> TargetException<'target, 'data, T, Self> {
            match result {
                Ok(t) => Ok(t),
                Err(e) => Err(self.temporary(e)),
            }
        }
    }

    impl<'target> TargetPriv<'target> for ReusableSlot<'target> {
        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn data_from_ptr<'data, T: Managed<'target, 'data>>(
            self,
            value: NonNull<T::Wraps>,
            _: Private,
        ) -> Self::Data<'data, T> {
            self.consume(value)
        }

        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn result_from_ptr<'data, T: Managed<'target, 'data>>(
            self,
            result: Result<NonNull<T::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> TargetResult<'target, 'data, T, Self> {
            match result {
                Ok(t) => Ok(self.consume(t)),
                Err(e) => Err(self.consume(e)),
            }
        }

        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn exception_from_ptr<'data, T>(
            self,
            result: Result<T, NonNull<jl_value_t>>,
            _: Private,
        ) -> TargetException<'target, 'data, T, Self> {
            match result {
                Ok(t) => Ok(t),
                Err(e) => Err(self.consume(e)),
            }
        }
    }

    impl<'target> TargetPriv<'target> for LocalReusableSlot<'target> {
        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn data_from_ptr<'data, T: Managed<'target, 'data>>(
            self,
            value: NonNull<T::Wraps>,
            _: Private,
        ) -> Self::Data<'data, T> {
            self.consume(value)
        }

        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn result_from_ptr<'data, T: Managed<'target, 'data>>(
            self,
            result: Result<NonNull<T::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> TargetResult<'target, 'data, T, Self> {
            match result {
                Ok(t) => Ok(self.consume(t)),
                Err(e) => Err(self.consume(e)),
            }
        }

        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn exception_from_ptr<'data, T>(
            self,
            result: Result<T, NonNull<jl_value_t>>,
            _: Private,
        ) -> TargetException<'target, 'data, T, Self> {
            match result {
                Ok(t) => Ok(t),
                Err(e) => Err(self.consume(e)),
            }
        }
    }

    impl<'target> TargetPriv<'target> for &mut ReusableSlot<'target> {
        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn data_from_ptr<'data, T: Managed<'target, 'data>>(
            self,
            value: NonNull<T::Wraps>,
            _: Private,
        ) -> Self::Data<'data, T> {
            self.temporary(value)
        }

        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn result_from_ptr<'data, T: Managed<'target, 'data>>(
            self,
            result: Result<NonNull<T::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> TargetResult<'target, 'data, T, Self> {
            match result {
                Ok(t) => Ok(self.temporary(t)),
                Err(e) => Err(self.temporary(e)),
            }
        }

        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn exception_from_ptr<'data, T>(
            self,
            result: Result<T, NonNull<jl_value_t>>,
            _: Private,
        ) -> TargetException<'target, 'data, T, Self> {
            match result {
                Ok(t) => Ok(t),
                Err(e) => Err(self.temporary(e)),
            }
        }
    }

    impl<'target> TargetPriv<'target> for &mut LocalReusableSlot<'target> {
        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn data_from_ptr<'data, T: Managed<'target, 'data>>(
            self,
            value: NonNull<T::Wraps>,
            _: Private,
        ) -> Self::Data<'data, T> {
            self.temporary(value)
        }

        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn result_from_ptr<'data, T: Managed<'target, 'data>>(
            self,
            result: Result<NonNull<T::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> TargetResult<'target, 'data, T, Self> {
            match result {
                Ok(t) => Ok(self.temporary(t)),
                Err(e) => Err(self.temporary(e)),
            }
        }

        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn exception_from_ptr<'data, T>(
            self,
            result: Result<T, NonNull<jl_value_t>>,
            _: Private,
        ) -> TargetException<'target, 'data, T, Self> {
            match result {
                Ok(t) => Ok(t),
                Err(e) => Err(self.temporary(e)),
            }
        }
    }

    impl<'target> TargetPriv<'target> for Unrooted<'target> {
        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn data_from_ptr<'data, T: Managed<'target, 'data>>(
            self,
            value: NonNull<T::Wraps>,
            _: Private,
        ) -> Self::Data<'data, T> {
            Ref::wrap(value)
        }

        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn result_from_ptr<'data, T: Managed<'target, 'data>>(
            self,
            result: Result<NonNull<T::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> TargetResult<'target, 'data, T, Self> {
            match result {
                Ok(t) => Ok(Ref::wrap(t)),
                Err(e) => Err(Ref::wrap(e)),
            }
        }

        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn exception_from_ptr<'data, T>(
            self,
            result: Result<T, NonNull<jl_value_t>>,
            _: Private,
        ) -> TargetException<'target, 'data, T, Self> {
            match result {
                Ok(t) => Ok(t),
                Err(e) => Err(Ref::wrap(e)),
            }
        }
    }

    #[cfg(feature = "multi-rt")]
    #[cfg(not(any(feature = "julia-1-6", feature = "julia-1-7", feature = "julia-1-8")))]
    impl<'target> TargetPriv<'target> for ActiveHandle<'target> {
        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn data_from_ptr<'data, T: Managed<'target, 'data>>(
            self,
            value: NonNull<T::Wraps>,
            _: Private,
        ) -> Self::Data<'data, T> {
            Ref::wrap(value)
        }

        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn result_from_ptr<'data, T: Managed<'target, 'data>>(
            self,
            result: Result<NonNull<T::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> TargetResult<'target, 'data, T, Self> {
            match result {
                Ok(t) => Ok(Ref::wrap(t)),
                Err(e) => Err(Ref::wrap(e)),
            }
        }

        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn exception_from_ptr<'data, T>(
            self,
            result: Result<T, NonNull<jl_value_t>>,
            _: Private,
        ) -> TargetException<'target, 'data, T, Self> {
            match result {
                Ok(t) => Ok(t),
                Err(e) => Err(Ref::wrap(e)),
            }
        }
    }

    impl<'target> TargetPriv<'target> for Pin<&'target mut WeakHandle> {
        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn data_from_ptr<'data, T: Managed<'target, 'data>>(
            self,
            value: NonNull<T::Wraps>,
            _: Private,
        ) -> Self::Data<'data, T> {
            Ref::wrap(value)
        }

        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn result_from_ptr<'data, T: Managed<'target, 'data>>(
            self,
            result: Result<NonNull<T::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> TargetResult<'target, 'data, T, Self> {
            match result {
                Ok(t) => Ok(Ref::wrap(t)),
                Err(e) => Err(Ref::wrap(e)),
            }
        }

        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn exception_from_ptr<'data, T>(
            self,
            result: Result<T, NonNull<jl_value_t>>,
            _: Private,
        ) -> TargetException<'target, 'data, T, Self> {
            match result {
                Ok(t) => Ok(t),
                Err(e) => Err(Ref::wrap(e)),
            }
        }
    }

    impl<'target> TargetPriv<'target> for StackHandle<'target> {
        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn data_from_ptr<'data, T: Managed<'target, 'data>>(
            self,
            value: NonNull<T::Wraps>,
            _: Private,
        ) -> Self::Data<'data, T> {
            Ref::wrap(value)
        }

        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn result_from_ptr<'data, T: Managed<'target, 'data>>(
            self,
            result: Result<NonNull<T::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> TargetResult<'target, 'data, T, Self> {
            match result {
                Ok(t) => Ok(Ref::wrap(t)),
                Err(e) => Err(Ref::wrap(e)),
            }
        }

        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn exception_from_ptr<'data, T>(
            self,
            result: Result<T, NonNull<jl_value_t>>,
            _: Private,
        ) -> TargetException<'target, 'data, T, Self> {
            match result {
                Ok(t) => Ok(t),
                Err(e) => Err(Ref::wrap(e)),
            }
        }
    }

    impl<'target, U: TargetPriv<'target>> TargetPriv<'target> for &U {
        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn data_from_ptr<'data, T: Managed<'target, 'data>>(
            self,
            value: NonNull<T::Wraps>,
            _: Private,
        ) -> Self::Data<'data, T> {
            Ref::wrap(value)
        }

        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn result_from_ptr<'data, T: Managed<'target, 'data>>(
            self,
            result: Result<NonNull<T::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> TargetResult<'target, 'data, T, Self> {
            match result {
                Ok(t) => Ok(Ref::wrap(t)),
                Err(e) => Err(Ref::wrap(e)),
            }
        }

        // Safety: the pointer must point to valid data.
        #[inline]
        unsafe fn exception_from_ptr<'data, T>(
            self,
            result: Result<T, NonNull<jl_value_t>>,
            _: Private,
        ) -> TargetException<'target, 'data, T, Self> {
            match result {
                Ok(t) => Ok(t),
                Err(e) => Err(Ref::wrap(e)),
            }
        }
    }
}
