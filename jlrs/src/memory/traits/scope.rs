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
    convert::cast::Cast,
    error::JlrsResult,
    memory::{
        frame::GcFrame,
        global::Global,
        output::{Output, OutputScope},
        traits::frame::Frame,
    },
    private::Private,
    value::{traits::wrapper::Wrapper, wrapper_ref::WrapperRef, UnrootedResult, UnrootedValue},
};

/// Provides `scope` and `scope_with_slots` methods to mutable references of types that implement
/// [`Frame`].
pub trait ScopeExt<'outer, 'scope, 'frame, 'data, F: Frame<'frame>>:
    Scope<'scope, 'frame, 'data, F>
where
    Self: 'outer,
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
    ///               .function("+")?
    ///               .call2(&mut *frame, v1, v2)?
    ///               .unwrap()
    ///               .cast::<usize>()
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
        T: 'outer,
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
    ///               .function("+")?
    ///               .call2(&mut *frame, v1, v2)?
    ///               .unwrap()
    ///               .cast::<usize>()
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
        T: 'outer,
        for<'inner> G: FnOnce(&mut GcFrame<'inner, F::Mode>) -> JlrsResult<T>;

    fn root_reference<T, U>(
        self,
        func: fn(T) -> WrapperRef<'frame, 'data, U>,
        t: T,
    ) -> JlrsResult<Option<<U as Cast<'frame, 'data>>::Output>>
    where
        T: Wrapper<'frame, 'data>,
        U: Wrapper<'frame, 'data> + Cast<'frame, 'data>;
}

impl<'outer, 'frame, 'data, F: Frame<'frame>> ScopeExt<'outer, 'frame, 'frame, 'data, F>
    for &'outer mut F
{
    fn scope<T, G>(self, func: G) -> JlrsResult<T>
    where
        T: 'outer,
        for<'inner> G: FnOnce(&mut GcFrame<'inner, F::Mode>) -> JlrsResult<T>,
    {
        F::scope(self, func, Private)
    }

    fn scope_with_slots<T, G>(self, capacity: usize, func: G) -> JlrsResult<T>
    where
        T: 'outer,
        for<'inner> G: FnOnce(&mut GcFrame<'inner, F::Mode>) -> JlrsResult<T>,
    {
        F::scope_with_slots(self, capacity, func, Private)
    }

    fn root_reference<T, U>(
        self,
        func: fn(T) -> WrapperRef<'frame, 'data, U>,
        t: T,
    ) -> JlrsResult<Option<<U as Cast<'frame, 'data>>::Output>>
    where
        T: Wrapper<'frame, 'data>,
        U: Wrapper<'frame, 'data> + Cast<'frame, 'data>,
    {
        unsafe {
            if let Some(v) = func(t).assume_reachable_value() {
                match v.root(self) {
                    Ok(v) => v.cast::<U>().map(|u| Some(u)),
                    Err(e) => Err(e),
                }
            } else {
                Ok(None)
            }
        }
    }
}

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
            Output<'scope>,
            &'inner mut GcFrame<'nested, F::Mode>,
        ) -> JlrsResult<UnrootedValue<'scope, 'data, 'inner>>;

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
    fn result_scope<G>(self, func: G) -> JlrsResult<Self::JuliaResult>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'scope>,
            &'inner mut GcFrame<'nested, F::Mode>,
        )
            -> JlrsResult<UnrootedResult<'scope, 'data, 'inner>>;

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
            Output<'scope>,
            &'inner mut GcFrame<'nested, F::Mode>,
        ) -> JlrsResult<UnrootedValue<'scope, 'data, 'inner>>;

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
    fn result_scope_with_slots<G>(self, capacity: usize, func: G) -> JlrsResult<Self::JuliaResult>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'scope>,
            &'inner mut GcFrame<'nested, F::Mode>,
        )
            -> JlrsResult<UnrootedResult<'scope, 'data, 'inner>>;
}

impl<'frame, 'data, F: Frame<'frame>> Scope<'frame, 'frame, 'data, F> for &mut F {
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
            Output<'frame>,
            &'inner mut GcFrame<'nested, F::Mode>,
        ) -> JlrsResult<UnrootedValue<'frame, 'data, 'inner>>,
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
    ///           let output = output.into_scope(frame);
    ///
    ///           Module::base(global)
    ///               .function("+")?
    ///               .call2(output, v1, v2)
    ///       })?.unwrap()
    ///           .cast::<usize>()?;
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
            Output<'frame>,
            &'inner mut GcFrame<'nested, F::Mode>,
        )
            -> JlrsResult<UnrootedResult<'frame, 'data, 'inner>>,
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
            Output<'frame>,
            &'inner mut GcFrame<'nested, F::Mode>,
        ) -> JlrsResult<UnrootedValue<'frame, 'data, 'inner>>,
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
    ///           let output = output.into_scope(frame);
    ///
    ///           Module::base(global)
    ///               .function("+")?
    ///               .call2(output, v1, v2)
    ///       })?.unwrap()
    ///           .cast::<usize>()?;
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
            Output<'frame>,
            &'inner mut GcFrame<'nested, F::Mode>,
        )
            -> JlrsResult<UnrootedResult<'frame, 'data, 'inner>>,
    {
        F::result_scope_with_slots(self, capacity, func, Private)
    }
}

impl<'scope, 'frame, 'data, 'borrow, F: Frame<'frame>> Scope<'scope, 'frame, 'data, F>
    for OutputScope<'scope, 'frame, 'borrow, F>
{
    fn value_scope<G>(self, func: G) -> JlrsResult<Self::Value>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'scope>,
            &'inner mut GcFrame<'nested, F::Mode>,
        ) -> JlrsResult<UnrootedValue<'scope, 'data, 'inner>>,
    {
        self.value_scope(func)
            .map(|ppv| UnrootedValue::new(ppv.ptr()))
    }

    fn result_scope<G>(self, func: G) -> JlrsResult<Self::JuliaResult>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'scope>,
            &'inner mut GcFrame<'nested, F::Mode>,
        )
            -> JlrsResult<UnrootedResult<'scope, 'data, 'inner>>,
    {
        self.result_scope(func)
    }

    fn value_scope_with_slots<G>(self, capacity: usize, func: G) -> JlrsResult<Self::Value>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'scope>,
            &'inner mut GcFrame<'nested, F::Mode>,
        ) -> JlrsResult<UnrootedValue<'scope, 'data, 'inner>>,
    {
        self.value_scope_with_slots(capacity, func)
            .map(|ppv| UnrootedValue::new(ppv.ptr()))
    }

    fn result_scope_with_slots<G>(self, capacity: usize, func: G) -> JlrsResult<Self::JuliaResult>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'scope>,
            &'inner mut GcFrame<'nested, F::Mode>,
        )
            -> JlrsResult<UnrootedResult<'scope, 'data, 'inner>>,
    {
        self.result_scope_with_slots(capacity, func)
    }
}

pub(crate) mod private {
    use crate::value::Value;
    use crate::{
        error::{JlrsResult, JuliaResult},
        memory::{output::OutputScope, traits::frame::Frame},
        private::Private,
        value::{UnrootedResult, UnrootedValue},
    };
    use jl_sys::jl_value_t;

    pub trait Scope<'scope, 'frame, 'data, F: Frame<'frame>>: Sized {
        type Value: Sized;
        type JuliaResult: Sized;

        unsafe fn value(self, value: *mut jl_value_t, _: Private) -> JlrsResult<Self::Value>;

        unsafe fn call_result(
            self,
            value: Result<*mut jl_value_t, *mut jl_value_t>,
            _: Private,
        ) -> JlrsResult<Self::JuliaResult>;
    }

    impl<'frame, 'data, F: Frame<'frame>> Scope<'frame, 'frame, 'data, F> for &mut F {
        type Value = Value<'frame, 'data>;
        type JuliaResult = JuliaResult<'frame, 'data>;

        unsafe fn value(self, value: *mut jl_value_t, _: Private) -> JlrsResult<Self::Value> {
            self.push_root(value, Private).map_err(Into::into)
        }

        unsafe fn call_result(
            self,
            value: Result<*mut jl_value_t, *mut jl_value_t>,
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

    impl<'scope, 'frame, 'data, 'inner, F: Frame<'frame>> Scope<'scope, 'frame, 'data, F>
        for OutputScope<'scope, 'frame, 'inner, F>
    {
        type Value = UnrootedValue<'scope, 'data, 'inner>;
        type JuliaResult = UnrootedResult<'scope, 'data, 'inner>;

        unsafe fn value(self, value: *mut jl_value_t, _: Private) -> JlrsResult<Self::Value> {
            Ok(UnrootedValue::new(value))
        }

        unsafe fn call_result(
            self,
            value: Result<*mut jl_value_t, *mut jl_value_t>,
            _: Private,
        ) -> JlrsResult<Self::JuliaResult> {
            match value {
                Ok(v) => Ok(UnrootedResult::Ok(UnrootedValue::new(v))),
                Err(e) => Ok(UnrootedResult::Err(UnrootedValue::new(e))),
            }
        }
    }
}
