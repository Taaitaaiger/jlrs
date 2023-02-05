// jlrs uses a foreign type to handle the rooting of Julia data.
//
// The main reasons this approach is taken:
//  - the stack only needs to store the roots, pushing a frame is a matter of noting the current
//    stack size, poppinga frame can be accomplished by truncating the stack to that size.
//  - the stack can grow, which makes rooting data an infallible operation.
//  - different async tasks can have a separate stack, without the need to update pointers to
//    keep these stacks linked whenever a frame is pushed to or popped from the stack.

use std::{
    cell::{Cell, UnsafeCell},
    ffi::c_void,
    ptr::{null_mut, NonNull},
};

use jl_sys::{jl_gc_wb, jl_value_t};
use jlrs_macros::julia_version;

use crate::{
    call::Call,
    data::{
        managed::{module::Module, symbol::Symbol, value::Value, Managed},
        types::foreign_type::{ForeignType, OpaqueType},
    },
    memory::{stack_frame::PinnedFrame, target::unrooted::Unrooted, PTls},
};

#[repr(C)]
#[derive(Default)]
pub(crate) struct Stack {
    slots: UnsafeCell<Vec<Cell<*mut c_void>>>,
}

// This is incorrect, Stack cannot be used from multiple threads, but ForeignType can only be
// implemented from types that implement Send + Sync. The stack is never shared with other threads
// in jlrs, ForeignType::mark can be called from multiple threads but only needs immutable access,
// and the stack might be dropped/swept from another thread which only happens if no references to
// it exist anymore.
unsafe impl Send for Stack {}
unsafe impl Sync for Stack {}

unsafe impl ForeignType for Stack {
    #[julia_version(since = "1.10")]
    fn mark(ptls: PTls, data: &Self) -> usize {
        // We can only get here while the GC is running, so there are no active mutable borrows,
        // but this function might be called from multiple threads so an immutable reference must
        // be used.
        let slots = unsafe { &*data.slots.get() };

        let mut n = 0;
        unsafe {
            for slot in slots {
                if !slot.get().is_null() {
                    if crate::memory::gc::mark_queue_obj(ptls, slot.get()) {
                        n += 1;
                    }
                }
            }
        }

        n
    }

    #[julia_version(until = "1.9")]
    fn mark(ptls: PTls, data: &Self) -> usize {
        // We can only get here while the GC is running, so there are no active mutable borrows,
        // but this function might be called from multiple threads so an immutable reference must
        // be used.
        let slots = unsafe { &*data.slots.get() };
        let slots_ptr = slots.as_ptr() as *const *mut c_void;
        let n_slots = slots.len();
        let raw_slots = unsafe { std::slice::from_raw_parts(slots_ptr, n_slots) };
        let self_ptr = data as *const _ as *mut c_void;

        // Called from ForeignType::mark, objs is a slice of pointers to Julia data.
        unsafe {
            crate::memory::gc::mark_queue_objarray(ptls, self_ptr, raw_slots);
        }

        0
    }
}

impl Stack {
    // Create the foreign type Stack in the Jlrs module, or return immediately if it already
    // exists.
    pub(crate) fn init<const N: usize>(frame: &PinnedFrame<N>, module: Module) {
        let global = module.unrooted_target();
        let sym = Symbol::new(&global, "Stack");

        if module.global(&global, sym).is_ok() {
            return;
        }

        unsafe {
            let lock_fn = module.global(&global, "lock_init_lock").unwrap().as_value();

            let unlock_fn = module
                .global(&global, "unlock_init_lock")
                .unwrap()
                .as_value();

            let sym = Symbol::new(&global, "Stack");

            lock_fn.call0(global).unwrap();

            if module.global(global, sym).is_ok() {
                unlock_fn.call0(global).unwrap();
                return;
            }

            // Safety: create_foreign_type is called with the correct arguments, the new type is
            // rooted until the constant has been set, and we've just checked if Jlrs.Stack
            // already exists.
            let dt_ref = Self::create_type(global, sym, module);
            let ptr = dt_ref.ptr();
            frame.set_sync_root(ptr.cast().as_ptr());

            let dt = dt_ref.as_managed();
            module.set_const_unchecked(sym, dt.as_value());

            unlock_fn.call0(global).unwrap();
        };
    }

    // Push a new root to the stack.
    //
    // Safety: `root` must point to data that hasn't been freed yet.
    pub(crate) unsafe fn push_root(&self, root: NonNull<jl_value_t>) {
        {
            // We can only get here while the GC isn't running, so there are
            // no active borrows.
            let slots = &mut *self.slots.get();
            slots.push(Cell::new(root.cast().as_ptr()));
        }

        jl_gc_wb(self as *const _ as *mut _, root.as_ptr());
    }

    // Reserve a slot on the stack.
    //
    // Safety: reserved slot may only be used until the frame it belongs to
    // is popped from the stack.
    pub(crate) unsafe fn reserve_slot(&self) -> usize {
        // We can only get here while the GC isn't running, so there are
        // no active borrows.
        let slots = &mut *self.slots.get();
        let offset = slots.len();
        slots.push(Cell::new(null_mut()));
        offset
    }

    // Grow the stack capacity by at least `additional` slots
    pub(crate) fn reserve(&self, additional: usize) {
        unsafe {
            // We can only get here while the GC isn't running, so there are
            // no active borrows.
            let slots = &mut *self.slots.get();
            slots.reserve(additional)
        }
    }

    // Set the root at `offset`
    pub(crate) unsafe fn set_root(&self, offset: usize, root: NonNull<jl_value_t>) {
        // We can only get here while the GC isn't running, so there are
        // no active borrows.
        let slots = &*self.slots.get();
        slots[offset].set(root.cast().as_ptr());
        jl_gc_wb(self as *const _ as *mut _, root.as_ptr());
    }

    // Pop roots from the stack, the new length is `offset`.
    //
    // Safety: must be called when a frame is popped from the stack.
    pub(crate) unsafe fn pop_roots(&self, offset: usize) {
        // We can only get here while the GC isn't running, so there are
        // no active borrows.
        let slots = &mut *self.slots.get();
        slots.truncate(offset);
    }

    // Returns the size of the stack
    pub(crate) fn size(&self) -> usize {
        unsafe {
            // We can only get here while the GC isn't running, so there are
            // no active borrows.
            let slots = &*self.slots.get();
            slots.len()
        }
    }

    // Create a new stack and move it to Julia.
    // Safety: root after allocating
    pub(crate) unsafe fn alloc() -> *mut Self {
        let global = Unrooted::new();
        let stack = Value::new(global, Stack::default());
        stack.ptr().cast().as_ptr()
    }
}
