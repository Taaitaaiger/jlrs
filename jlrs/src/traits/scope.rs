//! Scopes are used to create new values, rooting them in a frame, and setting their lifetimes.

use super::Frame;
use crate::error::JlrsResult;
use crate::frame::{Output, OutputScope, UnrootedCallResult, UnrootedValue, StaticFrame};
use crate::global::Global;

/// When you create a new Julia value or call a Julia function, the C API generally returns a
/// pointer to some data. The garbage collector is essentially the owner of this pointer and will
/// clean it up when it's no longer in use. In order to make the garbage collector aware of the
/// values that are in use from Rust, they must be rooted in a frame. The value can then be used
/// until the frame is popped. 
///
/// One common pattern in the Julia C API is to create a new frame to hold temporary values, call
/// a function, pop the frame and protect the result in the previous frame. Thanks to `Scope` this
/// same pattern can be used in jlrs.
///
/// `Scope` is implemented for [`OutputScope`] and any mutable reference to something that 
/// implements [`Frame`]. This means that the mutable reference to [`StaticFrame`] you use when
/// calling a method like [`Julia::frame`] is a valid scope, specifically one that uses itself to 
/// root a created value immediately, turning it into a `Value` that can be used until the frame
/// has been popped. In order to root a value in a previous frame you will need an 
/// [`OutputScope`]. To get such a scope you will need to use a method like [`Frame::value_frame`] 
/// or [`Frame::call_frame`], which both take a closure that provides you with an [`Output`] and a
/// mutable reference to a frame. This output can be converted to an [`OutputScope`] by calling 
/// [`Output:into_scope`]. The frame can now no longer be used, the scope can be used once and its 
/// result is propagated back to the frame that originally called [`Frame::value_frame`] or 
/// [`Frame::call_frame`], where it is finally rooted. 
///
/// NB: Methods that use a scope take them by value, when you use a mutable frame as a scope 
/// you'll need to reborrow it, `func(&mut *frame)` if you use it more than once.
pub trait Scope<'scope, 'frame, 'data, F: Frame<'frame>>:
    Sized + private::Scope<'scope, 'frame, 'data, F>
{
    /// Create a new `Global`.
    fn global(&self) -> Global<'scope> {
        unsafe { Global::new() }
    }

    /// Create a new `StaticFrame` that can be used to root `capacity` values, an `Output` for the
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
    ///   julia.frame(1, |global, frame| {
    ///       let _nt = frame.value_frame(2, |output, frame| {
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
    fn value_frame<G>(self, capacity: usize, func: G) -> JlrsResult<Self::Value>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'scope>,
            &'inner mut StaticFrame<'nested, F::Mode>,
        )
            -> JlrsResult<UnrootedValue<'scope, 'data, 'inner>>;

    fn frame<T, G: for<'nested> FnOnce(&mut StaticFrame<'nested, F::Mode>) -> JlrsResult<T>>(
        &mut self,
        capacity: usize,
        func: G,
    ) -> JlrsResult<T> { 
        todo!()
    }


    /// Create a new `StaticFrame` that can be used to root `capacity` values, an `Output` for the
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
    ///   julia.frame(1, |global, frame| {
    ///       let sum = frame.call_frame(2, |output, frame| {
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
    fn call_frame<G>(self, capacity: usize, func: G) -> JlrsResult<Self::CallResult>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'scope>,
            &'inner mut StaticFrame<'nested, F::Mode>,
        ) -> JlrsResult<
            UnrootedCallResult<'scope, 'data, 'inner>,
        >;
}

impl<'frame, 'data, F: Frame<'frame>> Scope<'frame, 'frame, 'data, F> for &mut F {
    fn value_frame<G>(self, capacity: usize, func: G) -> JlrsResult<Self::Value>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'frame>,
            &'inner mut StaticFrame<'nested, F::Mode>,
        )
            -> JlrsResult<UnrootedValue<'frame, 'data, 'inner>>,
    {
        self.value_frame(capacity, func)
    }

    fn call_frame<G>(self, capacity: usize, func: G) -> JlrsResult<Self::CallResult>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'frame>,
            &'inner mut StaticFrame<'nested, F::Mode>,
        ) -> JlrsResult<
            UnrootedCallResult<'frame, 'data, 'inner>,
        >,
    {
        self.call_frame(capacity, func)
    }
}

impl<'scope, 'frame, 'data, 'borrow, F: Frame<'frame>> Scope<'scope, 'frame, 'data, F>
    for OutputScope<'scope, 'frame, 'borrow, F>
{
    fn value_frame<G>(self, capacity: usize, func: G) -> JlrsResult<Self::Value>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'scope>,
            &'inner mut StaticFrame<'nested, F::Mode>,
        )
            -> JlrsResult<UnrootedValue<'scope, 'data, 'inner>>,
    {
        self.value_frame(capacity, func)
            .map(|ppv| UnrootedValue::new(ppv.inner()))
    }

    fn call_frame<G>(self, capacity: usize, func: G) -> JlrsResult<Self::CallResult>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'scope>,
            &'inner mut StaticFrame<'nested, F::Mode>,
        ) -> JlrsResult<
            UnrootedCallResult<'scope, 'data, 'inner>,
        >,
    {
        self.call_frame(capacity, func)
    }
}

pub(crate) mod private {
    use crate::error::{CallResult, JlrsResult};
    use crate::frame::{OutputScope, UnrootedCallResult, UnrootedValue};
    use crate::traits::private::Internal;
    use crate::traits::Frame;
    use crate::value::Value;
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
            self.root(value, Internal).map_err(Into::into)
        }

        unsafe fn call_result(
            self,
            value: Result<*mut jl_value_t, *mut jl_value_t>,
            _: Internal,
        ) -> JlrsResult<Self::CallResult> {
            match value {
                Ok(v) => self.root(v, Internal).map(|v| Ok(v)).map_err(Into::into),
                Err(e) => self
                    .root(e, Internal)
                    .map(|v| Err(v))
                    .map_err(Into::into),
            }
        }
    }

    impl<'scope, 'frame, 'data, 'borrow, F: Frame<'frame>> Scope<'scope, 'frame, 'data, F>
        for OutputScope<'scope, 'frame, 'borrow, F>
    {
        type Value = UnrootedValue<'scope, 'data, 'borrow>;
        type CallResult = UnrootedCallResult<'scope, 'data, 'borrow>;

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
