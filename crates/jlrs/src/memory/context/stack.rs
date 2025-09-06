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
    ptr::{NonNull, null_mut},
};

use jl_sys::{jl_gc_mark_queue_objarray, jl_sym_t, jl_tagged_gensym, jl_value_t};
use jlrs_sys::jlrs_gc_wb;

use crate::{
    call::Call,
    data::{
        managed::{Managed, module::Module, private::ManagedPriv, symbol::Symbol, value::Value},
        types::foreign_type::{ForeignType, OpaqueType},
    },
    gc_safe::GcSafeOnceLock,
    memory::{PTls, target::unrooted::Unrooted},
    prelude::{LocalScope, Target},
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
    unsafe fn mark<P>(ptls: PTls, data: &Self, parent: &P) -> usize {
        // We can only get here while the GC is running, so there are no active mutable borrows,
        // but this function might be called from multiple threads so an immutable reference must
        // be used.
        let slots = unsafe { &*data.slots.get() };
        let slots_ptr = slots.as_ptr() as *const _;
        let n_slots = slots.len();

        unsafe {
            jl_gc_mark_queue_objarray(
                ptls,
                parent as *const _ as *mut _,
                slots_ptr as *mut _,
                n_slots,
            );
        }

        0
    }
}

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
    pub(crate) unsafe fn init<'target, Tgt: Target<'target>>(tgt: &Tgt) {
        unsafe {
            let module = Module::jlrs_core(tgt);

            #[repr(transparent)]
            struct SendSycSym(*mut jl_sym_t);
            unsafe impl Send for SendSycSym {}
            unsafe impl Sync for SendSycSym {}
            static SYM: GcSafeOnceLock<SendSycSym> = GcSafeOnceLock::new();

            let sym = SYM.get_or_init(|| {
                let stack = "Stack";
                SendSycSym(jl_tagged_gensym(stack.as_ptr().cast(), stack.len()))
            });

            let sym = Symbol::wrap_non_null(NonNull::new_unchecked(sym.0), Private);
            if module.global(tgt, sym).is_ok() {
                return;
            }

            let lock_fn = module.global(tgt, "lock_init_lock").unwrap().as_value();
            let unlock_fn = module.global(tgt, "unlock_init_lock").unwrap().as_value();

            lock_fn.call(tgt, []).unwrap();

            if module.global(tgt, sym).is_ok() {
                unlock_fn.call(tgt, []).unwrap();
                return;
            }

            // Safety: create_foreign_type is called with the correct arguments, the new type is
            // rooted until the constant has been set, and we've just checked if JlrsCore.Stack
            // already exists.
            //
            // FIXME: don't set a constant in another module.
            tgt.local_scope::<_, 2>(|mut frame| {
                let dt = <Self as OpaqueType>::create_type(&mut frame, sym, module);
                module.set_const(&mut frame, sym, dt.as_value()).unwrap();
            });

            unlock_fn.call(tgt, []).unwrap();
        };
    }

    // Push a new root to the stack.
    //
    // Safety: `root` must point to data that hasn't been freed yet.
    #[inline]
    pub(crate) unsafe fn push_root(&self, root: NonNull<jl_value_t>) {
        unsafe {
            {
                // We can only get here while the GC isn't running, so there are
                // no active borrows.
                let slots = &mut *self.slots.get();
                slots.push(Cell::new(root.cast().as_ptr()));
            }

            jlrs_gc_wb(self as *const _ as *mut _, root.as_ptr().cast());
        }
    }

    // Reserve a slot on the stack.
    //
    // Safety: reserved slot may only be used until the frame it belongs to
    // is popped from the stack.
    #[inline]
    pub(crate) unsafe fn reserve_slot(&self) -> usize {
        unsafe {
            // We can only get here while the GC isn't running, so there are
            // no active borrows.
            let slots = &mut *self.slots.get();
            let offset = slots.len();
            slots.push(Cell::new(null_mut()));
            offset
        }
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
        unsafe {
            // We can only get here while the GC isn't running, so there are
            // no active borrows.
            let slots = &*self.slots.get();
            slots[offset].set(root.cast().as_ptr());
            jlrs_gc_wb(self as *const _ as *mut _, root.as_ptr().cast());
        }
    }

    // Pop roots from the stack, the new length is `offset`.
    //
    // Safety: must be called when a frame is popped from the stack.
    #[inline]
    pub(crate) unsafe fn pop_roots(&self, offset: usize) {
        unsafe {
            // We can only get here while the GC isn't running, so there are
            // no active borrows.
            let slots = &mut *self.slots.get();
            slots.truncate(offset);
            slots.shrink_to(offset);
        }
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
        unsafe {
            let global = Unrooted::new();
            let stack = Value::new(global, Stack::default());
            stack.ptr().cast().as_ptr()
        }
    }
}
