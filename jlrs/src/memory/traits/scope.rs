//! Scopes are used to create new values, rooting them in a frame, and setting their lifetimes.
//!
//! Two kinds of scopes are provided by jlrs. The simplest are mutable references to things that
//! implement [`Frame`]. In this case, the value is rooted in the current frame and can be used
//! until the frame is dropped. The other kind of scope is the [`OutputScope`]. Such a scope
//! targets an earlier frame, the created value is left unrooted until returning to the targeted
//! frame.
//!
//! Methods that use a scope generally use them by value. This ensures an [`OutputScope`] can only
//! be used once, but also forces you to reborrow frames. If you don't, the Rust compiler
//! considers the frame to have been moved and you won't be able to use it again. Alternatively,
//! you can use [`Frame::as_scope`].

use crate::{
    error::JlrsResult,
    memory::{
        frame::GcFrame,
        global::Global,
        output::{Output, OutputScope},
        traits::frame::Frame,
    },
    value::{UnrootedCallResult, UnrootedValue},
};

/// This trait is used to root raw Julia values in the current or an earlier frame. Scopes and
/// frames are very similar, in fact, all mutable references to frames are scopes: one that
/// targets that frame. The other implementor of this trait, [`OutputScope`], targets an earlier
/// frame. In addition to rooting values, this trait provides several methods that create a new
/// frame; if the scope is a frame, the frame's implementation of that method is called. If the
/// scope is an [`OutputScope`], the result is rooted the frame targeted by that scope.
pub trait Scope<'scope, 'frame, 'data, F: Frame<'frame>>:
    Sized + private::Scope<'scope, 'frame, 'data, F>
{
    /// Create a new `Global`.
    fn global(&self) -> Global<'scope> {
        unsafe { Global::new() }
    }

    /// Create a new `GcFrame` that can be used to root `capacity` values, an `Output` for the
    /// current scope, and use them to call the inner closure. The final result is not rooted in
    /// this newly created frame, but the current frame. The final result must not be the result
    /// of a function call, use [`Scope::call_frame`] for that purpose instead. If the current
    /// scope is a mutable reference to a frame, calling this method will require one slot of the
    /// current frame.
    ///
    /// Example:
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # use jlrs::util::JULIA;
    /// # fn main() {
    /// # JULIA.with(|j| {
    /// # let mut julia = j.borrow_mut();
    ///   julia.frame(|global, frame| {
    ///       let _nt = frame.value_frame(|output, frame| {
    ///           let v1 = Value::new(&mut *frame, 1usize)?;
    ///           let v2 = Value::new(&mut *frame, 2usize)?;
    ///
    ///           let output = output.into_scope(frame);
    ///           named_tuple!(output, "a" => v1, "b" => v2)
    ///       })?;
    ///
    ///       Ok(())
    ///   }).unwrap();
    /// # });
    /// # }
    /// ```
    fn value_frame<G>(self, func: G) -> JlrsResult<Self::Value>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'scope>,
            &'inner mut GcFrame<'nested, F::Mode>,
        ) -> JlrsResult<UnrootedValue<'scope, 'data, 'inner>>;

    /// Create a new `GcFrame` that can be used to root `capacity` values, an `Output` for the
    /// current scope, and use them to call the inner closure. The final result is not rooted in
    /// this newly created frame, but the current frame. The final result must be the result of a
    /// function call, if you want to create a new value use [`Scope::value_frame`] instead. If
    /// the current scope is a mutable reference to a frame, calling this method will require one
    /// slot of the current frame.
    ///
    /// Example:
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # use jlrs::util::JULIA;
    /// # fn main() {
    /// # JULIA.with(|j| {
    /// # let mut julia = j.borrow_mut();
    ///   julia.frame(|global, frame| {
    ///       let sum = frame.call_frame(|output, frame| {
    ///           let v1 = Value::new(&mut *frame, 1usize)?;
    ///           let v2 = Value::new(&mut *frame, 2usize)?;
    ///
    ///           let add = Module::base(global).function("+")?;
    ///
    ///           let output = output.into_scope(frame);
    ///           add.call2(output, v1, v2)
    ///       })?.unwrap().cast::<usize>()?;
    ///
    ///       assert_eq!(sum, 3);
    ///       Ok(())
    ///   }).unwrap();
    /// # });
    /// # }
    /// ```
    fn call_frame<G>(self, func: G) -> JlrsResult<Self::CallResult>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'scope>,
            &'inner mut GcFrame<'nested, F::Mode>,
        )
            -> JlrsResult<UnrootedCallResult<'scope, 'data, 'inner>>;

    /// Create a new `GcFrame` that can be used to root `capacity` values, an `Output` for the
    /// current scope, and use them to call the inner closure. The final result is not rooted in
    /// this newly created frame, but the current frame. The final result must not be the result
    /// of a function call, use [`Scope::call_frame`] for that purpose instead. If the current
    /// scope is a mutable reference to a frame, calling this method will require one slot of the
    /// current frame.
    ///
    /// Example:
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # use jlrs::util::JULIA;
    /// # fn main() {
    /// # JULIA.with(|j| {
    /// # let mut julia = j.borrow_mut();
    ///   julia.frame(|global, frame| {
    ///       let _nt = frame.value_frame(|output, frame| {
    ///           let v1 = Value::new(&mut *frame, 1usize)?;
    ///           let v2 = Value::new(&mut *frame, 2usize)?;
    ///
    ///           let output = output.into_scope(frame);
    ///           named_tuple!(output, "a" => v1, "b" => v2)
    ///       })?;
    ///
    ///       Ok(())
    ///   }).unwrap();
    /// # });
    /// # }
    /// ```
    fn value_frame_with_slots<G>(self, capacity: usize, func: G) -> JlrsResult<Self::Value>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'scope>,
            &'inner mut GcFrame<'nested, F::Mode>,
        ) -> JlrsResult<UnrootedValue<'scope, 'data, 'inner>>;

    /// Create a new `GcFrame` that can be used to root `capacity` values, an `Output` for the
    /// current scope, and use them to call the inner closure. The final result is not rooted in
    /// this newly created frame, but the current frame. The final result must be the result of a
    /// function call, if you want to create a new value use [`Scope::value_frame`] instead. If
    /// the current scope is a mutable reference to a frame, calling this method will require one
    /// slot of the current frame.
    ///
    /// Example:
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # use jlrs::util::JULIA;
    /// # fn main() {
    /// # JULIA.with(|j| {
    /// # let mut julia = j.borrow_mut();
    ///   julia.frame(|global, frame| {
    ///       let sum = frame.call_frame(|output, frame| {
    ///           let v1 = Value::new(&mut *frame, 1usize)?;
    ///           let v2 = Value::new(&mut *frame, 2usize)?;
    ///
    ///           let add = Module::base(global).function("+")?;
    ///
    ///           let output = output.into_scope(frame);
    ///           add.call2(output, v1, v2)
    ///       })?.unwrap().cast::<usize>()?;
    ///
    ///       assert_eq!(sum, 3);
    ///       Ok(())
    ///   }).unwrap();
    /// # });
    /// # }
    /// ```
    fn call_frame_with_slots<G>(self, capacity: usize, func: G) -> JlrsResult<Self::CallResult>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'scope>,
            &'inner mut GcFrame<'nested, F::Mode>,
        )
            -> JlrsResult<UnrootedCallResult<'scope, 'data, 'inner>>;
}

impl<'frame, 'data, F: Frame<'frame>> Scope<'frame, 'frame, 'data, F> for &mut F {
    fn value_frame<G>(self, func: G) -> JlrsResult<Self::Value>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'frame>,
            &'inner mut GcFrame<'nested, F::Mode>,
        ) -> JlrsResult<UnrootedValue<'frame, 'data, 'inner>>,
    {
        self.value_frame(func)
    }

    fn call_frame<G>(self, func: G) -> JlrsResult<Self::CallResult>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'frame>,
            &'inner mut GcFrame<'nested, F::Mode>,
        )
            -> JlrsResult<UnrootedCallResult<'frame, 'data, 'inner>>,
    {
        self.call_frame(func)
    }

    fn value_frame_with_slots<G>(self, capacity: usize, func: G) -> JlrsResult<Self::Value>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'frame>,
            &'inner mut GcFrame<'nested, F::Mode>,
        ) -> JlrsResult<UnrootedValue<'frame, 'data, 'inner>>,
    {
        self.value_frame_with_slots(capacity, func)
    }

    fn call_frame_with_slots<G>(self, capacity: usize, func: G) -> JlrsResult<Self::CallResult>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'frame>,
            &'inner mut GcFrame<'nested, F::Mode>,
        )
            -> JlrsResult<UnrootedCallResult<'frame, 'data, 'inner>>,
    {
        self.call_frame_with_slots(capacity, func)
    }
}

impl<'scope, 'frame, 'data, 'borrow, F: Frame<'frame>> Scope<'scope, 'frame, 'data, F>
    for OutputScope<'scope, 'frame, 'borrow, F>
{
    fn value_frame<G>(self, func: G) -> JlrsResult<Self::Value>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'scope>,
            &'inner mut GcFrame<'nested, F::Mode>,
        ) -> JlrsResult<UnrootedValue<'scope, 'data, 'inner>>,
    {
        self.value_frame(func)
            .map(|ppv| UnrootedValue::new(ppv.ptr()))
    }

    fn call_frame<G>(self, func: G) -> JlrsResult<Self::CallResult>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'scope>,
            &'inner mut GcFrame<'nested, F::Mode>,
        )
            -> JlrsResult<UnrootedCallResult<'scope, 'data, 'inner>>,
    {
        self.call_frame(func)
    }

    fn value_frame_with_slots<G>(self, capacity: usize, func: G) -> JlrsResult<Self::Value>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'scope>,
            &'inner mut GcFrame<'nested, F::Mode>,
        ) -> JlrsResult<UnrootedValue<'scope, 'data, 'inner>>,
    {
        self.value_frame_with_slots(capacity, func)
            .map(|ppv| UnrootedValue::new(ppv.ptr()))
    }

    fn call_frame_with_slots<G>(self, capacity: usize, func: G) -> JlrsResult<Self::CallResult>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'scope>,
            &'inner mut GcFrame<'nested, F::Mode>,
        )
            -> JlrsResult<UnrootedCallResult<'scope, 'data, 'inner>>,
    {
        self.call_frame_with_slots(capacity, func)
    }
}

pub(crate) mod private {
    use crate::value::Value;
    use crate::{
        error::{CallResult, JlrsResult},
        memory::{output::OutputScope, traits::frame::Frame},
        value::{traits::private::Internal, UnrootedCallResult, UnrootedValue},
    };
    use jl_sys::jl_value_t;

    pub trait Scope<'scope, 'frame, 'data, F: Frame<'frame>>: Sized {
        type Value: Sized;
        type CallResult: Sized;

        unsafe fn value(self, value: *mut jl_value_t, _: Internal) -> JlrsResult<Self::Value>;

        unsafe fn call_result(
            self,
            value: Result<*mut jl_value_t, *mut jl_value_t>,
            _: Internal,
        ) -> JlrsResult<Self::CallResult>;
    }

    impl<'frame, 'data, F: Frame<'frame>> Scope<'frame, 'frame, 'data, F> for &mut F {
        type Value = Value<'frame, 'data>;
        type CallResult = CallResult<'frame, 'data>;

        unsafe fn value(self, value: *mut jl_value_t, _: Internal) -> JlrsResult<Self::Value> {
            self.push_root(value, Internal).map_err(Into::into)
        }

        unsafe fn call_result(
            self,
            value: Result<*mut jl_value_t, *mut jl_value_t>,
            _: Internal,
        ) -> JlrsResult<Self::CallResult> {
            match value {
                Ok(v) => self
                    .push_root(v, Internal)
                    .map(|v| Ok(v))
                    .map_err(Into::into),
                Err(e) => self
                    .push_root(e, Internal)
                    .map(|v| Err(v))
                    .map_err(Into::into),
            }
        }
    }

    impl<'scope, 'frame, 'data, 'inner, F: Frame<'frame>> Scope<'scope, 'frame, 'data, F>
        for OutputScope<'scope, 'frame, 'inner, F>
    {
        type Value = UnrootedValue<'scope, 'data, 'inner>;
        type CallResult = UnrootedCallResult<'scope, 'data, 'inner>;

        unsafe fn value(self, value: *mut jl_value_t, _: Internal) -> JlrsResult<Self::Value> {
            Ok(UnrootedValue::new(value))
        }

        unsafe fn call_result(
            self,
            value: Result<*mut jl_value_t, *mut jl_value_t>,
            _: Internal,
        ) -> JlrsResult<Self::CallResult> {
            match value {
                Ok(v) => Ok(UnrootedCallResult::Ok(UnrootedValue::new(v))),
                Err(e) => Ok(UnrootedCallResult::Err(UnrootedValue::new(e))),
            }
        }
    }
}
