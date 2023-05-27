//! Trait used to declare what type of data is returned by a target.

use super::reusable_slot::ReusableSlot;
#[cfg(feature = "async")]
use crate::memory::target::frame::AsyncGcFrame;
use crate::{
    data::managed::{Managed, Ref},
    error::{JuliaResult, JuliaResultRef},
    memory::target::{frame::GcFrame, output::Output, unrooted::Unrooted}, prelude::Value,
};

/// Defines the return types of a target, `Data`, `Exception`, and `Result`.
pub trait TargetType<'target>: Sized {
    /// Type returned by methods that don't catch Julia exceptions.
    ///
    /// For rooting targets, this type is `T`.
    /// For non-rooting targets, this type is [`Ref<'target, 'data, T>`].
    type Data<'data, T: Managed<'target, 'data>>;

    /// Type returned by methods that catch Julia exceptions.
    ///
    /// For rooting targets, this type is [`JuliaResult<'target, 'data, T>`].
    /// For non-rooting targets, this type is [`JuliaResultRef<'target, 'data, Ref<'target, 'data, T>>`].
    type Result<'data, T: Managed<'target, 'data>>;

    /// Type returned by methods that don't return Julia data on succes, but can throw a Julia
    /// exception which is caught.
    ///
    /// For rooting targets, this type is [`JuliaResult<'target, 'data, T>`].
    /// For non-rooting targets, this type is [`JuliaResultRef<'target, 'data, T>`].
    type Exception<'data, T>;

    fn into_ok<'data, T: Managed<'target, 'data>>(data: Self::Data<'data, T>) -> Self::Result<'data, T>;
    fn into_err<'data, T: Managed<'target, 'data>>(data: Self::Data<'data, Value<'target, 'data>>) -> Self::Result<'data, T>;
}

impl<'target> TargetType<'target> for &mut GcFrame<'target> {
    type Data<'data, T: Managed<'target, 'data>> = T;
    type Result<'data, T: Managed<'target, 'data>> = JuliaResult<'target, 'data, T>;
    type Exception<'data, T> = JuliaResult<'target, 'data, T>;

    fn into_ok<'data, T: Managed<'target, 'data>>(data: Self::Data<'data, T>) -> Self::Result<'data, T> {
        Ok(data)
    }

    fn into_err<'data, T: Managed<'target, 'data>>(data: Self::Data<'data, Value<'target, 'data>>) -> Self::Result<'data, T> {
        Err(data)
    }
}

impl<'target> TargetType<'target> for GcFrame<'target> {
    type Data<'data, T: Managed<'target, 'data>> = T;
    type Result<'data, T: Managed<'target, 'data>> = JuliaResult<'target, 'data, T>;
    type Exception<'data, T> = JuliaResult<'target, 'data, T>;

    fn into_ok<'data, T: Managed<'target, 'data>>(data: Self::Data<'data, T>) -> Self::Result<'data, T> {
        Ok(data)
    }

    fn into_err<'data, T: Managed<'target, 'data>>(data: Self::Data<'data, Value<'target, 'data>>) -> Self::Result<'data, T> {
        Err(data)
    }
}

#[cfg(feature = "async")]
impl<'target> TargetType<'target> for &mut AsyncGcFrame<'target> {
    type Data<'data, T: Managed<'target, 'data>> = T;
    type Result<'data, T: Managed<'target, 'data>> = JuliaResult<'target, 'data, T>;
    type Exception<'data, T> = JuliaResult<'target, 'data, T>;

    fn into_ok<'data, T: Managed<'target, 'data>>(data: Self::Data<'data, T>) -> Self::Result<'data, T> {
        Ok(data)
    }

    fn into_err<'data, T: Managed<'target, 'data>>(data: Self::Data<'data, Value<'target, 'data>>) -> Self::Result<'data, T> {
        Err(data)
    }
}

#[cfg(feature = "async")]
impl<'target> TargetType<'target> for AsyncGcFrame<'target> {
    type Data<'data, T: Managed<'target, 'data>> = T;
    type Result<'data, T: Managed<'target, 'data>> = JuliaResult<'target, 'data, T>;
    type Exception<'data, T> = JuliaResult<'target, 'data, T>;

    fn into_ok<'data, T: Managed<'target, 'data>>(data: Self::Data<'data, T>) -> Self::Result<'data, T> {
        Ok(data)
    }

    fn into_err<'data, T: Managed<'target, 'data>>(data: Self::Data<'data, Value<'target, 'data>>) -> Self::Result<'data, T> {
        Err(data)
    }
}

impl<'target> TargetType<'target> for Output<'target> {
    type Data<'data, T: Managed<'target, 'data>> = T;
    type Result<'data, T: Managed<'target, 'data>> = JuliaResult<'target, 'data, T>;
    type Exception<'data, T> = JuliaResult<'target, 'data, T>;

    fn into_ok<'data, T: Managed<'target, 'data>>(data: Self::Data<'data, T>) -> Self::Result<'data, T> {
        Ok(data)
    }

    fn into_err<'data, T: Managed<'target, 'data>>(data: Self::Data<'data, Value<'target, 'data>>) -> Self::Result<'data, T> {
        Err(data)
    }
}

impl<'target> TargetType<'target> for &'target mut Output<'_> {
    type Data<'data, T: Managed<'target, 'data>> = T;
    type Result<'data, T: Managed<'target, 'data>> = JuliaResult<'target, 'data, T>;
    type Exception<'data, T> = JuliaResult<'target, 'data, T>;

    fn into_ok<'data, T: Managed<'target, 'data>>(data: Self::Data<'data, T>) -> Self::Result<'data, T> {
        Ok(data)
    }

    fn into_err<'data, T: Managed<'target, 'data>>(data: Self::Data<'data, Value<'target, 'data>>) -> Self::Result<'data, T> {
        Err(data)
    }
}

impl<'target> TargetType<'target> for ReusableSlot<'target> {
    type Data<'data, T: Managed<'target, 'data>> = T;
    type Result<'data, T: Managed<'target, 'data>> = JuliaResult<'target, 'data, T>;
    type Exception<'data, T> = JuliaResult<'target, 'data, T>;

    fn into_ok<'data, T: Managed<'target, 'data>>(data: Self::Data<'data, T>) -> Self::Result<'data, T> {
        Ok(data)
    }

    fn into_err<'data, T: Managed<'target, 'data>>(data: Self::Data<'data, Value<'target, 'data>>) -> Self::Result<'data, T> {
        Err(data)
    }
}

impl<'target> TargetType<'target> for &mut ReusableSlot<'target> {
    type Data<'data, T: Managed<'target, 'data>> = Ref<'target, 'data, T>;
    type Result<'data, T: Managed<'target, 'data>> =
        JuliaResultRef<'target, 'data, Ref<'target, 'data, T>>;
    type Exception<'data, T> = JuliaResultRef<'target, 'data, T>;

    fn into_ok<'data, T: Managed<'target, 'data>>(data: Self::Data<'data, T>) -> Self::Result<'data, T> {
        Ok(data)
    }

    fn into_err<'data, T: Managed<'target, 'data>>(data: Self::Data<'data, Value<'target, 'data>>) -> Self::Result<'data, T> {
        Err(data)
    }
}

impl<'target> TargetType<'target> for Unrooted<'target> {
    type Data<'data, T: Managed<'target, 'data>> = Ref<'target, 'data, T>;
    type Result<'data, T: Managed<'target, 'data>> =
        JuliaResultRef<'target, 'data, Ref<'target, 'data, T>>;
    type Exception<'data, T> = JuliaResultRef<'target, 'data, T>;

    fn into_ok<'data, T: Managed<'target, 'data>>(data: Self::Data<'data, T>) -> Self::Result<'data, T> {
        Ok(data)
    }

    fn into_err<'data, T: Managed<'target, 'data>>(data: Self::Data<'data, Value<'target, 'data>>) -> Self::Result<'data, T> {
        Err(data)
    }
}

impl<'target, U: TargetType<'target>> TargetType<'target> for &U {
    type Data<'data, T: Managed<'target, 'data>> = Ref<'target, 'data, T>;
    type Result<'data, T: Managed<'target, 'data>> =
        JuliaResultRef<'target, 'data, Ref<'target, 'data, T>>;
    type Exception<'data, T> = JuliaResultRef<'target, 'data, T>;

    fn into_ok<'data, T: Managed<'target, 'data>>(data: Self::Data<'data, T>) -> Self::Result<'data, T> {
        Ok(data)
    }

    fn into_err<'data, T: Managed<'target, 'data>>(data: Self::Data<'data, Value<'target, 'data>>) -> Self::Result<'data, T> {
        Err(data)
    }
}
