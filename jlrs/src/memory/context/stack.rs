// jlrs uses a foreign type to handle the rooting of Julia data.
//
// The main reasons this approach is taken:
//  - the stack only needs to store the roots, pushing a frame is a matter of noting the current
//    stack size, poppinga frame can be accomplished by truncating the stack to that size.
//  - the stack can grow, which makes rooting data an infallible operation.
//  - different async tasks can have a separate stack, without the need to update pointers to
//    keep these stacks linked whenever a frame is pushed to or popped from the stack.

use std::{
    borrow::BorrowMut,
    cell::Cell,
    ffi::c_void,
    ptr::{null_mut, NonNull},
    slice,
};

use atomic_refcell::AtomicRefCell;
use jl_sys::{jl_gc_wb, jl_value_t};

use crate::{
    memory::{gc::mark_queue_objarray, stack_frame::PinnedFrame, target::global::Global, PTls},
    prelude::{Module, Symbol, Value, Wrapper},
    wrappers::foreign::{create_foreign_type, ForeignType},
};

#[repr(C)]
#[derive(Default)]
pub(crate) struct Stack {
    pub(crate) slots: AtomicRefCell<Vec<Cell<*mut c_void>>>,
}

unsafe impl ForeignType for Stack {
    fn mark(ptls: PTls, data: &Self) -> usize {
        let slots = data.slots.borrow();
        let slots_ptr = slots.as_ptr() as *mut *mut c_void;
        let n_slots = slots.len();
        let raw_slots = unsafe { slice::from_raw_parts(slots_ptr, n_slots) };
        let self_ptr = data as *const _ as *mut c_void;

        unsafe {
            mark_queue_objarray(ptls, self_ptr, raw_slots);
        }

        0
    }
}

impl Stack {
    // Create the foreign type __JlrsStack__, or return it immediately if it already exists.
    pub(crate) unsafe fn register<const N: usize>(frame: &PinnedFrame<'_, N>) {
        let global = Global::new();
        let sym = Symbol::new(&global, "__JlrsStack__");
        let module = Module::main(&global);

        if module.global(&global, sym).is_ok() {
            return;
        }

        let dt_ref = create_foreign_type::<Self, _>(global, sym, module, None, true, false);
        let ptr = dt_ref.ptr();
        frame.set_sync_root(ptr.cast());

        let dt = dt_ref.wrapper_unchecked();
        module.set_const_unchecked(sym, dt.as_value());
    }

    // Push a new root to the stack
    pub(crate) unsafe fn push_root(&self, root: NonNull<jl_value_t>) {
        self.slots
            .borrow_mut()
            .push(Cell::new(root.cast().as_ptr()));

        jl_gc_wb(self as *const _ as *mut _, root.as_ptr());
    }

    // Reserve a slot on the stack
    pub(crate) unsafe fn reserve_slot(&self) -> usize {
        let mut slots = self.slots.borrow_mut();
        let offset = slots.len();
        slots.push(Cell::new(null_mut()));
        offset
    }

    // Grow the stack capacity by at least `additional` slots
    pub(crate) fn reserve(&self, additional: usize) {
        let mut slots = self.slots.borrow_mut();
        slots.borrow_mut().reserve(additional);
    }

    // Set the root at `offset`
    pub(crate) unsafe fn set_root(&self, offset: usize, root: NonNull<jl_value_t>) {
        self.slots.borrow_mut()[offset].set(root.cast().as_ptr());
        jl_gc_wb(self as *const _ as *mut _, root.as_ptr());
    }

    // Pop roots from the stack, the new length is `offset`.
    pub(crate) unsafe fn pop_roots(&self, offset: usize) {
        let mut slots = self.slots.borrow_mut();
        slots.truncate(offset);
    }

    // Returns the size of the stack
    pub(crate) fn size(&self) -> usize {
        self.slots.borrow().len()
    }

    // Allocate a new Stack through Julia's GC.
    // Safety: root after allocating
    pub(crate) unsafe fn alloc() -> *mut Self {
        let global = Global::new();
        let stack = Value::new(global, Stack::default());
        stack.ptr().cast()
    }
}
