//! Interact with Julia when calling Rust from Julia.
//!
//! This module is only available if the `ccall` feature is enabled.

use std::{
    cell::UnsafeCell,
    ffi::c_void,
    fmt::Debug,
    hint::spin_loop,
    ptr::NonNull,
    sync::{atomic::AtomicBool, Arc, Mutex},
};

use atomic::Ordering;
#[cfg(feature = "uv")]
use jl_sys::uv_async_send;
use jl_sys::{jl_tagged_gensym, jl_throw};
use once_cell::sync::OnceCell;
use threadpool::{Builder, ThreadPool};

use crate::{
    call::Call,
    convert::{ccall_types::CCallReturn, into_julia::IntoJulia},
    data::{
        managed::{
            module::Module,
            private::ManagedPriv,
            rust_result::{RustResult, RustResultRet},
            symbol::Symbol,
            value::{Value, ValueRef},
            Managed,
        },
        types::construct_type::ConstructType,
    },
    error::{JlrsError, JlrsResult},
    init_jlrs,
    memory::{
        context::stack::Stack,
        stack_frame::{PinnedFrame, StackFrame},
        target::{frame::GcFrame, unrooted::Unrooted, Target},
    },
    private::Private,
};

// The pool is lazily created either when it's first used, or when the number of threads is set.
// ThreadPool is !Sync, but it is safe to clone it (which creates a new handle to the pool) and
// use that handle to schedule new jobs to avoid having to lock the pool whenever a new job is
// scheduled.
static POOL: OnceCell<Mutex<ThreadPool>> = OnceCell::new();
static POOL_NAME: OnceCell<String> = OnceCell::new();
thread_local! {
    static LOCAL_POOL: ThreadPool = unsafe {
        init_pool().lock().unwrap().clone()
    }
}

unsafe fn init_pool() -> &'static Mutex<ThreadPool> {
    POOL.get_or_init(|| {
        let name = POOL_NAME.get_or_init(|| {
            let pool_name = "jlrs-pool";
            let sym = jl_tagged_gensym(pool_name.as_ptr().cast(), pool_name.len());
            Symbol::wrap_non_null(NonNull::new_unchecked(sym), Private)
                .as_string()
                .unwrap()
        });

        let pool = Builder::new()
            .num_threads(1)
            .thread_name(name.clone())
            .build();

        Mutex::new(pool)
    })
}

unsafe extern "C" fn set_pool_size(size: usize) {
    init_pool().lock().unwrap().set_num_threads(size);
}

unsafe extern "C" fn set_pool_name(module: Module) {
    POOL_NAME.get_or_init(|| {
        let name = module.name().as_str().unwrap();
        format!("{}-pool", name)
    });
}

/// Interact with Julia from a Rust function called through `ccall`.
///
/// When you call Rust from Julia through `ccall`, Julia has already been initialized and trying to
/// initialize it again causes a crash. In order to still be able to call Julia from Rust
/// you must create a scope first. You can use this struct to do so. It must never be used outside
/// functions called through `ccall`, and only once for each `ccall`ed function.
///
/// Exceptions must only be thrown indirectly by returning a `RustResult`.
pub struct CCall<'context> {
    frame: PinnedFrame<'context, 0>,
}

impl<'context> CCall<'context> {
    /// Create a new `CCall`
    ///
    /// Safety: This function must never be called outside a function called through `ccall` from
    /// Julia and must only be called once during that call.
    pub unsafe fn new(frame: &'context mut StackFrame<0>) -> Self {
        CCall { frame: frame.pin() }
    }

    /// Wake the task associated with `handle`.
    ///
    /// The handle must be the `handle` field of a `Base.AsyncCondition` in Julia. This can be
    /// used to call a long-running Rust function from Julia with `ccall` in another thread and
    /// wait for it to complete in Julia without blocking, an example is available in the
    /// repository: `ccall_with_threads`.
    ///
    /// This method is only available if the `uv` feature is enabled.
    ///
    /// Safety: the handle must be acquired from an `AsyncCondition`.
    #[cfg(feature = "uv")]
    pub unsafe fn uv_async_send(handle: *mut std::ffi::c_void) -> bool {
        uv_async_send(handle.cast()) == 0
    }

    /// Create a [`GcFrame`], call the given closure, and return its result.
    pub fn scope<T, F>(&mut self, func: F) -> JlrsResult<T>
    where
        for<'scope> F: FnOnce(GcFrame<'scope>) -> JlrsResult<T>,
    {
        unsafe {
            let stack = self.frame.stack_frame().sync_stack();
            let (owner, frame) = GcFrame::base(stack);
            let ret = func(frame);
            std::mem::drop(owner);
            ret
        }
    }

    /// Create an instance of `CCall` and use it to invoke the provided closure.
    ///
    /// Safety: this method must only be called from `ccall`ed functions. The returned data is
    /// unrooted and must be returned to Julia immediately.
    pub unsafe fn invoke<T, F>(func: F) -> T
    where
        T: 'static + CCallReturn,
        for<'scope> F: FnOnce(GcFrame<'scope>) -> T,
    {
        let mut frame = StackFrame::new();
        let mut ccall = std::mem::ManuallyDrop::new(CCall::new(&mut frame));

        let stack = ccall.frame.stack_frame().sync_stack();
        let (owner, frame) = GcFrame::base(stack);
        let owner = std::mem::ManuallyDrop::new(owner);
        let ret = func(frame);
        std::mem::drop(owner);
        std::mem::drop(ccall);
        ret
    }

    /// Create an instance of `CCall` and use it to invoke the provided closure.
    ///
    /// Safety: this method must only be called from `ccall`ed functions. The returned data is
    /// unrooted and must be returned to Julia immediately.
    pub unsafe fn invoke_fallible<T, F>(func: F) -> RustResultRet<T>
    where
        T: ConstructType,
        for<'scope> F: FnOnce(GcFrame<'scope>) -> JlrsResult<RustResultRet<T>>,
    {
        let mut frame = StackFrame::new();
        let mut ccall = std::mem::ManuallyDrop::new(CCall::new(&mut frame));

        let stack = ccall.frame.stack_frame().sync_stack();
        let (owner, frame) = GcFrame::base(stack);
        let owner = std::mem::ManuallyDrop::new(owner);

        let ret = match func(frame) {
            Ok(res) => res,
            Err(e) => {
                let mut frame = owner.restore();
                RustResult::<T>::jlrs_error(frame.as_extended_target(), *e)
                    .as_ref()
                    .leak()
            }
        };
        std::mem::drop(owner);
        std::mem::drop(ccall);
        ret
    }

    /// Invoke the provided closure.
    ///
    /// Safety: this method must only be called from `ccall`ed functions. The returned data is
    /// unrooted and must be returned to Julia immediately.
    pub unsafe fn stackless_invoke<T, F>(func: F) -> T
    where
        T: 'static + CCallReturn,
        for<'scope> F: FnOnce(Unrooted<'scope>) -> T,
    {
        func(Unrooted::new())
    }

    /// Create and throw an exception.
    ///
    /// This method calls `func` and throws the result as a Julia exception.
    ///
    /// Safety:
    ///
    /// Julia exceptions are implemented with `setjmp` / `longjmp`. This means that when an
    /// exception is thrown, control flow is returned to a `catch` block by jumping over
    /// intermediate stack frames. It's undefined behaviour to jump over frames that have pending
    /// drops, so you must take care to structure your code such that none of the intermediate
    /// frames have any pending drops.
    #[inline(never)]
    pub unsafe fn throw_exception<F>(mut self, func: F) -> !
    where
        F: for<'scope> FnOnce(&mut GcFrame<'scope>) -> Value<'scope, 'static>,
    {
        let exception = construct_exception(self.frame.stack_frame().sync_stack(), func);
        // catch unwinds the GC stack, so it's okay to forget self.
        std::mem::forget(self);
        jl_throw(exception.ptr().as_ptr())
    }

    /// Create an [`Unrooted`], call the given closure, and return its result.
    ///
    /// Unlike [`CCall::scope`] this method doesn't allocate a stack.
    ///
    /// Safety: must only be called from a `ccall`ed function that doesn't need to root any data.
    pub unsafe fn stackless_scope<T, F>(func: F) -> JlrsResult<T>
    where
        for<'scope> F: FnOnce(Unrooted<'scope>) -> JlrsResult<T>,
    {
        func(Unrooted::new())
    }

    /// Set the size of the internal thread pool.
    pub fn set_pool_size(&self, size: usize) {
        unsafe { set_pool_size(size) }
    }

    pub fn dispatch_to_pool<F, T>(func: F) -> Arc<DispatchHandle<T>>
    where
        F: FnOnce(Arc<DispatchHandle<T>>) + Send + 'static,
        T: IntoJulia + Send + Sync + ConstructType,
    {
        let handle = DispatchHandle::new();
        let cloned = handle.clone();
        LOCAL_POOL.with(|pool| {
            pool.execute(|| func(cloned));
        });

        handle
    }

    /// This function must be called before jlrs can be used. If a runtime or the `julia_module` macro
    /// is used this function is called automatically.
    ///
    /// A module can be provided to allow setting the size of the internal thread pool from Julia
    /// by calling `Jlrs.set_pool_size`.
    #[inline(never)]
    pub fn init_jlrs(&mut self, module: Option<Module>) {
        unsafe {
            init_jlrs(&mut self.frame);

            // Expose thread pool to Julia
            if let Some(module) = module {
                let unrooted = Unrooted::new();

                set_pool_name(module);

                let add_pool = Module::package_root_module(&unrooted, "Jlrs")
                    .unwrap()
                    .global(unrooted, "add_pool")
                    .unwrap()
                    .as_value();

                let fn_ptr = Value::new(unrooted, set_pool_size as *mut c_void).as_value();
                add_pool.call2(unrooted, module.as_value(), fn_ptr).unwrap();
            }
        }
    }
}

#[inline(never)]
unsafe fn construct_exception<'stack, F>(stack: &'stack Stack, func: F) -> ValueRef<'stack, 'static>
where
    for<'scope> F: FnOnce(&mut GcFrame<'scope>) -> Value<'scope, 'static>,
{
    let (owner, mut frame) = GcFrame::base(stack);
    let ret = func(&mut frame);
    let rewrapped = ValueRef::wrap(ret.unwrap_non_null(Private));
    std::mem::drop(owner);
    rewrapped
}

#[doc(hidden)]
#[repr(transparent)]
pub struct AsyncConditionHandle(pub *mut c_void);
unsafe impl Send for AsyncConditionHandle {}
unsafe impl Sync for AsyncConditionHandle {}

#[doc(hidden)]
#[repr(C)]
pub struct AsyncCCall {
    pub join_handle: *mut c_void,
    pub join_func: *mut c_void,
}

unsafe impl ConstructType for AsyncCCall {
    fn construct_type<'target, T>(
        target: crate::memory::target::ExtendedTarget<'target, '_, '_, T>,
    ) -> crate::data::managed::value::ValueData<'target, 'static, T>
    where
        T: Target<'target>,
    {
        let (target, _) = target.split();
        unsafe {
            Module::package_root_module(&target, "Jlrs")
                .unwrap()
                .submodule(&target, "Wrap")
                .unwrap()
                .as_managed()
                .global(target, "AsyncCCall")
                .unwrap()
        }
    }
}

/// Closures that implement this trait can be invoked on a separate thread to avoid blocking
/// Julia.
pub trait AsyncCallback<T: IntoJulia + Send + Sync + ConstructType>:
    'static + Send + Sync + FnOnce() -> JlrsResult<T>
{
}

impl<T, U> AsyncCallback<T> for U
where
    T: IntoJulia + Send + Sync + ConstructType,
    U: 'static + Send + Sync + FnOnce() -> JlrsResult<T>,
{
}

pub struct DispatchHandle<T> {
    result: UnsafeCell<Option<JlrsResult<T>>>,
    flag: AtomicBool,
}

impl<T> Debug for DispatchHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DispatchHandle")
            .field("Completed", &self.flag.load(Ordering::Relaxed))
            .finish()
    }
}

impl<T: IntoJulia> DispatchHandle<T> {
    pub fn new() -> Arc<Self> {
        Arc::new(DispatchHandle {
            result: UnsafeCell::new(None),
            flag: AtomicBool::new(false),
        })
    }

    pub unsafe fn set(self: Arc<Self>, result: JlrsResult<T>) {
        let res_ptr = self.result.get();
        *res_ptr = Some(result);
        self.flag.store(true, Ordering::Release)
    }

    pub unsafe fn join(self: Arc<Self>) -> JlrsResult<T> {
        while !self.flag.load(Ordering::Acquire) {
            spin_loop();
        }

        let mut unwrapped = Arc::try_unwrap(self).unwrap();

        match unwrapped.result.get_mut().take() {
            Some(Ok(res)) => Ok(res),
            Some(Err(e)) => Err(e),
            None => Err(Box::new(JlrsError::exception(
                "Unexpected error: no result",
            ))),
        }
    }
}

unsafe impl<T> Sync for DispatchHandle<T> {}
unsafe impl<T> Send for DispatchHandle<T> {}
