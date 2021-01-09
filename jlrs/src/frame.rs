//! Frames protect values from garbage collection.
//!
//! Julia's garbage collector essentially owns all Julia data, and frees it when it's no longer in
//! use. In order to indicate that some value is in use, it must be rooted in a frame. While the
//! frame exists, the value is protected from garbage collection.
//!
//! Four different kinds of frames exist; [`StaticFrame`], [`DynamicFrame`], [`NullFrame`], and
//! [`DynamicAsyncFrame`]. The first two of them can be nested and freely mixed. The main difference
//! between those two is that a [`StaticFrame`] is created with a definite capacity, while a
//! [`DynamicFrame`] will dynamically grow its capacity whenever a value is created or a function
//! is called. A [`StaticFrame`] is more efficient, a [`DynamicFrame`] is easier to use.
//!
//! The third type, [`NullFrame`] can only be used if you call Rust from Julia. They don't
//! allocate at all and can only be used to borrow array data.
//!
//! The final type, [`DynamicAsyncFrame`] is only available when you use the async runtime. Structs that
//! implement [`JuliaTask`] can use this kind of frame in the `run`-method. It's essentially a
//! [`DynamicFrame`] with the additional feature that it can be used to call
//! [`Value::call_async`].
//!
//! Frames have a lifetime, `'frame`. This lifetime ensures that a [`Value`] can only be used as
//! long as the frame that protects it has not been dropped.
//!
//! Most functionality that frames implement is defined by the [`Frame`] trait.
//!
//! [`StaticFrame`]: struct.StaticFrame.html
//! [`DynamicFrame`]: struct.DynamicFrame.html
//! [`NullFrame`]: struct.NullFrame.html
//! [`DynamicAsyncFrame`]: struct.DynamicAsyncFrame.html
//! [`Value`]: ../value/struct.Value.html
//! [`Value::call_async`]: ../value/struct.Value.html#method.call_async
//! [`Frame`]: ../traits/trait.Frame.html
//! [`JuliaTask`]: ../traits/multitask/trait.JuliaTask.html

use crate::error::JlrsResult;
#[cfg(all(feature = "async", target_os = "linux"))]
use crate::mode::Async;
#[cfg(all(feature = "async", target_os = "linux"))]
use crate::traits::mode::private::Mode as _;
use crate::traits::Frame;
use crate::CCall;
use crate::{traits::mode::Mode, traits::private::Internal};
#[cfg(all(feature = "async", target_os = "linux"))]
use futures::future::{FutureExt, LocalBoxFuture};
use jl_sys::jl_value_t;
use std::ffi::c_void;
#[cfg(all(feature = "async", target_os = "linux"))]
use std::future::Future;
use std::marker::PhantomData;
use std::mem;
use std::ptr::null_mut;

pub const PAGE_SIZE: usize = 4096 / mem::size_of::<usize>();
pub const MAX_DYNAMIC_FRAME_SIZE: usize = 64;

#[derive(Copy, Clone, Default)]
pub struct FrameIdx(pub(crate) usize);

/// A `StaticFrame` is a frame that has a definite number of slots available to root values. With
/// some exceptions, creating new `Value`s and calling them requires one slot each. Rather than
/// growing to accomodate for new slots on the GC stack when a slot is needed, a `StaticFrame`
/// reserves its slots on creation.
///
/// You get access to a `StaticFrame` by calling [`Julia::frame`] or [`Frame::frame`], most of
/// their functionality is defined in the [`Frame`] trait.
///
/// [`value`]: ../value/index.html
/// [`Julia::frame`]: ../struct.Julia.html#method.frame
/// [`Frame::frame`]: ../traits/trait.Frame.html#method.frame
/// [`Frame`]: ../traits/trait.Frame.html
pub struct StaticFrame<'frame, M: Mode> {
    pub(crate) raw_frame: &'frame mut [*mut c_void],
    pub(crate) page: Option<Box<[*mut c_void]>>,
    pub(crate) len: usize,
    pub(crate) mode: M,
}

impl<'frame, M: Mode> StaticFrame<'frame, M> {
    pub(crate) unsafe fn new(
        raw_frame: &'frame mut [*mut c_void],
        capacity: usize,
        mode: M,
    ) -> Self {
        mode.push_frame(raw_frame, capacity, Internal);
        StaticFrame {
            raw_frame,
            page: None,
            len: 0,
            mode,
        }
    }

    /// The number of values currently rooted in this frame.
    pub fn size(&self) -> usize {
        self.len
    }

    /// The number of values that can be rooted in this frame.
    pub fn capacity(&self) -> usize {
        self.raw_frame[0] as usize >> 1
    }

    pub(crate) unsafe fn nested_frame<'nested>(
        &'nested mut self,
        capacity: usize,
    ) -> StaticFrame<'nested, M> {
        let raw_frame = if self.capacity() + capacity + 4 > self.raw_frame.len() {
            if self.page.is_none() || self.page.as_ref().unwrap().len() < capacity + 2 {
                let raw = vec![null_mut::<c_void>(); PAGE_SIZE.max(capacity + 2)];
                let page = raw.into_boxed_slice();
                self.page = Some(page);
            }

            self.page.as_mut().unwrap().as_mut()
        } else {
            let cap = self.capacity();
            &mut self.raw_frame[cap + 2..]
        };

        StaticFrame::new(raw_frame, capacity, self.mode.clone())
    }

    pub(crate) unsafe fn nested_dynamic_frame<'nested>(
        &'nested mut self,
    ) -> DynamicFrame<'nested, M> {
        let raw_frame = if self.capacity() + MAX_DYNAMIC_FRAME_SIZE + 2 > self.raw_frame.len() {
            if self.page.is_none() {
                let raw = vec![null_mut::<c_void>(); PAGE_SIZE];
                let page = raw.into_boxed_slice();
                self.page = Some(page);
            }

            self.page.as_mut().unwrap().as_mut()
        } else {
            let cap = self.capacity();
            &mut self.raw_frame[cap + 2..]
        };

        DynamicFrame::new(raw_frame, self.mode.clone())
    }
}

impl<'frame, M: Mode> Drop for StaticFrame<'frame, M> {
    fn drop(&mut self) {
        unsafe { self.mode.clone().pop_frame(self.raw_frame, Internal) }
    }
}

/// A `DynamicFrame` is a frame that has a dynamic number of slots on the GC stack. With some
/// exceptions, creating new `Value`s and calling them require one slot each. A `DynamicFrame`
/// acquires a new slot every time one is needed. See the documentation in the [`value`] module
/// for more information about the costs. You get access to a `DynamicFrame` by calling
/// [`Julia::dynamic_frame`] or [`Frame::dynamic_frame`], most of
/// their functionality is defined in the [`Frame`] trait.
///
/// [`value`]: ../value/index.html
/// [`Julia::dynamic_frame`]: ../struct.Julia.html#method.dynamic_frame
/// [`Frame::dynamic_frame`]: ../traits/trait.Frame.html#method.dynamic_frame
/// [`Frame`]: ../traits/trait.Frame.html
pub struct DynamicFrame<'frame, M: Mode> {
    pub(crate) raw_frame: &'frame mut [*mut c_void],
    pub(crate) page: Option<Box<[*mut c_void]>>,
    pub(crate) mode: M,
}

impl<'frame, M: Mode> DynamicFrame<'frame, M> {
    pub(crate) unsafe fn new(raw_frame: &'frame mut [*mut c_void], mode: M) -> Self {
        mode.push_frame(raw_frame, 0, Internal);
        DynamicFrame {
            raw_frame,
            page: None,
            mode,
        }
    }

    /// The number of values currently rooted in this frame.
    pub fn size(&self) -> usize {
        self.raw_frame[0] as usize >> 1
    }

    pub(crate) unsafe fn nested_frame<'nested>(
        &'nested mut self,
        capacity: usize,
    ) -> StaticFrame<'nested, M> {
        let len = self.raw_frame[0] as usize >> 1;
        let raw_frame = if len + capacity + 4 > self.raw_frame.len() {
            if self.page.is_none() || self.page.as_ref().unwrap().len() < capacity + 2 {
                let raw = vec![null_mut::<c_void>(); PAGE_SIZE.max(capacity + 2)];
                let page = raw.into_boxed_slice();
                self.page = Some(page);
            }

            self.page.as_mut().unwrap().as_mut()
        } else {
            &mut self.raw_frame[len + 2..]
        };

        StaticFrame::new(raw_frame, capacity, self.mode.clone())
    }

    pub(crate) unsafe fn nested_dynamic_frame<'nested>(
        &'nested mut self,
    ) -> DynamicFrame<'nested, M> {
        let len = self.size();
        let raw_frame = if len + MAX_DYNAMIC_FRAME_SIZE + 4 > self.raw_frame.len() {
            if self.page.is_none() {
                let raw = vec![null_mut::<c_void>(); PAGE_SIZE];
                let page = raw.into_boxed_slice();
                self.page = Some(page);
            }

            self.page.as_mut().unwrap().as_mut()
        } else {
            &mut self.raw_frame[len + 2..]
        };

        DynamicFrame::new(raw_frame, self.mode.clone())
    }
}

impl<'frame, M: Mode> Drop for DynamicFrame<'frame, M> {
    fn drop(&mut self) {
        unsafe { self.mode.pop_frame(self.raw_frame, Internal) }
    }
}

/// A `NullFrame` can be used if you call Rust from Julia through `ccall` and want to borrow array
/// data but not perform any allocations. It can't be nested or be used for functions that
/// allocate (like creating new values or calling functions). Functions that depend on allocation
/// will return `JlrsError::NullFrame` if you call them with a `NullFrame`.
pub struct NullFrame<'frame>(PhantomData<&'frame ()>);

impl<'frame> NullFrame<'frame> {
    pub(crate) unsafe fn new(_: &'frame mut CCall) -> Self {
        NullFrame(PhantomData)
    }
}

/// A `DynamicAsyncFrame` is a special kind of `DynamicFrame` that's available when you implement
/// [`JuliaTask`]. In addition to the capabilities of a `DynamicFrame` it can be used to call
/// [`Value::call_async`] which lets you call a function in a new thread in Julia. This feature is
/// only available by using [`AsyncJulia`].
///
/// [`Value::call_async`]: ../value/struct.Value.html#method.call_async
/// [`JuliaTask`]: ../traits/multitask/trait.JuliaTask.html
/// [`AsyncJulia`]: ../multitask/struct.AsyncJulia.html
#[cfg(all(feature = "async", target_os = "linux"))]
pub struct DynamicAsyncFrame<'frame> {
    pub(crate) raw_frame: &'frame mut [*mut c_void],
    pub(crate) page: Option<Box<[*mut c_void]>>,
    pub(crate) mode: Async<'frame>,
}

#[cfg(all(feature = "async", target_os = "linux"))]
impl<'frame> DynamicAsyncFrame<'frame> {
    pub(crate) unsafe fn new(raw_frame: &'frame mut [*mut c_void], mode: Async<'frame>) -> Self {
        mode.push_frame(raw_frame, 0, Internal);
        DynamicAsyncFrame {
            raw_frame,
            page: None,
            mode,
        }
    }

    /// The number of values currently rooted in this frame.
    pub fn size(&self) -> usize {
        self.raw_frame[0] as usize >> 1
    }

    /*
    impl FnOnce(&'a mut DynamicAsyncFrame<'frame>) -> Pin<Box<dyn Future<Output=JlrsResult<f64>> + 'frame>>
    */

    pub unsafe fn async_frame<'nested, T, F, G>(
        &'nested mut self,
        func: G,
    ) -> LocalBoxFuture<'nested, JlrsResult<T>>
    where
        T: 'frame,
        F: Future<Output = JlrsResult<T>> + 'nested,
        G: for<'a> FnOnce(DynamicAsyncFrame<'nested>) -> F + 'nested,
    {
        let nested_async = async move {
            let mut nested = self.nested_async_frame();
            func(nested).await
        };

        nested_async.boxed_local()
    }

    pub(crate) unsafe fn nested_frame<'nested>(
        &'nested mut self,
        capacity: usize,
    ) -> StaticFrame<'nested, Async<'frame>> {
        let len = self.size();
        let raw_frame = if len + capacity + 4 > self.raw_frame.len() {
            if self.page.is_none() || self.page.as_ref().unwrap().len() < capacity + 2 {
                let raw = vec![null_mut::<c_void>(); PAGE_SIZE.max(capacity + 2)];
                let page = raw.into_boxed_slice();
                self.page = Some(page);
            }

            self.page.as_mut().unwrap().as_mut()
        } else {
            &mut self.raw_frame[len + 2..]
        };

        StaticFrame::new(raw_frame, capacity, self.mode.clone())
    }

    pub(crate) unsafe fn nested_dynamic_frame<'nested>(
        &'nested mut self,
    ) -> DynamicFrame<'nested, Async<'frame>> {
        let len = self.raw_frame[0] as usize >> 1;
        let raw_frame = if len + MAX_DYNAMIC_FRAME_SIZE + 4 > self.raw_frame.len() {
            if self.page.is_none() {
                let raw = vec![null_mut::<c_void>(); PAGE_SIZE];
                let page = raw.into_boxed_slice();
                self.page = Some(page);
            }

            self.page.as_mut().unwrap().as_mut()
        } else {
            &mut self.raw_frame[len + 2..]
        };

        DynamicFrame::new(raw_frame, self.mode.clone())
    }

    pub(crate) unsafe fn nested_async_frame<'nested>(
        &'nested mut self,
    ) -> DynamicAsyncFrame<'nested> {
        let len = self.raw_frame[0] as usize >> 1;
        let raw_frame = if len + MAX_DYNAMIC_FRAME_SIZE + 4 > self.raw_frame.len() {
            if self.page.is_none() {
                let raw = vec![null_mut::<c_void>(); PAGE_SIZE];
                let page = raw.into_boxed_slice();
                self.page = Some(page);
            }

            self.page.as_mut().unwrap().as_mut()
        } else {
            &mut self.raw_frame[len + 2..]
        };

        DynamicAsyncFrame::new(raw_frame, self.mode.clone())
    }

    pub fn len(&self) -> usize {
        self.raw_frame[0] as usize >> 1
    }
}

#[cfg(all(feature = "async", target_os = "linux"))]
impl<'frame> Drop for DynamicAsyncFrame<'frame> {
    fn drop(&mut self) {
        unsafe { self.mode.clone().pop_frame(self.raw_frame, Internal) }
    }
}

/// A `Value` that is about to be rooted.
#[repr(transparent)]
pub(crate) struct PendingValue<'frame, 'data>(
    *mut jl_value_t,
    PhantomData<&'frame ()>,
    PhantomData<&'data ()>,
);

impl<'frame, 'data> PendingValue<'frame, 'data> {
    pub(crate) fn inner(self) -> *mut jl_value_t {
        self.0
    }

    pub(crate) fn new(contents: *mut jl_value_t) -> Self {
        PendingValue(contents, PhantomData, PhantomData)
    }
}

/// A `Value` that has not yet been rooted.
#[repr(transparent)]
pub struct UnrootedValue<'frame, 'data, 'borrow>(
    pub(crate) *mut jl_value_t,
    PhantomData<&'frame ()>,
    PhantomData<&'data ()>,
    PhantomData<&'borrow ()>,
);

impl<'frame, 'data, 'borrow> UnrootedValue<'frame, 'data, 'borrow> {
    pub(crate) fn done(self) -> PendingValue<'frame, 'data> {
        PendingValue::new(self.0)
    }

    pub(crate) fn inner(self) -> *mut jl_value_t {
        self.0
    }

    pub(crate) fn new(contents: *mut jl_value_t) -> Self {
        UnrootedValue(contents, PhantomData, PhantomData, PhantomData)
    }
}

pub(crate) type PendingCallResult<'frame, 'data> =
    Result<PendingValue<'frame, 'data>, PendingValue<'frame, 'data>>;

/// A `CallResult` that has not yet been rooted.
pub enum UnrootedCallResult<'frame, 'data, 'inner> {
    Ok(UnrootedValue<'frame, 'data, 'inner>),
    Err(UnrootedValue<'frame, 'data, 'inner>),
}

impl<'frame, 'data, 'inner> UnrootedCallResult<'frame, 'data, 'inner> {
    pub(crate) fn done(self) -> PendingCallResult<'frame, 'data> {
        match self {
            Self::Ok(pov) => Ok(pov.done()),
            Self::Err(pov) => Err(pov.done()),
        }
    }
}

/// An output that can be converted into an [`OutputScope`] to root a value in some earlier frame.
pub struct Output<'scope>(PhantomData<&'scope ()>);

impl<'scope> Output<'scope> {
    pub(crate) fn new() -> Self {
        Output(PhantomData)
    }

    /// Convert the output to an [`OutputScope`].
    pub fn into_scope<'frame, 'borrow, F: Frame<'frame>>(
        self,
        frame: &'borrow mut F,
    ) -> OutputScope<'scope, 'frame, 'borrow, F> {
        OutputScope::new(self, frame)
    }
}

/// A [`Scope`] that can be used once to root a value in an earlier frame.
pub struct OutputScope<'scope, 'frame, 'borrow, F: Frame<'frame>>(
    &'borrow mut F,
    Output<'scope>,
    PhantomData<&'frame ()>,
);

impl<'scope, 'frame, 'borrow, F: Frame<'frame>> OutputScope<'scope, 'frame, 'borrow, F> {
    fn new(output: Output<'scope>, frame: &'borrow mut F) -> Self {
        OutputScope(frame, output, PhantomData)
    }

    /// Nest a `value_frame` and propagate the output to the new frame. See
    /// [`Scope::value_frame`] for more information.
    pub fn value_frame<'data, G>(
        self,
        capacity: usize,
        func: G,
    ) -> JlrsResult<UnrootedValue<'scope, 'data, 'borrow>>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'scope>,
            &'inner mut StaticFrame<'nested, F::Mode>,
        ) -> JlrsResult<UnrootedValue<'scope, 'data, 'inner>>,
    {
        unsafe {
            let mut frame = self.0.nested_frame(capacity, Internal);
            let out = Output::new();
            func(out, &mut frame).map(|pv| UnrootedValue::new(pv.inner()))
        }
    }

    /// Nest a `call_frame` and propagate the output to the new frame. See
    /// [`Scope::value_frame`] for more information.
    pub fn call_frame<'data, G>(
        self,
        capacity: usize,
        func: G,
    ) -> JlrsResult<UnrootedCallResult<'scope, 'data, 'borrow>>
    where
        G: for<'nested, 'inner> FnOnce(
            Output<'scope>,
            &'inner mut StaticFrame<'nested, F::Mode>,
        )
            -> JlrsResult<UnrootedCallResult<'scope, 'data, 'inner>>,
    {
        unsafe {
            let mut frame = self.0.nested_frame(capacity, Internal);
            let out = Output::new();
            func(out, &mut frame).map(|pv| match pv {
                UnrootedCallResult::Ok(pv) => {
                    UnrootedCallResult::Ok(UnrootedValue::new(pv.inner()))
                }
                UnrootedCallResult::Err(pv) => {
                    UnrootedCallResult::Err(UnrootedValue::new(pv.inner()))
                }
            })
        }
    }
}
