//! Task delegated to a background thread that can call into Julia.
//!
//! This module is available since Julia 1.9.

use std::{
    fmt,
    marker::{PhantomData, PhantomPinned},
    mem::{self, MaybeUninit},
    os::raw::c_void,
    ptr::NonNull,
    sync::Arc,
    thread::{self, JoinHandle},
};

use atomic::Ordering;
use jl_sys::{jl_adopt_thread, jl_gc_alloc_typed, jlrs_gc_safe_enter, jlrs_gc_wb};
use parking_lot::{Condvar, Mutex};

use super::{
    module::JlrsCore,
    private::ManagedPriv,
    value::{Value, ValueData, ValueRef, ValueRet},
    Atomic, Managed, Ref,
};
use crate::{
    call::Call,
    convert::{
        ccall_types::{CCallArg, CCallReturn},
        into_simple_vector::FromSimpleVector,
    },
    data::{
        layout::{typed_layout::HasLayout, valid_layout::ValidLayout},
        types::construct_type::{ConstructType, TypeVarEnv},
    },
    error::{AccessError, JlrsError},
    inline_static_ref,
    memory::{gc::gc_safe, get_tls, target::TargetResult},
    prelude::{DataType, JlrsResult, LocalScope, Target, TargetType},
    private::Private,
    runtime::handle::{delegated_handle::DelegatedHandle, notify, wait},
    util::uv_async_send_func,
    weak_handle_unchecked,
};

/// A delegated task.
///
/// Call `Base.fetch` to wait for a delegated task to complete and fetch the result.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct DelegatedTask<'scope>(
    NonNull<DelegatedTaskLayout<'scope>>,
    PhantomData<&'scope ()>,
);

impl<'scope> DelegatedTask<'scope> {
    fn fetch<'target, Tgt: Target<'target>>(
        &self,
        target: Tgt,
    ) -> JlrsResult<ValueData<'target, 'static, Tgt>> {
        unsafe { self.unwrap_non_null(Private).as_ref().fetch(target) }
    }

    fn new<'target, Tgt: Target<'target>>(target: Tgt) -> DelegatedTaskData<'target, Tgt> {
        unsafe {
            target.with_local_scope::<_, _, 1>(|target, mut frame| {
                let cond =
                    inline_static_ref!(ASYNC_CONDITION, DataType, "Base.AsyncCondition", &frame);
                let cond = cond.as_value().call_unchecked(&mut frame, []);

                let ptls = get_tls();
                let ty = JlrsCore::delegated_task(&target);
                let ptr = jl_gc_alloc_typed(
                    ptls,
                    mem::size_of::<DelegatedTaskLayout>(),
                    ty.unwrap(Private).cast(),
                ) as *mut MaybeUninit<DelegatedTaskLayout>;

                let layout = (&mut *ptr).write(DelegatedTaskLayout::new(cond));

                let nn_ptr = NonNull::from(layout);
                DelegatedTask(nn_ptr, PhantomData).root(target)
            })
        }
    }

    fn set(self, value: Value<'_, 'static>) {
        unsafe {
            let layout = self.unwrap_non_null(Private).as_ref();
            layout.atomic.store(Some(value), Ordering::Release);
            jlrs_gc_wb(self.unwrap(Private).cast(), value.unwrap(Private).cast());
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
            .cast::<DelegatedTaskLayout>()
            .as_ref();
        let mut guard = layout.thread_handle.lock();
        *guard = Some(handle);
    }
}

impl fmt::Debug for DelegatedTask<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("DelegatedTask").finish()
    }
}

impl<'scope, 'data> ManagedPriv<'scope, 'data> for DelegatedTask<'scope> {
    type Wraps = DelegatedTaskLayout<'scope>;

    type WithLifetimes<'target, 'da> = DelegatedTask<'target>;

    const NAME: &'static str = "DelegatedTask";

    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: crate::private::Private) -> Self {
        DelegatedTask(inner, PhantomData)
    }

    fn unwrap_non_null(self, _: crate::private::Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

unsafe impl<'scope> ConstructType for DelegatedTask<'scope> {
    type Static = DelegatedTask<'static>;
    const CACHEABLE: bool = false;

    fn construct_type_uncached<'target, Tgt>(target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        JlrsCore::delegated_task(&target).as_value().root(target)
    }

    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        _: &TypeVarEnv,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        JlrsCore::delegated_task(&target).as_value().root(target)
    }

    fn base_type<'target, Tgt>(target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        Some(JlrsCore::delegated_task(target).as_value())
    }
}

unsafe impl<'scope> CCallArg for DelegatedTask<'scope> {
    type CCallArgType = Value<'scope, 'static>;
    type FunctionArgType = Self;
}

unsafe impl<'scope, 'data> HasLayout<'scope, 'data> for DelegatedTask<'scope> {
    type Layout = DelegatedTaskLayout<'scope>;
}

/// A reference to a [`DelegatedTask`] that has not been explicitly rooted.
pub type DelegatedTaskRef<'scope> = Ref<'scope, 'static, DelegatedTask<'scope>>;

/// A [`DelegatedTaskRef`] with static lifetimes.
///
/// This is a useful shorthand for signatures of `ccall`able functions that return a
/// [`DelegatedTaskRef`].
pub type DelegatedTaskRet = DelegatedTaskRef<'static>;

/// [`DelegatedTask`] or [`DelegatedTaskRef`], depending on the target type `Tgt`.
pub type DelegatedTaskData<'target, Tgt> =
    <Tgt as TargetType<'target>>::Data<'static, DelegatedTask<'target>>;

/// `JuliaResult<DelegatedTask>` or `JuliaResultRef<DelegatedTaskRef>`, depending on the target
/// type `Tgt`.
pub type DelegatedTaskResult<'target, Tgt> =
    TargetResult<'target, 'static, DelegatedTask<'target>, Tgt>;

/// Layout of [`DelegatedTask`].
#[repr(C)]
pub struct DelegatedTaskLayout<'scope> {
    fetch_fn: unsafe extern "C" fn(DelegatedTask) -> ValueRet,
    thread_handle: Box<Mutex<Option<JoinHandle<JlrsResult<()>>>>>,
    cond: ValueRef<'scope, 'static>,
    atomic: Atomic<'scope, 'static, Value<'scope, 'static>>,
    _pinned: PhantomPinned,
}

unsafe impl<'scope> ValidLayout for DelegatedTaskLayout<'scope> {
    fn valid_layout(ty: Value) -> bool {
        let target = unsafe { weak_handle_unchecked!() };
        ty == JlrsCore::delegated_task(&target).as_value()
    }

    fn type_object<'target, Tgt: Target<'target>>(target: &Tgt) -> Value<'target, 'static> {
        JlrsCore::delegated_task(target).as_value()
    }
}

impl<'scope> DelegatedTaskLayout<'scope> {
    fn new(cond: Value<'_, 'static>) -> Self {
        let ptr = cond.unwrap_non_null(Private);
        let cond = ValueRef::wrap(ptr);

        DelegatedTaskLayout {
            fetch_fn: delegated_task_fetch,
            thread_handle: Box::new(Mutex::new(None)),
            cond,
            atomic: Atomic::new(),
            _pinned: PhantomPinned,
        }
    }

    fn fetch<'target, Tgt: Target<'target>>(
        &self,
        target: Tgt,
    ) -> JlrsResult<ValueData<'target, 'static, Tgt>> {
        if let Some(x) = self.thread_handle.lock().take() {
            match unsafe { gc_safe(|| x.join()) } {
                Ok(Ok(_)) => unsafe {
                    if let Some(v) = self.atomic.load(&target, Ordering::Relaxed) {
                        Ok(v.root(target))
                    } else {
                        Err(AccessError::UndefRef)?
                    }
                },
                Ok(Err(e)) => Err(e)?,
                Err(_e) => Err(JlrsError::exception("delegated task panicked"))?,
            }
        } else {
            Err(JlrsError::exception("already joined"))?
        }
    }
}

/// Spawn a new delegated task.
pub fn spawn_delegated_task<'scope, 'target, D, F, Tgt>(
    target: Tgt,
    func: F,
    data: D,
) -> DelegatedTaskData<'target, Tgt>
where
    for<'delegate> F:
        'static + Send + FnOnce(DelegatedHandle, D::InScope<'delegate>) -> JlrsResult<ValueRet>,
    D: FromSimpleVector<'scope>,
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
        target.with_local_scope::<_, _, 2>(|target, mut frame| {
            let delegated_data = data.into_simple_vector(&mut frame);
            let delegated_data = Sendable(delegated_data.as_ref().leak());
            let active = Arc::new((Mutex::new(false), Condvar::new()));
            let active_clone = active.clone();
            let task = DelegatedTask::new(&mut frame);

            let task_ref = Sendable(task.as_ref().leak());
            let handle = thread::spawn(move || {
                let _pgcstack = jl_adopt_thread();

                let delegated_data = delegated_data.inner();
                let task_ref = task_ref.inner();

                let res = crate::weak_handle_unchecked!().local_scope::<_, 2>(|mut frame| {
                    let task = task_ref.root(&mut frame);
                    let delegated_data = delegated_data.root(&mut frame);
                    notify(&active_clone);
                    let handle = DelegatedHandle::new();
                    let delegated_data =
                        <D::InScope<'_> as FromSimpleVector>::from_simple_vector(delegated_data);
                    match func(handle, delegated_data) {
                        Ok(res) => {
                            task.set(res.as_value());
                            task.notify();
                            Ok(())
                        }
                        Err(e) => Err(e),
                    }
                });

                let ptls = get_tls();
                jlrs_gc_safe_enter(ptls);

                res
            });

            task.set_join_handle(handle);
            gc_safe(|| wait(&active));

            task.root(target)
        })
    }
}

// Should only be called from Julia.
unsafe extern "C" fn delegated_task_fetch(handle: DelegatedTask) -> ValueRet {
    let weak_handle = weak_handle_unchecked!();
    handle
        .fetch(&weak_handle)
        .map(|v| v.leak())
        .return_or_throw()
}
