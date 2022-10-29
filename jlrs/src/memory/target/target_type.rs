//! Traits used to declare what type of data is returned by a target.

use crate::{
    error::{JuliaResult, JuliaResultRef},
    memory::target::{frame::GcFrame, global::Global, output::Output},
    wrappers::ptr::Ref,
    wrappers::ptr::Wrapper,
};

#[cfg(feature = "async")]
use crate::memory::target::frame::AsyncGcFrame;

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
}

/// Defines the return types of an `ExceptionTarget`, `Exception`.
pub trait ExceptionTargetType<'target> {
    /// Type returned by methods that don't return Julia data on succes, but can throw a Julia
    /// exception which is caught.
    ///
    /// For rooting targets, this type is [`JuliaResult<'target, 'data, T>`].
    /// For non-rooting targets, this type is [`JuliaResultRef<'target, 'data, T>`].
    // TODO: 'data
    type Exception<'data, T>;
}

impl<'target> TargetType<'target> for &mut GcFrame<'target> {
    type Data<'data, T: Wrapper<'target, 'data>> = T;
    type Result<'data, T: Wrapper<'target, 'data>> = JuliaResult<'target, 'data, T>;
}

impl<'target> TargetType<'target> for GcFrame<'target> {
    type Data<'data, T: Wrapper<'target, 'data>> = T;
    type Result<'data, T: Wrapper<'target, 'data>> = JuliaResult<'target, 'data, T>;
}

#[cfg(feature = "async")]
impl<'target> TargetType<'target> for &mut AsyncGcFrame<'target> {
    type Data<'data, T: Wrapper<'target, 'data>> = T;
    type Result<'data, T: Wrapper<'target, 'data>> = JuliaResult<'target, 'data, T>;
}

#[cfg(feature = "async")]
impl<'target> TargetType<'target> for AsyncGcFrame<'target> {
    type Data<'data, T: Wrapper<'target, 'data>> = T;
    type Result<'data, T: Wrapper<'target, 'data>> = JuliaResult<'target, 'data, T>;
}

impl<'target> TargetType<'target> for Output<'target> {
    type Data<'data, T: Wrapper<'target, 'data>> = T;
    type Result<'data, T: Wrapper<'target, 'data>> = JuliaResult<'target, 'data, T>;
}

impl<'target> TargetType<'target> for &'target mut Output<'_> {
    type Data<'data, T: Wrapper<'target, 'data>> = T;
    type Result<'data, T: Wrapper<'target, 'data>> = JuliaResult<'target, 'data, T>;
}

impl<'target> TargetType<'target> for Global<'target> {
    type Data<'data, T: Wrapper<'target, 'data>> = Ref<'target, 'data, T>;
    type Result<'data, T: Wrapper<'target, 'data>> =
        JuliaResultRef<'target, 'data, Ref<'target, 'data, T>>;
}

impl<'target, U: TargetType<'target>> TargetType<'target> for &U {
    type Data<'data, T: Wrapper<'target, 'data>> = Ref<'target, 'data, T>;
    type Result<'data, T: Wrapper<'target, 'data>> =
        JuliaResultRef<'target, 'data, Ref<'target, 'data, T>>;
}

impl<'target> ExceptionTargetType<'target> for &mut GcFrame<'target> {
    type Exception<'data, T> = JuliaResult<'target, 'data, T>;
}

impl<'target> ExceptionTargetType<'target> for GcFrame<'target> {
    type Exception<'data, T> = JuliaResult<'target, 'data, T>;
}

#[cfg(feature = "async")]
impl<'target> ExceptionTargetType<'target> for &mut AsyncGcFrame<'target> {
    type Exception<'data, T> = JuliaResult<'target, 'data, T>;
}

#[cfg(feature = "async")]
impl<'target> ExceptionTargetType<'target> for AsyncGcFrame<'target> {
    type Exception<'data, T> = JuliaResult<'target, 'data, T>;
}

impl<'target> ExceptionTargetType<'target> for Output<'target> {
    type Exception<'data, T> = JuliaResult<'target, 'data, T>;
}

impl<'target> ExceptionTargetType<'target> for &'target mut Output<'_> {
    type Exception<'data, T> = JuliaResult<'target, 'data, T>;
}

impl<'target> ExceptionTargetType<'target> for Global<'target> {
    type Exception<'data, T> = JuliaResultRef<'target, 'data, T>;
}

impl<'target, U: ExceptionTargetType<'target>> ExceptionTargetType<'target> for &U {
    type Exception<'data, T> = JuliaResultRef<'target, 'data, T>;
}
