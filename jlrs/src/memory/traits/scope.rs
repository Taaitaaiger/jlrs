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
//!
//! Scopes can be nested. Methods like [`Scope::value_scope`] and [`Scope::result_scope`] can be
//! used to create a value or call a Julia function from new closure and root the result in an
//! earlier frame, while [`ScopeExt::scope`] can be used to return arbitrary data.

use crate::{
    error::JlrsResult,
    memory::{
        frame::GcFrame,
        global::Global,
        output::{Output, OutputResult, OutputScope, OutputValue},
        traits::frame::Frame,
    },
    private::Private,
};

/// Provides `scope` and `scope_with_slots` methods to mutable references of types that implement
/// [`Frame`].
pub trait ScopeExt<'target, 'current, 'data, F: Frame<'current>>:
    Scope<'target, 'current, 'data, F>
{
    /// Create a [`GcFrame`] and call the given closure with it. Returns the result of this
    /// closure.
    ///
    /// Example:
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # use jlrs::util::JULIA;
    /// # fn main() {
    /// # JULIA.with(|j| {
    /// # let mut julia = j.borrow_mut();
    ///   julia.scope(|global, frame| {
    ///       let sum = frame.scope(|frame| {
    ///           let v1 = Value::new(&mut *frame, 1usize)?;
    ///           let v2 = Value::new(&mut *frame, 2usize)?;
    ///
    ///           Module::base(global)
    ///               .function(&mut *frame, "+")?
    ///               .call2(&mut *frame, v1, v2)?
    ///               .unwrap()
    ///               .unbox::<usize>()
    ///       })?;
    ///
    ///       assert_eq!(sum, 3);
    ///       Ok(())
    ///   }).unwrap();
    /// # });
    /// # }
    /// ```
    fn scope<T, G>(self, func: G) -> JlrsResult<T>
    where
        for<'inner> G: FnOnce(&mut GcFrame<'inner, F::Mode>) -> JlrsResult<T>;

    /// Create a [`GcFrame`] with `capacity` slots and call the given closure with it. Returns the
    /// result of this closure.
    ///
    /// Example:
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # use jlrs::util::JULIA;
    /// # fn main() {
    /// # JULIA.with(|j| {
    /// # let mut julia = j.borrow_mut();
    ///   julia.scope(|global, frame| {
    ///       let sum = frame.scope_with_slots(3, |frame| {
    ///           let v1 = Value::new(&mut *frame, 1usize)?;
    ///           let v2 = Value::new(&mut *frame, 2usize)?;
    ///
    ///           Module::base(global)
    ///               .function(&mut *frame, "+")?
    ///               .call2(&mut *frame, v1, v2)?
    ///               .unwrap()
    ///               .unbox::<usize>()
    ///       })?;
    ///
    ///       assert_eq!(sum, 3);
    ///       Ok(())
    ///   }).unwrap();
    /// # });
    /// # }
    /// ```
    fn scope_with_slots<T, G>(self, capacity: usize, func: G) -> JlrsResult<T>
    where
        for<'inner> G: FnOnce(&mut GcFrame<'inner, F::Mode>) -> JlrsResult<T>;
}

impl<'current, 'data, F: Frame<'current>> ScopeExt<'current, 'current, 'data, F> for &mut F {
    fn scope<T, G>(self, func: G) -> JlrsResult<T>
    where
        for<'inner> G: FnOnce(&mut GcFrame<'inner, F::Mode>) -> JlrsResult<T>,
    {
        F::scope(self, func, Private)
    }

    fn scope_with_slots<T, G>(self, capacity: usize, func: G) -> JlrsResult<T>
    where
        for<'inner> G: FnOnce(&mut GcFrame<'inner, F::Mode>) -> JlrsResult<T>,
    {
        F::scope_with_slots(self, capacity, func, Private)
    }
}

/// This trait is used to root raw Julia values in the current or an earlier frame. Scopes and
/// frames are very similar, in fact, all mutable references to frames are scopes: one that
/// targets that frame. The other implementor of this trait, [`OutputScope`], targets an earlier
/// frame. In addition to rooting values, this trait provides several methods that create a new
/// frame; if the scope is a frame, the frame's implementation of that method is called. If the
/// scope is an [`OutputScope`], the result is rooted the frame targeted by that scope.
pub trait Scope<'target, 'current, 'data, F>:
    Sized + private::Scope<'target, 'current, 'data, F>
where
    F: Frame<'current>,
{
    /// Create a new `Global`.
    fn global(&self) -> Global<'target> {
        unsafe { Global::new() }
    }

    /// Create a new `GcFrame` that can be used to root `capacity` values, an `Output` for the
    /// current scope, and use them to call the inner closure. The final result is not rooted in
    /// this newly created frame, but the current frame. The final result must not be the result
    /// of a function call, use [`Scope::result_scope`] for that purpose instead. If the current
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
    ///   julia.scope(|global, frame| {
    ///       let _nt = frame.value_scope(|output, frame| {
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
    fn value_scope<G>(self, func: G) -> JlrsResult<Self::Value>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'target>,
            &'inner mut GcFrame<'nested, F::Mode>,
        ) -> JlrsResult<OutputValue<'target, 'data, 'inner>>;

    /// Create a new `GcFrame` that can be used to root `capacity` values, an `Output` for the
    /// current scope, and use them to call the inner closure. The final result is not rooted in
    /// this newly created frame, but the current frame. The final result must be the result of a
    /// function call, if you want to create a new value use [`Scope::value_scope`] instead. If
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
    ///   julia.scope(|global, frame| {
    ///       let sum = frame.result_scope(|output, frame| {
    ///           let v1 = Value::new(&mut *frame, 1usize)?;
    ///           let v2 = Value::new(&mut *frame, 2usize)?;
    ///
    ///           let add = Module::base(global).function(&mut *frame, "+")?;
    ///
    ///           let output = output.into_scope(frame);
    ///           add.call2(output, v1, v2)
    ///       })?.unwrap().unbox::<usize>()?;
    ///
    ///       assert_eq!(sum, 3);
    ///       Ok(())
    ///   }).unwrap();
    /// # });
    /// # }
    /// ```
    fn result_scope<G>(self, func: G) -> JlrsResult<Self::JuliaResult>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'target>,
            &'inner mut GcFrame<'nested, F::Mode>,
        ) -> JlrsResult<OutputResult<'target, 'data, 'inner>>;

    /// Create a new `GcFrame` that can be used to root `capacity` values, an `Output` for the
    /// current scope, and use them to call the inner closure. The final result is not rooted in
    /// this newly created frame, but the current frame. The final result must not be the result
    /// of a function call, use [`Scope::result_scope`] for that purpose instead. If the current
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
    ///   julia.scope(|global, frame| {
    ///       let _nt = frame.value_scope(|output, frame| {
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
    fn value_scope_with_slots<G>(self, capacity: usize, func: G) -> JlrsResult<Self::Value>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'target>,
            &'inner mut GcFrame<'nested, F::Mode>,
        ) -> JlrsResult<OutputValue<'target, 'data, 'inner>>;

    /// Create a new `GcFrame` that can be used to root `capacity` values, an `Output` for the
    /// current scope, and use them to call the inner closure. The final result is not rooted in
    /// this newly created frame, but the current frame. The final result must be the result of a
    /// function call, if you want to create a new value use [`Scope::value_scope`] instead. If
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
    ///   julia.scope(|global, frame| {
    ///       let sum = frame.result_scope(|output, frame| {
    ///           let v1 = Value::new(&mut *frame, 1usize)?;
    ///           let v2 = Value::new(&mut *frame, 2usize)?;
    ///
    ///           let add = Module::base(global).function(&mut *frame, "+")?;
    ///
    ///           let output = output.into_scope(frame);
    ///           add.call2(output, v1, v2)
    ///       })?.unwrap().unbox::<usize>()?;
    ///
    ///       assert_eq!(sum, 3);
    ///       Ok(())
    ///   }).unwrap();
    /// # });
    /// # }
    /// ```
    fn result_scope_with_slots<G>(self, capacity: usize, func: G) -> JlrsResult<Self::JuliaResult>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'target>,
            &'inner mut GcFrame<'nested, F::Mode>,
        ) -> JlrsResult<OutputResult<'target, 'data, 'inner>>;
}

impl<'current, 'data, F: Frame<'current>> Scope<'current, 'current, 'data, F> for &mut F {
    /// Creates a [`GcFrame`] and calls the given closure with it. Returns the result of this
    /// closure.
    ///
    /// Example:
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # use jlrs::util::JULIA;
    /// # fn main() {
    /// # JULIA.with(|j| {
    /// # let mut julia = j.borrow_mut();
    /// julia.scope(|_global, frame| {
    ///     let _nt = frame.value_scope(|output, frame| {
    ///         let i = Value::new(&mut *frame, 2u64)?;
    ///         let j = Value::new(&mut *frame, 1u32)?;
    ///    
    ///         let output = output.into_scope(frame);
    ///         named_tuple!(output, "i" => i, "j" => j)
    ///     })?;
    ///
    ///     Ok(())
    /// }).unwrap();
    /// # });
    /// # }
    /// ```
    fn value_scope<G>(self, func: G) -> JlrsResult<Self::Value>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'current>,
            &'inner mut GcFrame<'nested, F::Mode>,
        ) -> JlrsResult<OutputValue<'current, 'data, 'inner>>,
    {
        F::value_scope(self, func, Private)
    }

    /// Creates a [`GcFrame`] and calls the given closure with it. Returns the result of this
    /// closure.
    ///
    /// Example:
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # use jlrs::util::JULIA;
    /// # fn main() {
    /// # JULIA.with(|j| {
    /// # let mut julia = j.borrow_mut();
    ///   julia.scope(|global, frame| {
    ///       let sum = frame.result_scope(|output, frame| {
    ///           let v1 = Value::new(&mut *frame, 1usize)?;
    ///           let v2 = Value::new(&mut *frame, 2usize)?;
    ///
    ///           let func = Module::base(global)
    ///               .function(&mut *frame, "+")?;
    ///
    ///           let output = output.into_scope(frame);
    ///           func.call2(output, v1, v2)
    ///       })?.unwrap()
    ///           .unbox::<usize>()?;
    ///
    ///       assert_eq!(sum, 3);
    ///       Ok(())
    ///   }).unwrap();
    /// # });
    /// # }
    /// ```
    fn result_scope<G>(self, func: G) -> JlrsResult<Self::JuliaResult>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'current>,
            &'inner mut GcFrame<'nested, F::Mode>,
        )
            -> JlrsResult<OutputResult<'current, 'data, 'inner>>,
    {
        F::result_scope(self, func, Private)
    }

    /// Creates a [`GcFrame`] and calls the given closure with it. Returns the result of this
    /// closure.
    ///
    /// Example:
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # use jlrs::util::JULIA;
    /// # fn main() {
    /// # JULIA.with(|j| {
    /// # let mut julia = j.borrow_mut();
    /// julia.scope(|_global, frame| {
    ///     let _nt = frame.value_scope_with_slots(2, |output, frame| {
    ///         let i = Value::new(&mut *frame, 2u64)?;
    ///         let j = Value::new(&mut *frame, 1u32)?;
    ///    
    ///         let output = output.into_scope(frame);
    ///         named_tuple!(output, "i" => i, "j" => j)
    ///     })?;
    ///
    ///     Ok(())
    /// }).unwrap();
    /// # });
    /// # }
    /// ```
    fn value_scope_with_slots<G>(self, capacity: usize, func: G) -> JlrsResult<Self::Value>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'current>,
            &'inner mut GcFrame<'nested, F::Mode>,
        ) -> JlrsResult<OutputValue<'current, 'data, 'inner>>,
    {
        F::value_scope_with_slots(self, capacity, func, Private)
    }

    /// Creates a [`GcFrame`] with `capacity` preallocated slots and calls the given closure with
    /// it. Returns the result of this closure.
    ///
    /// Example:
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # use jlrs::util::JULIA;
    /// # fn main() {
    /// # JULIA.with(|j| {
    /// # let mut julia = j.borrow_mut();
    ///   julia.scope(|global, frame| {
    ///       let sum = frame.result_scope_with_slots(2, |output, frame| {
    ///           let v1 = Value::new(&mut *frame, 1usize)?;
    ///           let v2 = Value::new(&mut *frame, 2usize)?;
    ///
    ///
    ///           let func = Module::base(global)
    ///               .function(&mut *frame, "+")?;
    ///
    ///           let output = output.into_scope(frame);
    ///           func.call2(output, v1, v2)
    ///       })?.unwrap()
    ///           .unbox::<usize>()?;
    ///
    ///       assert_eq!(sum, 3);
    ///       Ok(())
    ///   }).unwrap();
    /// # });
    /// # }
    /// ```
    fn result_scope_with_slots<G>(self, capacity: usize, func: G) -> JlrsResult<Self::JuliaResult>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'current>,
            &'inner mut GcFrame<'nested, F::Mode>,
        )
            -> JlrsResult<OutputResult<'current, 'data, 'inner>>,
    {
        F::result_scope_with_slots(self, capacity, func, Private)
    }
}

impl<'target, 'current, 'data, 'borrow, F: Frame<'current>> Scope<'target, 'current, 'data, F>
    for OutputScope<'target, 'current, 'borrow, F>
{
    fn value_scope<G>(self, func: G) -> JlrsResult<Self::Value>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'target>,
            &'inner mut GcFrame<'nested, F::Mode>,
        ) -> JlrsResult<OutputValue<'target, 'data, 'inner>>,
    {
        self.value_scope(func)
            .map(|ppv| OutputValue::wrap_non_null(ppv.unwrap_non_null()))
    }

    fn result_scope<G>(self, func: G) -> JlrsResult<Self::JuliaResult>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'target>,
            &'inner mut GcFrame<'nested, F::Mode>,
        ) -> JlrsResult<OutputResult<'target, 'data, 'inner>>,
    {
        self.result_scope(func)
    }

    fn value_scope_with_slots<G>(self, capacity: usize, func: G) -> JlrsResult<Self::Value>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'target>,
            &'inner mut GcFrame<'nested, F::Mode>,
        ) -> JlrsResult<OutputValue<'target, 'data, 'inner>>,
    {
        self.value_scope_with_slots(capacity, func)
            .map(|ppv| OutputValue::wrap_non_null(ppv.unwrap_non_null()))
    }

    fn result_scope_with_slots<G>(self, capacity: usize, func: G) -> JlrsResult<Self::JuliaResult>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'target>,
            &'inner mut GcFrame<'nested, F::Mode>,
        ) -> JlrsResult<OutputResult<'target, 'data, 'inner>>,
    {
        self.result_scope_with_slots(capacity, func)
    }
}

pub(crate) mod private {
    use std::ptr::NonNull;

    use crate::wrappers::ptr::value::Value;
    use crate::{
        error::{JlrsResult, JuliaResult},
        memory::{
            output::{OutputResult, OutputScope, OutputValue},
            traits::frame::Frame,
        },
        private::Private,
    };
    use jl_sys::jl_value_t;

    pub trait Scope<'target, 'current, 'data, F: Frame<'current>>: Sized {
        type Value: Sized;
        type JuliaResult: Sized;

        unsafe fn value(self, value: NonNull<jl_value_t>, _: Private) -> JlrsResult<Self::Value>;

        unsafe fn call_result(
            self,
            value: Result<NonNull<jl_value_t>, NonNull<jl_value_t>>,
            _: Private,
        ) -> JlrsResult<Self::JuliaResult>;
    }

    impl<'current, 'data, F: Frame<'current>> Scope<'current, 'current, 'data, F> for &mut F {
        type Value = Value<'current, 'data>;
        type JuliaResult = JuliaResult<'current, 'data>;

        unsafe fn value(self, value: NonNull<jl_value_t>, _: Private) -> JlrsResult<Self::Value> {
            self.push_root(value.cast(), Private).map_err(Into::into)
        }

        unsafe fn call_result(
            self,
            value: Result<NonNull<jl_value_t>, NonNull<jl_value_t>>,
            _: Private,
        ) -> JlrsResult<Self::JuliaResult> {
            match value {
                Ok(v) => self
                    .push_root(v, Private)
                    .map(|v| Ok(v))
                    .map_err(Into::into),
                Err(e) => self
                    .push_root(e, Private)
                    .map(|v| Err(v))
                    .map_err(Into::into),
            }
        }
    }

    impl<'target, 'current, 'data, 'inner, F: Frame<'current>> Scope<'target, 'current, 'data, F>
        for OutputScope<'target, 'current, 'inner, F>
    {
        type Value = OutputValue<'target, 'data, 'inner>;
        type JuliaResult = OutputResult<'target, 'data, 'inner>;

        unsafe fn value(self, value: NonNull<jl_value_t>, _: Private) -> JlrsResult<Self::Value> {
            Ok(OutputValue::wrap_non_null(value))
        }

        unsafe fn call_result(
            self,
            value: Result<NonNull<jl_value_t>, NonNull<jl_value_t>>,
            _: Private,
        ) -> JlrsResult<Self::JuliaResult> {
            match value {
                Ok(v) => Ok(OutputResult::Ok(OutputValue::wrap_non_null(v))),
                Err(e) => Ok(OutputResult::Err(OutputValue::wrap_non_null(e))),
            }
        }
    }
}
