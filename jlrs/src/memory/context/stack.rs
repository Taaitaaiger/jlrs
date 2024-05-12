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
    fmt,
    ptr::{null_mut, NonNull},
};

use jl_sys::{jl_tagged_gensym, jl_value_t, jlrs_gc_wb};
use once_cell::sync::Lazy;

use crate::{
    call::Call,
    data::{
        managed::{module::Module, private::ManagedPriv, symbol::Symbol, value::Value, Managed},
        types::foreign_type::{create_foreign_type_nostack, ForeignType},
    },
    memory::{target::unrooted::Unrooted, PTls},
    prelude::LocalScope,
    private::Private,
};

#[repr(C)]
#[derive(Default)]
pub(crate) struct Stack {
    slots: UnsafeCell<Vec<Cell<*mut c_void>>>,
}

impl fmt::Debug for Stack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let n_slots = unsafe { NonNull::new_unchecked(self.slots.get()).as_ref().capacity() };
        f.debug_struct("Stack")
            .field("addr", &self.slots.get())
            .field("n_slots", &n_slots)
            .finish()
    }
}

// This is incorrect, Stack cannot be used from multiple threads, but ForeignType can only be
// implemented from types that implement Send + Sync. The stack is never shared with other threads
// in jlrs, ForeignType::mark can be called from multiple threads but only needs immutable access,
// and the stack might be dropped/swept from another thread which only happens if no references to
// it exist anymore.
unsafe impl Send for Stack {}
unsafe impl Sync for Stack {}

unsafe impl ForeignType for Stack {
    fn mark(ptls: PTls, data: &Self) -> usize {
        // We can only get here while the GC is running, so there are no active mutable borrows,
        // but this function might be called from multiple threads so an immutable reference must
        // be used.
        let slots = unsafe { &*data.slots.get() };
        let slots_ptr = slots.as_ptr() as *const _;
        let n_slots = slots.len();
        let value_slots = unsafe { std::slice::from_raw_parts(slots_ptr, n_slots) };

        unsafe {
            crate::memory::gc::mark_queue_objarray(ptls, data.as_value_ref(), value_slots);
        }

        0
    }
}

pub(crate) struct StaticSymbol(Symbol<'static>);
impl StaticSymbol {
    #[inline]
    pub(crate) fn as_symbol(&self) -> Symbol {
        self.0
    }
}
unsafe impl Send for StaticSymbol {}
unsafe impl Sync for StaticSymbol {}

// Each "instance" of jlrs needs its own stack type to account for multiple versions of jlrs being
// used by different crates, and that libraries distributed as JLLs might be compiled with
// different versions of Rust.
//
// Safety: This data can only be initialized after Julia has been initialized, and must only be
// accessed from threads that can call into Julia.
pub(crate) static STACK_TYPE_NAME: Lazy<StaticSymbol> = Lazy::new(|| unsafe {
    let stack = "Stack";
    let sym = jl_tagged_gensym(stack.as_ptr().cast(), stack.len());
    StaticSymbol(Symbol::wrap_non_null(NonNull::new_unchecked(sym), Private))
});

impl Stack {
    // Create the foreign type Stack in the JlrsCore module, or return immediately if it already
    // exists.
    #[cfg_attr(
        not(any(
            feature = "local-rt",
            feature = "async-rt",
            feature = "multi-rt",
            feature = "ccall"
        )),
        allow(unused)
    )]
    pub(crate) unsafe fn init() {
        unsafe {
            let unrooted = Unrooted::new();

            unrooted.local_scope::<_, 1>(|mut frame| {
                let module = Module::jlrs_core(&unrooted);

                let sym = STACK_TYPE_NAME.as_symbol();
                if module.global(&unrooted, sym).is_ok() {
                    return;
                }
                let lock_fn = module
                    .global(&unrooted, "lock_init_lock")
                    .unwrap()
                    .as_value();

                let unlock_fn = module
                    .global(&unrooted, "unlock_init_lock")
                    .unwrap()
                    .as_value();

                lock_fn.call0(unrooted).unwrap();

                if module.global(unrooted, sym).is_ok() {
                    unlock_fn.call0(unrooted).unwrap();
                    return;
                }

                // Safety: create_foreign_type is called with the correct arguments, the new type is
                // rooted until the constant has been set, and we've just checked if JlrsCore.Stack
                // already exists.
                let dt = create_foreign_type_nostack::<Self, _>(&mut frame, sym, module);
                module.set_const_unchecked(sym, dt.as_value());

                unlock_fn.call0(unrooted).unwrap();
            });
        };
    }

    // Push a new root to the stack.
    //
    // Safety: `root` must point to data that hasn't been freed yet.
    #[inline]
    pub(crate) unsafe fn push_root(&self, root: NonNull<jl_value_t>) {
        {
            // We can only get here while the GC isn't running, so there are
            // no active borrows.
            let slots = &mut *self.slots.get();
            slots.push(Cell::new(root.cast().as_ptr()));
        }

        jlrs_gc_wb(self as *const _ as *mut _, root.as_ptr().cast());
    }

    // Reserve a slot on the stack.
    //
    // Safety: reserved slot may only be used until the frame it belongs to
    // is popped from the stack.
    #[inline]
    pub(crate) unsafe fn reserve_slot(&self) -> usize {
        // We can only get here while the GC isn't running, so there are
        // no active borrows.
        let slots = &mut *self.slots.get();
        let offset = slots.len();
        slots.push(Cell::new(null_mut()));
        offset
    }

    // Grow the stack capacity by at least `additional` slots
    #[inline]
    pub(crate) fn reserve(&self, additional: usize) {
        unsafe {
            // We can only get here while the GC isn't running, so there are
            // no active borrows.
            let slots = &mut *self.slots.get();
            slots.reserve(additional)
        }
    }

    // Set the root at `offset`
    #[inline]
    pub(crate) unsafe fn set_root(&self, offset: usize, root: NonNull<jl_value_t>) {
        // We can only get here while the GC isn't running, so there are
        // no active borrows.
        let slots = &*self.slots.get();
        slots[offset].set(root.cast().as_ptr());
        jlrs_gc_wb(self as *const _ as *mut _, root.as_ptr().cast());
    }

    // Pop roots from the stack, the new length is `offset`.
    //
    // Safety: must be called when a frame is popped from the stack.
    #[inline]
    pub(crate) unsafe fn pop_roots(&self, offset: usize) {
        // We can only get here while the GC isn't running, so there are
        // no active borrows.
        let slots = &mut *self.slots.get();
        slots.truncate(offset);
    }

    // Returns the size of the stack
    #[inline]
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
    #[inline]
    #[cfg_attr(
        not(any(feature = "local-rt", feature = "async-rt", feature = "ccall")),
        allow(unused)
    )]
    pub(crate) unsafe fn alloc() -> *mut Self {
        let global = Unrooted::new();
        let stack = Value::new(global, Stack::default());
        stack.ptr().cast().as_ptr()
    }
}
