#[cfg(feature = "sync-rt")]
mod tests {
    use super::super::super::util::JULIA;
    use jlrs::memory::gc::{Gc, GcCollection};

    #[test]
    fn disable_enable_gc() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            unsafe { jlrs.enable_gc(false) };
            assert!(!jlrs.gc_is_enabled());
            unsafe { jlrs.enable_gc(true) };
            assert!(jlrs.gc_is_enabled());

            jlrs.scope(|_global, frame| {
                unsafe { frame.enable_gc(false) };
                assert!(!frame.gc_is_enabled());
                unsafe { frame.enable_gc(true) };
                assert!(frame.gc_is_enabled());
                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn collect_garbage() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            unsafe {
                jlrs.gc_collect(GcCollection::Auto);
                jlrs.gc_collect(GcCollection::Incremental);
                jlrs.gc_collect(GcCollection::Full);
            }

            jlrs.scope(|_global, frame| {
                unsafe {
                    frame.gc_collect(GcCollection::Auto);
                    frame.gc_collect(GcCollection::Incremental);
                    frame.gc_collect(GcCollection::Full);
                }
                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn insert_safepoint() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            unsafe {
                jlrs.gc_safepoint();
            }

            jlrs.scope(|_global, frame| {
                unsafe {
                    frame.gc_safepoint();
                }
                Ok(())
            })
            .unwrap();
        })
    }
}
