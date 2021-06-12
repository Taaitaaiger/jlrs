//! Scopes provide a nestable context in which Julia can be called.
//!
//! All interactions with Julia happen inside a scope. A base scope can be created with
//! [`Julia::scope`] and [`Julia::scope_with_slots`], these methods take a closure which is
//! called inside these methods after creating the arguments it requires: a [`Global`] and a
//! mutable reference to a [`GcFrame`]. Each scope has exactly one [`GcFrame`] which is dropped 
//! when you leave the scope. Any value which is rooted in this frame or can be reached from a
//! root will not be freed by the garbage collector until the frame has been dropped, it's valid
//! for the rest of the scope associated with that frame. 
//!
//! Holding on to many roots will slow down the garbage collector. Scanning will be slower because
//! more values can be reached, and because this data is not freed memory pressure increases 
//! causing the garbage collector to run more often. In order to manage the number of roots, it's
//! possible to create nested scopes with their own `GcFrame`. The frames form a stack, when a
//! nested scope is created the new frame is constructed at the top of this stack. This kind of 
//! functionality is provided by the two traits in this module, [`Scope`] and [`ScopeExt`].
//!
//! There are several ways to create a nested scope. The easiest is [`ScopeExt::scope`], which 
//! behaves the same way as [`Julia::scope`]. This method is relatively limited in the sense that
//! it cannot be used to create a new value inside this new scope and root it in the frame of 
//! parent scope. Several methods are available to handle that case, which is particularly useful 
//! if you want to create a new value or call a function with some temporary values. These methods 
//! are [`Scope::value_scope`], used to allocate a value in a nested scope and root it in the 
//! frame of a parent scope; [`Scope::result_scope`], used to call a function in a nested scope 
//! and root the result in the frame of a parent scope; [`ScopeExt::wrapper_scope`] and 
//! [`ScopeExt::wrapper_result_scope`] are also available, they do the same thing as the previous
//! two methods but they will cast the result to the given wrapper type before returning it.
//!
//! Two traits exist because there are two implementors of [`Scope`] and they behave differently.
//! The first implementor is all mutable references to types that implement the [`Frame`] trait,
//! [`ScopeExt`] is also implemented. Methods that create new values that must be rooted usually
//! take an argument that implements `Scope`, when a mutable reference to a frame is used the
//! value is rooted in that frame. Because the scope is taken by value and mutable references 
//! don't implement `Copy`, it's necessary to mutably reborrow the frame when calling these 
//! methods to prevent the frame from moving. These methods only care about the fact that it's a
//! mutable reference to a frame, not the duration of that borrow.
//!
//! The other implementor, [`OutputScope`], is used in nested scopes that root a value in the 
//! frame of a parent scope. It doesn't implement [`ScopeExt`]. As mentioned before, frames form a
//! stack and the frame of a nested scope is constructed on top of its parent. Due to this design,
//! it's not possible to directly root a value in some ancestral frame. Rather, rooting has to be
//! postponed until the target frame is the active frame again. 
//!
//! Methods that root a value in the frame of a parent scope take a closure with two arguments, an
//! [`Output`] and a mutable reference to a [`GcFrame`]. The frame can be used to root temporary 
//! values, once all temporary values have been created and there's nothing else that needs to be
//! rooted in the current frame, the `Output` can be converted to an `OutputScope`. Unlike frames,
//! the [`Scope`] trait is not implemented for a mutable reference but for `OutputScope` itself. 
//! Because it implements this trait it can be nested. In this case the output is propagated to 
//! the new scope, ie the the result still targets the same scope. An `OutputScope` can be used 
//! a single time to create a new value, this value is left unrooted until the target scope is 
//! reached.
//!
//! You should always immediately return such an unrooted value from the closure without calling
//! any function in jlrs, even those that don't take a frame or a scope as an argument. Functions
//! that call the C API but don't take a scope or frame can still allocate new values internally,
//! which can trigger a garbage collection cycle. Because an unrooted value exists which is likely
//! unreachable, such a cycle can free the value that has just been created.

use crate::{
    error::{JlrsResult, JuliaResult},
    layout::typecheck::Typecheck,
    memory::{
        frame::GcFrame,
        global::Global,
        output::{Output, OutputResult, OutputScope, OutputValue},
        frame::Frame,
    },
    private::Private,
    wrappers::ptr::Wrapper,
};

/// Extension for [`Scope`] implemented by mutable references to frames. It offers methods like
/// [`ScopeExt::scope`], [`ScopeExt::wrapper_scope`], and [`ScopeExt::wrapper_result_scope`] that
/// can be used to create a nested scope that returns arbitrary data, and to automatically call
/// cast before returning a Julia value or result respectively.
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

    /// The same as [`Scope::value_scope`], but the value is cast to `T` before returning it.
    fn wrapper_scope<T, G>(self, func: G) -> JlrsResult<T>
    where
        T: Wrapper<'current, 'data> + Typecheck,
        for<'nested, 'inner> G: FnOnce(
            Output<'current>,
            &'inner mut GcFrame<'nested, F::Mode>,
        ) -> JlrsResult<OutputValue<'current, 'data, 'inner>>;

    /// The same as [`Scope::value_scope_with_slots`], but the value is cast to `T` before
    /// returning it.
    fn wrapper_scope_with_slots<T, G>(self, capacity: usize, func: G) -> JlrsResult<T>
    where
        T: Wrapper<'current, 'data> + Typecheck,
        for<'nested, 'inner> G: FnOnce(
            Output<'current>,
            &'inner mut GcFrame<'nested, F::Mode>,
        ) -> JlrsResult<OutputValue<'current, 'data, 'inner>>;

    /// The same as [`Scope::result_scope`], on success the result is cast to `T` before returning 
    /// it.
    fn wrapper_result_scope<T, G>(self, func: G) -> JlrsResult<JuliaResult<'current, 'data, T>>
    where
        T: Wrapper<'current, 'data> + Typecheck,
        for<'nested, 'inner> G: FnOnce(
            Output<'current>,
            &'inner mut GcFrame<'nested, F::Mode>,
        )
            -> JlrsResult<OutputResult<'current, 'data, 'inner>>;

    /// The same as [`Scope::result_scope_with_slots`], on success the result is cast to `T` 
    /// before returning it.
    fn wrapper_result_scope_with_slots<T, G>(
        self,
        capacity: usize,
        func: G,
    ) -> JlrsResult<JuliaResult<'current, 'data, T>>
    where
        T: Wrapper<'current, 'data> + Typecheck,
        for<'nested, 'inner> G: FnOnce(
            Output<'current>,
            &'inner mut GcFrame<'nested, F::Mode>,
        )
            -> JlrsResult<OutputResult<'current, 'data, 'inner>>;
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

    fn wrapper_scope<T, G>(self, func: G) -> JlrsResult<T>
    where
        T: Wrapper<'current, 'data> + Typecheck,
        for<'nested, 'inner> G: FnOnce(
            Output<'current>,
            &'inner mut GcFrame<'nested, F::Mode>,
        ) -> JlrsResult<OutputValue<'current, 'data, 'inner>>,
    {
        F::value_scope(self, func, Private)?.cast::<T>()
    }

    fn wrapper_scope_with_slots<T, G>(self, capacity: usize, func: G) -> JlrsResult<T>
    where
        T: Wrapper<'current, 'data> + Typecheck,
        for<'nested, 'inner> G: FnOnce(
            Output<'current>,
            &'inner mut GcFrame<'nested, F::Mode>,
        ) -> JlrsResult<OutputValue<'current, 'data, 'inner>>,
    {
        F::value_scope_with_slots(self, capacity, func, Private)?.cast::<T>()
    }

    fn wrapper_result_scope<T, G>(self, func: G) -> JlrsResult<JuliaResult<'current, 'data, T>>
    where
        T: Wrapper<'current, 'data> + Typecheck,
        for<'nested, 'inner> G: FnOnce(
            Output<'current>,
            &'inner mut GcFrame<'nested, F::Mode>,
        )
            -> JlrsResult<OutputResult<'current, 'data, 'inner>>,
    {
        match F::result_scope(self, func, Private)? {
            Ok(v) => Ok(Ok(v.cast::<T>()?)),
            Err(e) => Ok(Err(e)),
        }
    }

    fn wrapper_result_scope_with_slots<T, G>(
        self,
        capacity: usize,
        func: G,
    ) -> JlrsResult<JuliaResult<'current, 'data, T>>
    where
        T: Wrapper<'current, 'data> + Typecheck,
        for<'nested, 'inner> G: FnOnce(
            Output<'current>,
            &'inner mut GcFrame<'nested, F::Mode>,
        )
            -> JlrsResult<OutputResult<'current, 'data, 'inner>>,
    {
        match F::result_scope_with_slots(self, capacity, func, Private)? {
            Ok(v) => Ok(Ok(v.cast::<T>()?)),
            Err(e) => Ok(Err(e)),
        }
    }
}

/// Trait that provides methods to create nested scopes which eventually root a value in the frame 
/// of a target scope. It's implemented for [`OutputScope`] and mutable references to implementors
/// of [`Frame`]. In addition to nesting, many methods that allocate a new value take an
/// implementation of this trait as their first argument. If a mutable reference to a frame is 
/// used this way, the value is rooted in that frame. If it's an [`OutputScope`], it's rooted in 
/// the frame for which the [`Output`] was originally created.
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
            frame::Frame,
        },
        private::Private,
    };
    use jl_sys::jl_value_t;

    pub trait Scope<'target, 'current, 'data, F: Frame<'current>>: Sized {
        type Value: Sized;
        type JuliaResult: Sized;

        // safety: the value must be a valid pointer to a Julia value.
        unsafe fn value(self, value: NonNull<jl_value_t>, _: Private) -> JlrsResult<Self::Value>;

        // safety: the value must be a valid pointer to a Julia value.
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
