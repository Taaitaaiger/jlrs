mod util;
#[cfg(feature = "local-rt")]
mod tests {
    use jlrs::{
        memory::gc::{Gc, GcCollection},
        prelude::*,
        runtime::handle::IsActive,
    };

    use super::util::JULIA;

    fn disable_enable_gc() {
        JULIA.with(|handle| {
            let mut handle = handle.borrow_mut();
            handle.gc_interface().enable_gc(false);
            assert!(!handle.gc_interface().gc_is_enabled());
            handle.gc_interface().enable_gc(true);
            assert!(handle.gc_interface().gc_is_enabled());

            handle.with_stack(|mut stack| {
                stack.scope(|frame| {
                    frame.enable_gc(false);
                    assert!(!frame.gc_is_enabled());
                    frame.enable_gc(true);
                    assert!(frame.gc_is_enabled());
                })
            });
        })
    }

    fn collect_garbage() {
        JULIA.with(|handle| {
            let mut handle = handle.borrow_mut();

            handle.gc_interface().gc_collect(GcCollection::Auto);
            handle.gc_interface().gc_collect(GcCollection::Incremental);
            handle.gc_interface().gc_collect(GcCollection::Full);

            handle.with_stack(|mut stack| {
                stack.scope(|frame| {
                    frame.gc_collect(GcCollection::Auto);
                    frame.gc_collect(GcCollection::Incremental);
                    frame.gc_collect(GcCollection::Full);
                })
            });
        });
    }

    fn insert_safepoint() {
        JULIA.with(|handle| {
            let mut handle = handle.borrow_mut();
            handle.gc_interface().gc_safepoint();

            handle.with_stack(|mut stack| {
                stack.scope(|frame| {
                    frame.gc_safepoint();
                })
            })
        })
    }

    #[test]
    fn gc_tests() {
        disable_enable_gc();
        collect_garbage();
        insert_safepoint();
    }
}
