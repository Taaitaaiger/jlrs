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
pub trait TargetType<'target, 'data, T: Wrapper<'target, 'data>>: Sized {
    /// Type returned by methods that don't catch Julia exceptions.
    ///
    /// For rooting targets, this type is `T`.
    /// For non-rooting targets, this type is [`Ref<'target, 'data, T>`].
    type Data;

    /// Type returned by methods that catch Julia exceptions.
    ///
    /// For rooting targets, this type is [`JuliaResult<'target, 'data, T>`].
    /// For non-rooting targets, this type is [`JuliaResultRef<'target, 'data, Ref<'target, 'data, T>>`].
    type Result;
}

/// Defines the return types of an `ExceptionTarget`, `Exception`.
pub trait ExceptionTargetType<'target, 'data, T> {
    /// Type returned by methods that don't return Julia data on succes, but can throw a Julia
    /// exception which is caught.
    ///
    /// For rooting targets, this type is [`JuliaResult<'target, 'data, T>`].
    /// For non-rooting targets, this type is [`JuliaResultRef<'target, 'data, T>`].
    type Exception;
}

impl<'target, 'data, T: Wrapper<'target, 'data>> TargetType<'target, 'data, T>
    for &mut GcFrame<'target>
{
    type Data = T;
    type Result = JuliaResult<'target, 'data, T>;
}

impl<'target, 'data, T: Wrapper<'target, 'data>> TargetType<'target, 'data, T>
    for GcFrame<'target>
{
    type Data = T;
    type Result = JuliaResult<'target, 'data, T>;
}

#[cfg(feature = "async")]
impl<'target, 'data, T: Wrapper<'target, 'data>> TargetType<'target, 'data, T>
    for &mut AsyncGcFrame<'target>
{
    type Data = T;
    type Result = JuliaResult<'target, 'data, T>;
}

#[cfg(feature = "async")]
impl<'target, 'data, T: Wrapper<'target, 'data>> TargetType<'target, 'data, T>
    for AsyncGcFrame<'target>
{
    type Data = T;
    type Result = JuliaResult<'target, 'data, T>;
}

impl<'target, 'data, T: Wrapper<'target, 'data>> TargetType<'target, 'data, T> for Output<'target> {
    type Data = T;
    type Result = JuliaResult<'target, 'data, T>;
}

impl<'target, 'data, T: Wrapper<'target, 'data>> TargetType<'target, 'data, T>
    for &'target mut Output<'_>
{
    type Data = T;
    type Result = JuliaResult<'target, 'data, T>;
}

impl<'target, 'data, T: Wrapper<'target, 'data>> TargetType<'target, 'data, T> for Global<'target> {
    type Data = Ref<'target, 'data, T>;
    type Result = JuliaResultRef<'target, 'data, Ref<'target, 'data, T>>;
}

impl<'target, 'data, W: Wrapper<'target, 'data>, T: TargetType<'target, 'data, W>>
    TargetType<'target, 'data, W> for &T
{
    type Data = Ref<'target, 'data, W>;
    type Result = JuliaResultRef<'target, 'data, Ref<'target, 'data, W>>;
}

impl<'target, 'data, T> ExceptionTargetType<'target, 'data, T> for &mut GcFrame<'target> {
    type Exception = JuliaResult<'target, 'data, T>;
}

impl<'target, 'data, T> ExceptionTargetType<'target, 'data, T> for GcFrame<'target> {
    type Exception = JuliaResult<'target, 'data, T>;
}

#[cfg(feature = "async")]
impl<'target, 'data, T> ExceptionTargetType<'target, 'data, T> for &mut AsyncGcFrame<'target> {
    type Exception = JuliaResult<'target, 'data, T>;
}

#[cfg(feature = "async")]
impl<'target, 'data, T> ExceptionTargetType<'target, 'data, T> for AsyncGcFrame<'target> {
    type Exception = JuliaResult<'target, 'data, T>;
}

impl<'target, 'data, T> ExceptionTargetType<'target, 'data, T> for Output<'target> {
    type Exception = JuliaResult<'target, 'data, T>;
}

impl<'target, 'data, T> ExceptionTargetType<'target, 'data, T> for &'target mut Output<'_> {
    type Exception = JuliaResult<'target, 'data, T>;
}

impl<'target, 'data, T> ExceptionTargetType<'target, 'data, T> for Global<'target> {
    type Exception = JuliaResultRef<'target, 'data, T>;
}

impl<'target, 'data, W, T: ExceptionTargetType<'target, 'data, W>>
    ExceptionTargetType<'target, 'data, W> for &T
{
    type Exception = JuliaResultRef<'target, 'data, W>;
}
