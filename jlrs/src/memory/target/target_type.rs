//! Traits used to declare what type of data is returned by a target.

use super::reusable_slot::ReusableSlot;
#[cfg(feature = "async")]
use crate::memory::target::frame::AsyncGcFrame;
use crate::{
    error::{JuliaResult, JuliaResultRef},
    memory::target::{frame::GcFrame, output::Output, unrooted::Unrooted},
    wrappers::ptr::{Ref, Wrapper},
};

/// Defines the return types of a target, `Data` and `Result`.
pub trait TargetType<'target>: Sized {
    /// Type returned by methods that don't catch Julia exceptions.
    ///
    /// For rooting targets, this type is `T`.
    /// For non-rooting targets, this type is [`Ref<'target, 'data, T>`].
    type Data<'data, T: Wrapper<'target, 'data>>;

    /// Type returned by methods that catch Julia exceptions.
    ///
    /// For rooting targets, this type is [`JuliaResult<'target, 'data, T>`].
    /// For non-rooting targets, this type is [`JuliaResultRef<'target, 'data, Ref<'target, 'data, T>>`].
    type Result<'data, T: Wrapper<'target, 'data>>;

    /// Type returned by methods that don't return Julia data on succes, but can throw a Julia
    /// exception which is caught.
    ///
    /// For rooting targets, this type is [`JuliaResult<'target, 'data, T>`].
    /// For non-rooting targets, this type is [`JuliaResultRef<'target, 'data, T>`].
    type Exception<'data, T>;
}

impl<'target> TargetType<'target> for &mut GcFrame<'target> {
    type Data<'data, T: Wrapper<'target, 'data>> = T;
    type Result<'data, T: Wrapper<'target, 'data>> = JuliaResult<'target, 'data, T>;
    type Exception<'data, T> = JuliaResult<'target, 'data, T>;
}

impl<'target> TargetType<'target> for GcFrame<'target> {
    type Data<'data, T: Wrapper<'target, 'data>> = T;
    type Result<'data, T: Wrapper<'target, 'data>> = JuliaResult<'target, 'data, T>;
    type Exception<'data, T> = JuliaResult<'target, 'data, T>;
}

#[cfg(feature = "async")]
impl<'target> TargetType<'target> for &mut AsyncGcFrame<'target> {
    type Data<'data, T: Wrapper<'target, 'data>> = T;
    type Result<'data, T: Wrapper<'target, 'data>> = JuliaResult<'target, 'data, T>;
    type Exception<'data, T> = JuliaResult<'target, 'data, T>;
}

#[cfg(feature = "async")]
impl<'target> TargetType<'target> for AsyncGcFrame<'target> {
    type Data<'data, T: Wrapper<'target, 'data>> = T;
    type Result<'data, T: Wrapper<'target, 'data>> = JuliaResult<'target, 'data, T>;
    type Exception<'data, T> = JuliaResult<'target, 'data, T>;
}

impl<'target> TargetType<'target> for Output<'target> {
    type Data<'data, T: Wrapper<'target, 'data>> = T;
    type Result<'data, T: Wrapper<'target, 'data>> = JuliaResult<'target, 'data, T>;
    type Exception<'data, T> = JuliaResult<'target, 'data, T>;
}

impl<'target> TargetType<'target> for &'target mut Output<'_> {
    type Data<'data, T: Wrapper<'target, 'data>> = T;
    type Result<'data, T: Wrapper<'target, 'data>> = JuliaResult<'target, 'data, T>;
    type Exception<'data, T> = JuliaResult<'target, 'data, T>;
}

impl<'target> TargetType<'target> for ReusableSlot<'target> {
    type Data<'data, T: Wrapper<'target, 'data>> = T;
    type Result<'data, T: Wrapper<'target, 'data>> = JuliaResult<'target, 'data, T>;
    type Exception<'data, T> = JuliaResult<'target, 'data, T>;
}

impl<'target> TargetType<'target> for &mut ReusableSlot<'target> {
    type Data<'data, T: Wrapper<'target, 'data>> = Ref<'target, 'data, T>;
    type Result<'data, T: Wrapper<'target, 'data>> =
        JuliaResultRef<'target, 'data, Ref<'target, 'data, T>>;
    type Exception<'data, T> = JuliaResultRef<'target, 'data, T>;
}

impl<'target> TargetType<'target> for Unrooted<'target> {
    type Data<'data, T: Wrapper<'target, 'data>> = Ref<'target, 'data, T>;
    type Result<'data, T: Wrapper<'target, 'data>> =
        JuliaResultRef<'target, 'data, Ref<'target, 'data, T>>;
    type Exception<'data, T> = JuliaResultRef<'target, 'data, T>;
}

impl<'target, U: TargetType<'target>> TargetType<'target> for &U {
    type Data<'data, T: Wrapper<'target, 'data>> = Ref<'target, 'data, T>;
    type Result<'data, T: Wrapper<'target, 'data>> =
        JuliaResultRef<'target, 'data, Ref<'target, 'data, T>>;
    type Exception<'data, T> = JuliaResultRef<'target, 'data, T>;
}
