mod util;
#[cfg(feature = "local-rt")]
mod tests {
    use jlrs::{
        memory::gc::{Gc, GcCollection},
        prelude::*,
    };

    use super::util::JULIA;

    fn disable_enable_gc() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            let mut jlrs = jlrs.instance(&mut frame);
            jlrs.enable_gc(false);
            assert!(!jlrs.gc_is_enabled());
            jlrs.enable_gc(true);
            assert!(jlrs.gc_is_enabled());

            jlrs.returning::<JlrsResult<_>>()
                .scope(|frame| {
                    frame.enable_gc(false);
                    assert!(!frame.gc_is_enabled());
                    frame.enable_gc(true);
                    assert!(frame.gc_is_enabled());
                    Ok(())
                })
                .unwrap();
        })
    }

    fn collect_garbage() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            let mut jlrs = jlrs.instance(&mut frame);

            jlrs.gc_collect(GcCollection::Auto);
            jlrs.gc_collect(GcCollection::Incremental);
            jlrs.gc_collect(GcCollection::Full);

            jlrs.returning::<JlrsResult<_>>()
                .scope(|frame| {
                    frame.gc_collect(GcCollection::Auto);
                    frame.gc_collect(GcCollection::Incremental);
                    frame.gc_collect(GcCollection::Full);

                    Ok(())
                })
                .unwrap();
        })
    }

    fn insert_safepoint() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            let mut jlrs = jlrs.instance(&mut frame);
            jlrs.gc_safepoint();

            jlrs.returning::<JlrsResult<_>>()
                .scope(|frame| {
                    frame.gc_safepoint();
                    Ok(())
                })
                .unwrap();
        })
    }

    #[test]
    fn gc_tests() {
        disable_enable_gc();
        collect_garbage();
        insert_safepoint();
    }
}
