use jl_sys::unsized_local_scope;

use super::target::frame::{GcFrame, LocalFrame, LocalGcFrame, UnsizedLocalGcFrame};

pub trait LocalReturning<'ctx> {
    fn returning<T>(&mut self) -> &mut impl LocalScope<'ctx, T>;
}

pub trait Returning<'ctx> {
    fn returning<T>(&mut self) -> &mut impl Scope<'ctx, T>;
}

pub trait LocalScope<'a, T> {
    #[inline]
    fn local_scope<F, const N: usize>(&self, func: F) -> T
    where
        for<'scope> F: FnOnce(LocalGcFrame<'scope, N>) -> T,
    {
        unsafe {
            let mut local_frame = LocalFrame::new();
            let pinned = local_frame.pin();
            let res = func(LocalGcFrame::new(&pinned));
            pinned.pop();
            res
        }
    }

    #[inline]
    fn unsized_local_scope<F>(&self, size: usize, func: F) -> T
    where
        for<'scope> F: FnOnce(UnsizedLocalGcFrame<'scope>) -> T,
    {
        unsafe {
            let mut func = Some(func);
            unsized_local_scope(size, |frame| {
                let frame = UnsizedLocalGcFrame::new(frame);
                func.take().unwrap()(frame)
            })
        }
    }
}

pub trait Scope<'a, T>: LocalScope<'a, T> {
    fn scope<F>(&mut self, func: F) -> T
    where
        for<'scope> F: FnOnce(GcFrame<'scope>) -> T;
}
