use std::{
    cell::Cell,
    ffi::c_void,
    marker::PhantomPinned,
    mem::MaybeUninit,
    ptr::{self, null_mut, NonNull},
};

use crate::{
    bindings::{jlrs_ppgcstack, jlrs_unsized_scope},
    types::jl_gcframe_t,
};

const NULL_CELL: Cell<*mut c_void> = Cell::new(null_mut());

pub type Roots<const N: usize> = [Cell<*mut c_void>; N];
pub type RootsUnsized = [Cell<*mut c_void>];

#[repr(C)]
pub struct SplitGcFrame<const M: usize, const N: usize> {
    header: jl_gcframe_t,
    roots_head: Roots<M>,
    roots_tail: Roots<N>,
    _pinned: PhantomPinned,
}

impl<const M: usize, const N: usize> SplitGcFrame<M, N> {
    #[inline]
    pub const unsafe fn new() -> Self {
        SplitGcFrame {
            header: jl_gcframe_t::new_split(M, N),
            roots_head: [NULL_CELL; M],
            roots_tail: [NULL_CELL; N],
            _pinned: PhantomPinned,
        }
    }

    #[inline]
    pub unsafe fn set_head_root(&self, slot: usize, root: *mut c_void) {
        debug_assert!(slot < M, "Out of bounds slot");
        self.roots_head.get_unchecked(slot).set(root);
    }

    #[inline]
    pub unsafe fn get_head_root(&self, slot: usize) -> &Cell<*mut c_void> {
        debug_assert!(slot < M, "Out of bounds slot");
        self.roots_head.get_unchecked(slot)
    }

    #[inline]
    pub unsafe fn set_tail_root(&self, slot: usize, root: *mut c_void) {
        debug_assert!(slot < N, "Out of bounds slot");
        self.roots_tail.get_unchecked(slot).set(root);
    }

    #[inline]
    pub unsafe fn get_tail_root(&self, slot: usize) -> &Cell<*mut c_void> {
        debug_assert!(slot < N, "Out of bounds slot");
        self.roots_tail.get_unchecked(slot)
    }

    #[inline]
    pub unsafe fn push_frame(&mut self) {
        let x = jlrs_ppgcstack();
        let mut pgcstack = NonNull::new_unchecked(x).cast::<GcStack>();
        let gcstack_ref = pgcstack.as_mut();
        let top = gcstack_ref.ptr.read();
        self.header.prev.set(top as _);
        gcstack_ref.set_top(&mut self.header)
    }
}

#[repr(C)]
pub struct RawGcFrame<const N: usize> {
    header: jl_gcframe_t,
    roots: Roots<N>,
    _pinned: PhantomPinned,
}

impl<const N: usize> RawGcFrame<N> {
    #[inline]
    pub const unsafe fn new() -> Self {
        RawGcFrame {
            header: jl_gcframe_t::new::<N>(),
            roots: [NULL_CELL; N],
            _pinned: PhantomPinned,
        }
    }

    #[inline]
    pub unsafe fn set_root(&self, slot: usize, root: *mut c_void) {
        debug_assert!(slot < N, "Out of bounds slot");
        self.roots.get_unchecked(slot).set(root);
    }

    #[inline]
    pub unsafe fn get_root(&self, slot: usize) -> &Cell<*mut c_void> {
        self.roots.get_unchecked(slot)
    }

    #[inline]
    pub unsafe fn push_frame(&mut self) {
        let mut pgcstack = NonNull::new_unchecked(jlrs_ppgcstack()).cast::<GcStack>();
        let gcstack_ref = pgcstack.as_mut();
        let top = gcstack_ref.ptr.read();
        self.header.prev.set(top as _);
        gcstack_ref.set_top(&mut self.header)
    }
}

#[inline]
pub unsafe fn pop_frame() {
    let mut pgcstack = NonNull::new_unchecked(jlrs_ppgcstack()).cast::<GcStack>();
    let gcstack_ref = pgcstack.as_mut();
    let top = gcstack_ref.ptr.read();
    let prev = (*top).prev.get();
    gcstack_ref.set_top(prev as _);
}

#[repr(C)]
pub struct SizedGcFrame<'scope, const N: usize> {
    raw: &'scope RawGcFrame<N>,
}

impl<const N: usize> SizedGcFrame<'_, N> {
    #[inline]
    pub unsafe fn root_value(&mut self, slot: usize, value: *mut c_void) {
        debug_assert_ne!(N, slot, "Slot out of bounds");

        unsafe {
            self.roots().get_unchecked(slot).set(value as _);
        }
    }

    #[inline]
    const unsafe fn roots(&self) -> &Roots<N> {
        &self.raw.roots
    }
}

#[repr(C)]
pub struct UnsizedGcFrame<'scope> {
    raw: &'scope [Cell<*mut c_void>],
}

impl<'scope> UnsizedGcFrame<'scope> {
    #[inline]
    pub unsafe fn root_value(&mut self, slot: usize, value: *mut c_void) {
        debug_assert!(self.size() > slot, "Slot out of bounds");

        unsafe {
            self.roots().get_unchecked(slot).set(value as _);
        }
    }

    #[inline]
    const unsafe fn roots(&self) -> &RootsUnsized {
        let sz = self.size();
        let ptr = self.raw.as_ptr().add(2);
        std::slice::from_raw_parts(ptr, sz)
    }

    #[inline]
    pub const fn size(&self) -> usize {
        self.raw.len() - 2
    }

    #[inline]
    pub const unsafe fn get_root(&self, slot: usize) -> &'scope Cell<*mut c_void> {
        debug_assert!(self.size() > slot, "Slot out of bounds");
        &*self.roots().as_ptr().add(slot)
    }
}

#[repr(transparent)]
struct VolatilePtr<T>(*mut T);

impl<T> VolatilePtr<T> {
    #[inline]
    unsafe fn write(&mut self, v: *mut T) {
        ptr::write_volatile(&mut self.0, v);
    }

    unsafe fn read(&mut self) -> *mut T {
        self.0
    }
}

#[repr(transparent)]
struct GcStack {
    ptr: VolatilePtr<jl_gcframe_t>,
    _pinned: PhantomPinned,
}

impl GcStack {
    #[inline]
    unsafe fn set_top(&mut self, frame: *mut jl_gcframe_t) {
        self.ptr.write(frame)
    }
}

// Safety: Julia must have been initialized, must only be called from a thread known to Julia via `jlrs_unsized_scope`.
#[inline]
unsafe extern "C-unwind" fn unsized_scope_trampoline<T, F>(
    frame: *mut jl_gcframe_t,
    callback: *mut c_void,
    result: *mut c_void,
) where
    F: for<'scope> FnMut(UnsizedGcFrame<'scope>) -> T,
{
    let head = &*(frame.cast::<Cell<*mut c_void>>());
    let n = (head.get() as usize) >> 2;
    let raw = std::slice::from_raw_parts(head, n + 2);

    let frame = UnsizedGcFrame { raw };
    let mut callback = NonNull::new_unchecked(callback as *mut F);
    let res = callback.as_mut()(frame);
    NonNull::new_unchecked(result as *mut MaybeUninit<T>)
        .as_mut()
        .write(res);
}

// Safety: Julia must have been initialized, must only be called from a thread known to Julia.
#[inline]
pub unsafe fn unsized_local_scope<T, F>(size: usize, mut func: F) -> T
where
    F: for<'scope> FnMut(UnsizedGcFrame<'scope>) -> T,
{
    let mut result = MaybeUninit::<T>::uninit();

    {
        let trampoline = unsized_scope_trampoline::<T, F>;
        let func = (&mut func) as *mut _ as *mut c_void;
        let result = (&mut result) as *mut _ as *mut c_void;
        jlrs_unsized_scope(size, trampoline, func, result);
    }

    result.assume_init()
}

// Safety: Julia must have been initialized, must be called from a thread known to Julia.
#[inline]
pub unsafe fn sized_local_scope<T, F, const N: usize>(mut func: F) -> T
where
    F: for<'scope> FnMut(SizedGcFrame<'scope, N>) -> T,
{
    let mut frame: RawGcFrame<N> = RawGcFrame::new();
    frame.push_frame();
    let sized_frame = SizedGcFrame { raw: &frame };
    let res = func(sized_frame);
    pop_frame();
    res
}
