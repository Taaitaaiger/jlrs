mod util;
#[cfg(feature = "local-rt")]
mod tests {
    use std::sync::{
        atomic::{AtomicU8, Ordering},
        Arc,
    };

    use jlrs::{data::managed::parachute::AttachParachute, memory::gc::Gc, prelude::*};

    use super::util::JULIA;

    struct SignalsDrop(Arc<AtomicU8>);
    impl Drop for SignalsDrop {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn create_parachute() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let data = vec![1usize, 2usize];
                    let mut parachute = data.attach_parachute(&mut frame);
                    assert_eq!(parachute.len(), 2);
                    parachute.push(3);
                    assert_eq!(parachute.len(), 3);
                    Ok(())
                })
                .unwrap();
        });
    }

    fn remove_parachute() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame).scope(|mut frame| {
                let arc = frame.scope(|mut frame| {
                    let arc = Arc::new(AtomicU8::new(0));
                    let data = SignalsDrop(arc);
                    let parachute = data.attach_parachute(&mut frame);
                    parachute.remove_parachute()
                });

                assert_eq!(arc.0.fetch_add(1, Ordering::Relaxed), 0);
                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                assert_eq!(arc.0.fetch_add(1, Ordering::Relaxed), 1);
            });
        });
    }

    fn is_dropped() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame).scope(|mut frame| {
                let arc = frame.scope(|mut frame| {
                    let arc = Arc::new(AtomicU8::new(0));
                    let data = SignalsDrop(arc.clone());
                    data.attach_parachute(&mut frame);
                    arc
                });

                assert_eq!(arc.fetch_add(1, Ordering::Relaxed), 0);
                frame.gc_collect(jlrs::memory::gc::GcCollection::Full);
                assert_eq!(arc.fetch_add(1, Ordering::Relaxed), 2);
            });
        });
    }

    #[test]
    fn parachute_tests() {
        create_parachute();
        remove_parachute();
        is_dropped();
    }
}
