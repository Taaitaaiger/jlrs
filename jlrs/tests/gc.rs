mod util;
#[cfg(feature = "sync-rt")]
mod tests {
    use super::util::JULIA;
    use jlrs::{
        memory::gc::{Gc, GcCollection},
        prelude::*,
    };

    #[test]
    fn disable_enable_gc() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            let mut jlrs = jlrs.instance(&mut frame);
            jlrs.enable_gc(false);
            assert!(!jlrs.gc_is_enabled());
            jlrs.enable_gc(true);
            assert!(jlrs.gc_is_enabled());

            jlrs.scope(|frame| {
                frame.enable_gc(false);
                assert!(!frame.gc_is_enabled());
                frame.enable_gc(true);
                assert!(frame.gc_is_enabled());
                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn collect_garbage() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            let mut jlrs = jlrs.instance(&mut frame);

            jlrs.gc_collect(GcCollection::Auto);
            jlrs.gc_collect(GcCollection::Incremental);
            jlrs.gc_collect(GcCollection::Full);

            jlrs.scope(|frame| {
                frame.gc_collect(GcCollection::Auto);
                frame.gc_collect(GcCollection::Incremental);
                frame.gc_collect(GcCollection::Full);

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn insert_safepoint() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            let mut jlrs = jlrs.instance(&mut frame);
            jlrs.gc_safepoint();

            jlrs.scope(|frame| {
                frame.gc_safepoint();
                Ok(())
            })
            .unwrap();
        })
    }
}
