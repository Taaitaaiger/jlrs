//! Traits for rooting Julia data in a specific scope's frame
//!
//! When you take a look at the signatures of methods that return newly allocated Julia data, such
//! as [`Value::new`] and [`Array::new`], you'll notice that these methods don't take a mutable
//! reference to a frame, but a [`PartialScope`] and [`Scope`] respectively. Because you can't
//! access a parent frame from a nested scope, it's not possible to call methods that allocate
//! data with a parent frame. Instead, you need to reserve a slot in that frame by creating an
//! `Output` in advance.
//!
//! Both [`Output`] and mutable references to frames implement `PartialScope`. Methods that take a
//! `PartialScope` allocate a single value and use the provided implementation to root that value.
//! Methods that need to allocate and root temporary values take a `Scope`, while mutable
//! references to frames do implement this trait, `Output` does not because it can only be used to
//! root a single value once. An `Output` can be converted to an `OutputScope` by calling
//! [`Output::into_scope`] and providing it with a mutable reference to the current frame.
//!
//! The `Scope` trait provides a single method, [`Scope::split`]. This method splits the
//! implementation to an `Output` and a mutable reference to a frame. When a frame is used as a
//! `Scope`, the `Output` is reserved in that frame and the frame is the frame itself. If an
//! `OutputScope` is used, the existing `Output` and frame are returned.
//!  
//! A few examples:
//!
//! ```
//! # use jlrs::prelude::*;
//! # use jlrs::util::JULIA;
//! # #[cfg(feature = "lts")]
//! # fn main() {}
//! # #[cfg(not(feature = "lts"))]
//! # fn main() {
//! # JULIA.with(|j| {
//! # let mut julia = j.borrow_mut();
//! julia.scope(|global, frame| {
//!     // Value::new takes a partial scope, here a frame is used, so the
//!     // value is rooted in the current frame.
//!     let _i = Value::new(&mut *frame, 2u64)?;
//!     
//!     // We can also reserve an output in the current frame and use
//!     // that output. This has the same effect as the previous example.
//!     let output = frame.reserve_output()?;
//!     let _j = Value::new(output, 1u32)?;
//!     
//!     // Simarly, we can use an OutputScope because everything that
//!     // implements Scope implements PartialScope.
//!     let output = frame.reserve_output()?;
//!     let output_scope = output.into_scope(frame);
//!     let _k = Value::new(output_scope, 1u32)?;
//!
//!     // Using an output this way isn't particularly useful, because in all
//!     // the above examples the result is rooted in the current frame.
//!     // Outputs are more useful in combination with a nested scope.
//!     let output_a = frame.reserve_output()?;
//!     let output_b = frame.reserve_output()?;
//!     let (_array, _value) = frame.scope(|frame| {
//!         // By using the output from a nested scope, the data is rooted in
//!         // the parent frame and both these values can be returned from
//!         // this scope
//!         let output_scope = output_a.into_scope(frame);
//!         let array = Array::new::<f32, _, _, _>(output_scope, (3, 3))?
//!             .into_jlrs_result()?;
//!
//!         let value = Value::new(output_b, 3usize)?;
//!
//!         Ok((array, value))
//!     })?;
//!
//!     Ok(())
//! })
//! # .unwrap();
//! # });
//! # }
//! ```
//
//! [`Value::new`]: crate::wrappers::ptr::value::Value::new
//! [`Array::new`]: crate::wrappers::ptr::array::Array::new
//! [`Output::into_scope`]: crate::memory::output::Output::into_scope

use crate::{
    error::JlrsResult,
    memory::{
        frame::Frame,
        global::Global,
        output::{Output, OutputScope},
    },
};

/// This trait is used with functions that return Julia data rooted in some scope which need to
/// allocate and root temporary data. It's implemented by mutable references to implementations of
/// [`Frame`] and [`OutputScope`]. This trait provides a single method, `Scope::split` which
/// splits it into an output and the current frame. If this method is used with a mutable
/// reference to a frame, the output is reserved in that frame. If it's used with an `OutputScope`
/// the existing output is returned. In both cases the frame is the current frame, it's
/// recommended to create a nested scope to root temporary Julia data.
pub trait Scope<'target, 'current, F>:
    private::Scope<'target, 'current, F> + PartialScope<'target>
where
    F: Frame<'current>,
{
    /// Split the scope into an output and a frame. If the scope is a frame, the output is
    /// allocated in the current frame. If it's an [`OutputScope`], the existing output is
    /// returned.
    fn split<'own>(self) -> JlrsResult<(Output<'target>, &'own mut F)>
    where
        Self: 'own;
}

impl<'current, F> Scope<'current, 'current, F> for &mut F
where
    F: Frame<'current>,
{
    fn split<'own>(self) -> JlrsResult<(Output<'current>, &'own mut F)>
    where
        Self: 'own,
    {
        let output = self.reserve_output()?;
        Ok((output, self))
    }
}

impl<'target, 'current, 'borrow, F> Scope<'target, 'current, F>
    for OutputScope<'target, 'current, 'borrow, F>
where
    F: Frame<'current>,
{
    fn split<'own>(self) -> JlrsResult<(Output<'target>, &'own mut F)>
    where
        Self: 'own,
    {
        Ok((self.output, self.frame))
    }
}

/// This trait is used with functions that return Julia data rooted in some scope which don't need
/// to allocate and root temporary data. It's implemented by mutable references to implementations
/// of [`Frame`], [`OutputScope`], and [`Output`]. In the first case the data rooted in that
/// frame, in the other two cases the provided `Output` is used.
pub trait PartialScope<'target>: private::PartialScope<'target> {
    /// Returns a new `Global`.
    fn global(&self) -> Global<'target> {
        unsafe { Global::new() }
    }
}

impl<'target, F> PartialScope<'target> for &mut F where F: Frame<'target> {}

impl<'target> PartialScope<'target> for Output<'target> {}

impl<'target, 'current, 'inner, F> PartialScope<'target>
    for OutputScope<'target, 'current, 'inner, F>
where
    F: Frame<'current>,
{
}

pub(crate) mod private {
    use std::ptr::NonNull;

    use crate::{
        error::{JlrsError, JlrsResult, JuliaResult},
        memory::{frame::Frame, output::Output},
        private::Private,
        wrappers::ptr::value::Value,
    };
    use crate::{memory::output::OutputScope, wrappers::ptr::private::Wrapper};
    use jl_sys::jl_value_t;

    pub trait Scope<'target, 'current, F>: Sized + PartialScope<'target>
    where
        F: Frame<'current>,
    {
    }

    impl<'current, F> Scope<'current, 'current, F> for &mut F where F: Frame<'current> {}

    impl<'target, 'current, 'borrow, F> Scope<'target, 'current, F>
        for OutputScope<'target, 'current, 'borrow, F>
    where
        F: Frame<'current>,
    {
    }

    pub trait PartialScope<'target>: Sized {
        // safety: the value must be a valid pointer to a Julia value.
        unsafe fn value<'data, T: Wrapper<'target, 'data>>(
            self,
            value: NonNull<T::Wraps>,
            _: Private,
        ) -> JlrsResult<T>;

        // safety: the value must be a valid pointer to a Julia value.
        unsafe fn call_result<'data, T: Wrapper<'target, 'data>>(
            self,
            result: Result<NonNull<T::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> JlrsResult<JuliaResult<'target, 'data, T>>;
    }

    impl<'current, F> PartialScope<'current> for &mut F
    where
        F: Frame<'current>,
    {
        unsafe fn value<'data, T: Wrapper<'current, 'data>>(
            self,
            value: NonNull<T::Wraps>,
            _: Private,
        ) -> JlrsResult<T> {
            let v = self
                .push_root(value, Private)
                .map_err(JlrsError::alloc_error)?;

            Ok(v)
        }

        unsafe fn call_result<'data, T: Wrapper<'current, 'data>>(
            self,
            result: Result<NonNull<T::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> JlrsResult<JuliaResult<'current, 'data, T>> {
            match result {
                Ok(v) => {
                    let v = self.push_root(v, Private).map_err(JlrsError::alloc_error)?;
                    Ok(Ok(v))
                }
                Err(e) => {
                    let e = self.push_root(e, Private).map_err(JlrsError::alloc_error)?;
                    Ok(Err(e))
                }
            }
        }
    }

    impl<'target, 'current, 'borrow, F> PartialScope<'target>
        for OutputScope<'target, 'current, 'borrow, F>
    where
        F: Frame<'current>,
    {
        unsafe fn value<'data, T: Wrapper<'target, 'data>>(
            self,
            value: NonNull<T::Wraps>,
            _: Private,
        ) -> JlrsResult<T> {
            Ok(self.set_root(value))
        }

        unsafe fn call_result<'data, T: Wrapper<'target, 'data>>(
            self,
            result: Result<NonNull<T::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> JlrsResult<JuliaResult<'target, 'data, T>> {
            let rooted = match result {
                Ok(v) => Ok(self.set_root(v)),
                Err(e) => Err(self.set_root(e)),
            };

            Ok(rooted)
        }
    }

    impl<'target> PartialScope<'target> for Output<'target> {
        unsafe fn value<'data, T: Wrapper<'target, 'data>>(
            self,
            value: NonNull<T::Wraps>,
            _: Private,
        ) -> JlrsResult<T> {
            self.set_root::<T>(value);
            Ok(T::wrap_non_null(value, Private))
        }

        unsafe fn call_result<'data, T: Wrapper<'target, 'data>>(
            self,
            result: Result<NonNull<T::Wraps>, NonNull<jl_value_t>>,
            _: Private,
        ) -> JlrsResult<JuliaResult<'target, 'data, T>> {
            let rooted = match result {
                Ok(v) => {
                    self.set_root::<T>(v);
                    Ok(T::wrap_non_null(v, Private))
                }
                Err(e) => {
                    self.set_root::<Value>(e);
                    Err(Value::wrap_non_null(e, Private))
                }
            };

            Ok(rooted)
        }
    }
}
