//! Task delegated to a background thread that can call into Julia.

use std::{
    fmt,
    marker::{PhantomData, PhantomPinned},
    mem::{self, MaybeUninit},
    os::raw::c_void,
    ptr::NonNull,
    thread::{self, JoinHandle},
};

use jl_sys::jl_gc_alloc_typed;
use parking_lot::Mutex;

use super::{
    module::JlrsCore,
    private::ManagedPriv,
    value::{Value, ValueData, ValueRef, ValueRet},
    Managed, Ref,
};
use crate::{
    call::Call,
    convert::ccall_types::{CCallArg, CCallReturn},
    data::{
        layout::{is_bits::IsBits, typed_layout::HasLayout, valid_layout::ValidLayout},
        types::construct_type::{ConstructType, TypeVarEnv},
    },
    error::JlrsError,
    inline_static_ref,
    memory::{
        gc::gc_safe,
        get_tls,
        target::{unrooted::Unrooted, TargetResult},
    },
    prelude::{DataType, JlrsResult, Target, TargetType},
    private::Private,
    util::uv_async_send_func,
    weak_handle_unchecked,
};

/// A background task.
///
/// Call `Base.fetch` to wait for a background task to complete and fetch the result.
#[repr(transparent)]
pub struct BackgroundTask<'scope, T>(
    NonNull<BackgroundTaskLayout<'scope, T>>,
    PhantomData<&'scope ()>,
)
where
    T: 'static + HasLayout<'static, 'static>,
    T::Layout: IsBits + Clone;

impl<T> Clone for BackgroundTask<'_, T>
where
    T: 'static + HasLayout<'static, 'static>,
    T::Layout: IsBits + Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

impl<T> Copy for BackgroundTask<'_, T>
where
    T: 'static + HasLayout<'static, 'static>,
    T::Layout: IsBits + Clone,
{
}

impl<'scope, T> BackgroundTask<'scope, T>
where
    T: 'static + HasLayout<'static, 'static>,
    T::Layout: IsBits + Clone + CCallReturn,
{
    fn new<'target, Tgt: Target<'target>>(target: Tgt) -> BackgroundTaskData<'target, T, Tgt> {
        unsafe {
            target.with_local_scope::<_, _, 2>(|target, mut frame| {
                let cond =
                    inline_static_ref!(ASYNC_CONDITION, DataType, "Base.AsyncCondition", &frame);
                let cond = cond.as_value().call_unchecked(&mut frame, []);

                let ptls = get_tls();
                let ty = Self::construct_type(&mut frame);
                let ptr = jl_gc_alloc_typed(
                    ptls,
                    mem::size_of::<BackgroundTaskLayout<T>>(),
                    ty.unwrap(Private).cast(),
                ) as *mut MaybeUninit<BackgroundTaskLayout<T>>;

                let layout = (&mut *ptr).write(BackgroundTaskLayout::<T>::new(cond));

                let nn_ptr = NonNull::from(layout);
                BackgroundTask(nn_ptr, PhantomData).root(target)
            })
        }
    }

    fn set(self, value: T::Layout) {
        unsafe {
            let layout = self.unwrap_non_null(Private).as_mut();
            layout.atomic = value;
        }
    }

    unsafe fn notify(self) {
        let func = uv_async_send_func();
        let cond = self.unwrap_non_null(Private).as_ref().cond;
        let handle_ref = cond.ptr().cast::<*mut c_void>().as_ref();
        let handle = *handle_ref;

        func(handle);
    }

    unsafe fn set_join_handle(self, handle: JoinHandle<JlrsResult<()>>) {
        let layout = self
            .unwrap_non_null(Private)
            .cast::<BackgroundTaskLayout<T>>()
            .as_ref();
        let mut guard = layout.thread_handle.lock();
        *guard = Some(handle);
    }
}

impl<T> fmt::Debug for BackgroundTask<'_, T>
where
    T: 'static + HasLayout<'static, 'static>,
    T::Layout: IsBits + Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("BackgroundTask").finish()
    }
}

impl<'scope, 'data, T> ManagedPriv<'scope, 'data> for BackgroundTask<'scope, T>
where
    T: 'static + HasLayout<'static, 'static>,
    T::Layout: IsBits + Clone,
{
    type Wraps = BackgroundTaskLayout<'scope, T>;

    type WithLifetimes<'target, 'da> = BackgroundTask<'target, T>;

    const NAME: &'static str = "BackgroundTask";

    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: crate::private::Private) -> Self {
        BackgroundTask(inner, PhantomData)
    }

    fn unwrap_non_null(self, _: crate::private::Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

unsafe impl<'scope, T> ConstructType for BackgroundTask<'scope, T>
where
    T: 'static + HasLayout<'static, 'static>,
    T::Layout: IsBits + Clone,
{
    type Static = BackgroundTask<'static, T>;

    fn construct_type_uncached<'target, Tgt>(target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, _, 1>(|target, mut frame| unsafe {
            let t = T::construct_type(&mut frame);
            let bgtask_ua = JlrsCore::background_task(&target);
            bgtask_ua.apply_types_unchecked(target, [t])
        })
    }

    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        _: &TypeVarEnv,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, _, 1>(|target, mut frame| unsafe {
            let t = T::construct_type(&mut frame);
            let bgtask_ua = JlrsCore::background_task(&target);
            bgtask_ua.apply_types_unchecked(target, [t])
        })
    }

    fn base_type<'target, Tgt>(target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        Some(JlrsCore::background_task(target).as_value())
    }
}

unsafe impl<'scope, T> CCallArg for BackgroundTask<'scope, T>
where
    T: 'static + HasLayout<'static, 'static>,
    T::Layout: IsBits + Clone,
{
    type CCallArgType = Value<'scope, 'static>;
    type FunctionArgType = Self;
}

unsafe impl<'scope, 'data, T> HasLayout<'scope, 'data> for BackgroundTask<'scope, T>
where
    T: 'static + HasLayout<'static, 'static>,
    T::Layout: IsBits + Clone,
{
    type Layout = BackgroundTaskLayout<'scope, T>;
}

/// A reference to a [`BackgroundTask`] that has not been explicitly rooted.
pub type BackgroundTaskRef<'scope, T> = Ref<'scope, 'static, BackgroundTask<'scope, T>>;

/// A [`BackgroundTaskRef`] with static lifetimes.
///
/// This is a useful shorthand for signatures of `ccall`able functions that return a
/// [`BackgroundTaskRef`].
pub type BackgroundTaskRet<T> = BackgroundTaskRef<'static, T>;

/// [`BackgroundTask`] or [`BackgroundTaskRef`], depending on the target type `Tgt`.
pub type BackgroundTaskData<'target, T, Tgt> =
    <Tgt as TargetType<'target>>::Data<'static, BackgroundTask<'target, T>>;

/// `JuliaResult<BackgroundTask>` or `JuliaResultRef<BackgroundTaskRef>`, depending on the target
/// type `Tgt`.
pub type BackgroundTaskResult<'target, T, Tgt> =
    TargetResult<'target, 'static, BackgroundTask<'target, T>, Tgt>;

/// Layout of [`BackgroundTask`].
#[repr(C)]
pub struct BackgroundTaskLayout<'scope, T>
where
    T: 'static + HasLayout<'static, 'static>,
    T::Layout: IsBits + Clone,
{
    fetch_fn: unsafe extern "C" fn(handle: BackgroundTask<T>) -> ValueRet,
    thread_handle: Box<Mutex<Option<JoinHandle<JlrsResult<()>>>>>,
    cond: ValueRef<'scope, 'static>,
    atomic: T::Layout,
    _pinned: PhantomPinned,
}

unsafe impl<'scope, T> ValidLayout for BackgroundTaskLayout<'scope, T>
where
    T: 'static + HasLayout<'static, 'static>,
    T::Layout: IsBits + Clone,
{
    fn valid_layout(ty: Value) -> bool {
        if ty.is::<DataType>() {
            unsafe {
                let weak_handle = weak_handle_unchecked!();
                let ty = ty.cast_unchecked::<DataType>();
                let constructed = BackgroundTask::<T>::construct_type(&weak_handle).as_managed();
                ty == constructed
            }
        } else {
            false
        }
    }

    fn type_object<'target, Tgt: Target<'target>>(target: &Tgt) -> Value<'target, 'static> {
        JlrsCore::background_task(target).as_value()
    }
}

impl<'scope, T> BackgroundTaskLayout<'scope, T>
where
    T: 'static + HasLayout<'static, 'static>,
    T::Layout: IsBits + Clone + CCallReturn,
{
    fn new(cond: Value<'_, 'static>) -> Self {
        let ptr = cond.unwrap_non_null(Private);
        let cond = ValueRef::wrap(ptr);

        unsafe {
            BackgroundTaskLayout {
                fetch_fn: background_task_fetch,
                thread_handle: Box::new(Mutex::new(None)),
                cond,
                atomic: std::mem::zeroed::<T::Layout>(),
                _pinned: PhantomPinned,
            }
        }
    }

    fn fetch(&self) -> JlrsResult<T::Layout> {
        // This blocks Julia
        if let Some(x) = self.thread_handle.lock().take() {
            match unsafe { gc_safe(|| x.join()) } {
                Ok(Ok(_)) => Ok(self.atomic.clone()),
                Ok(Err(e)) => Err(e)?,
                Err(_e) => Err(JlrsError::exception("background task panicked"))?,
            }
        } else {
            Err(JlrsError::exception("already joined"))?
        }
    }
}

/// Spawn a new background task.
pub fn spawn_background_task<'scope, 'target, T, F, Tgt>(
    target: Tgt,
    func: F,
) -> BackgroundTaskData<'target, T, Tgt>
where
    F: 'static + Send + FnOnce() -> JlrsResult<T::Layout>,
    T: 'static + HasLayout<'static, 'static>,
    T::Layout: IsBits + Clone + CCallReturn,
    Tgt: Target<'target>,
{
    struct Sendable<L>(L);
    impl<L> Sendable<L> {
        fn inner(self) -> L {
            self.0
        }
    }

    unsafe impl<L> Send for Sendable<L> {}

    unsafe {
        target.with_local_scope::<_, _, 1>(|target, mut frame| {
            let task = BackgroundTask::new(&mut frame);
            let task_ref = Sendable(task.as_ref().leak());

            let handle = thread::spawn(move || {
                let task_ref = task_ref.inner();
                let task = task_ref.as_managed();

                match func() {
                    Ok(res) => {
                        task.set(res);
                        task.notify();
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            });

            task.set_join_handle(handle);
            task.root(target)
        })
    }
}

// Should only be called from Julia.
unsafe extern "C" fn background_task_fetch<T>(handle: BackgroundTask<T>) -> ValueRet
where
    T: 'static + HasLayout<'static, 'static>,
    T::Layout: IsBits + Clone + CCallReturn,
{
    let res = handle
        .unwrap_non_null(Private)
        .as_ref()
        .fetch()
        .return_or_throw();

    let unrooted = Unrooted::new();
    Value::try_new_with::<T, _, _>(&unrooted, res)
        .return_or_throw()
        .leak()
}
