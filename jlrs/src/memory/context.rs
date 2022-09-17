use std::{
    cell::{Cell, RefCell},
    ffi::c_void,
    ptr::{null_mut, NonNull}, marker::PhantomData,
};

use jl_sys::{
    jl_gc_alloc_typed, jl_gc_mark_queue_obj, jl_gc_schedule_foreign_sweepfunc, jl_gc_wb,
    jl_new_foreign_type, jl_value_t,
};

use crate::{
    prelude::{DataType, Global, Module, Symbol, Wrapper},
    private::Private,
    wrappers::ptr::private::WrapperPriv,
};

use super::{get_tls, PTls, ledger::Ledger};
pub(crate) const ROOT: Cell<*const Stack> = Cell::new(null_mut());

pub struct TaskContext<'base> {
    ledger: &'base RefCell<Ledger>,
    stack: Stack,
    _base: PhantomData<&'base mut &'base ()>,
}

impl<'base> TaskContext<'base> {
    pub(crate) fn ledger(&self) -> &'base RefCell<Ledger> {
        self.ledger
    }
}

#[repr(C)]
pub(crate) struct AsyncContextFrame<const N: usize> {
    len: Cell<*mut c_void>,
    prev: Cell<*mut c_void>,
    sync: Cell<*const Stack>,
    roots: [Cell<*const Stack>; N],
}

impl<const N: usize> AsyncContextFrame<N> {
    pub fn new() -> Self {
        AsyncContextFrame {
            len: Cell::new(((N + 1) << 2) as *mut c_void),
            prev: Cell::new(null_mut()),
            sync: ROOT,
            roots: [ROOT; N],
        }
    }

    pub(crate) unsafe fn set<'context>(
        &'context self,
        idx: usize,
        context: NonNull<Stack>,
    ) -> &'context Stack {
        self.roots[idx].set(context.cast().as_ptr());
        context.as_ref()
    }

    pub(crate) unsafe fn set_sync<'context>(
        &'context self,
        context: NonNull<Stack>,
    ) -> &'context Stack {
        self.sync.set(context.cast().as_ptr());
        context.as_ref()
    }
}

pub(crate) struct WrappedContext(&'static Stack);

impl WrappedContext {
    pub unsafe fn wrap(ctx: &Stack) -> Self {
        WrappedContext(std::mem::transmute(ctx))
    }

    pub unsafe fn unwrap(self) -> &'static Stack {
        self.0
    }
}

unsafe impl Send for WrappedContext {}
unsafe impl Sync for WrappedContext {}

#[repr(C)]
pub(crate) struct Stack {
    slots: Option<RefCell<Vec<Cell<*mut c_void>>>>,
}

impl Stack {
    pub unsafe fn init() -> DataType<'static> {
        let global = Global::new();
        let sym = Symbol::new(global, "__JlrsStack__");
        let module = Module::main(global);

        if let Ok(dt) = module.global_ref(sym) {
            return DataType::wrap_non_null(NonNull::new_unchecked(dt.ptr().cast()), Private);
        }

        {
            let mut frame = [null_mut(), null_mut(), null_mut()];
            cfg_if::cfg_if! {
                if #[cfg(feature = "lts")] {
                    let rtls = NonNull::new_unchecked(jl_sys::jl_get_ptls_states()).as_mut();
                    frame[0] = (1 << 2) as *mut c_void;
                    frame[1] = rtls.pgcstack.cast();
                    rtls.pgcstack = (&mut frame) as *mut _ as *mut _;
                } else {
                    use jl_sys::{jl_get_current_task, jl_task_t};
                    let task = NonNull::new_unchecked(jl_get_current_task().cast::<jl_task_t>()).as_mut();
                    frame[0] = (1 << 2) as *mut c_void;
                    frame[1] = task.gcstack.cast();
                    task.gcstack = (&mut frame) as *mut _ as *mut _;
                }
            }

            let ptr = jl_new_foreign_type(
                sym.unwrap(Private),
                module.unwrap(Private),
                DataType::any_type(global).unwrap(Private),
                Some(mark),
                Some(sweep),
                1,
                0,
            );
            debug_assert!(!ptr.is_null());
            frame[2] = ptr.cast();

            let dt = DataType::wrap_non_null(NonNull::new_unchecked(ptr), Private);
            module.set_global_unchecked(sym, dt.as_value());

            cfg_if::cfg_if! {
                if #[cfg(feature = "lts")] {
                    let rtls = NonNull::new_unchecked(jl_sys::jl_get_ptls_states()).as_mut();
                    rtls.pgcstack = frame[1].cast();
                } else {
                    let task = NonNull::new_unchecked(jl_get_current_task().cast::<jl_task_t>()).as_mut();
                    task.gcstack = frame[1].cast();
                }
            }

            dt
        }
    }

    pub(crate) unsafe fn push_root(&self, root: NonNull<jl_value_t>) {
        self.slots
            .as_ref()
            .unwrap_unchecked()
            .borrow_mut()
            .push(Cell::new(root.cast().as_ptr()));

        jl_gc_wb(self as *const _ as *mut _, root.as_ptr());
    }

    pub(crate) unsafe fn reserve(&self) -> usize {
        let mut slots = self.slots.as_ref().unwrap_unchecked().borrow_mut();
        let offset = slots.len();
        slots.push(Cell::new(null_mut()));
        offset
    }

    pub(crate) unsafe fn set_root(&self, offset: usize, root: NonNull<jl_value_t>) {
        self.slots.as_ref().unwrap_unchecked().borrow_mut()[offset].set(root.cast().as_ptr());
        jl_gc_wb(self as *const _ as *mut _, root.as_ptr());
    }

    pub(crate) unsafe fn pop_roots(&self, n_roots: usize) {
        let size = self.size();
        let mut slots = self.slots.as_ref().unwrap_unchecked().borrow_mut();
        slots.truncate(size - n_roots);
    }

    pub fn size(&self) -> usize {
        unsafe { self.slots.as_ref().unwrap_unchecked().borrow().len() }
    }

    pub unsafe fn new(ty: DataType) -> NonNull<Stack> {
        let ptls = get_tls();

        let size = std::mem::size_of::<Stack>();

        let ptr = jl_gc_alloc_typed(ptls, size, ty.unwrap(Private).cast()).cast::<Stack>();
        ptr.write(Stack {
            slots: Some(RefCell::new(vec![])),
        });
        jl_gc_schedule_foreign_sweepfunc(ptls, ptr.cast());
        NonNull::new_unchecked(ptr)
    }
}

unsafe extern "C" fn mark(arg1: PTls, obj: *mut jl_value_t) -> usize {
    let ctx = NonNull::new_unchecked(obj).cast::<Stack>().as_ref();
    let mut nptr = 0;

    for cell in ctx.slots.as_ref().unwrap_unchecked().borrow().iter() {
        let ptr = cell.get();
        if !ptr.is_null() {
            nptr |= jl_gc_mark_queue_obj(arg1, ptr.cast());
        }
    }

    nptr as _
}

unsafe extern "C" fn sweep(obj: *mut jl_value_t) {
    NonNull::new_unchecked(obj.cast::<Stack>())
        .as_mut()
        .slots
        .take();
}

/// Protects the `Stack` from being freed by the GC.
///
///
#[repr(C)]
pub struct ContextFrame {
    len: *mut c_void,
    prev: Cell<*mut c_void>,
    root: Cell<*const Stack>,
}

impl ContextFrame {
    pub fn new() -> Self {
        ContextFrame {
            len: (1 << 2) as *mut c_void,
            prev: Cell::new(null_mut()),
            root: ROOT,
        }
    }

    pub(crate) unsafe fn set<'context>(
        &'context self,
        context: NonNull<Stack>,
    ) -> &'context Stack {
        self.root.set(context.cast().as_ptr());
        context.as_ref()
    }

    pub(crate) fn set_prev(&self, prev: *mut c_void) {
        self.prev.set(prev)
    }

    pub(crate) fn prev(&self) -> *mut c_void {
        self.prev.get()
    }

    pub(crate) fn get<'context>(&'context self) -> Option<&'context Stack> {
        let root = self.root.get();
        unsafe { root.as_ref() }
    }
}
