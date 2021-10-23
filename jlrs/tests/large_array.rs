use jlrs::prelude::*;
mod util;
use util::JULIA;

#[test]
fn create_large_array() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.scope_with_slots(1, |_, frame| {
            let array = Array::new::<f32, _, _, _>(frame, &[1, 1, 1, 1, 1, 1, 1, 1, 1][..]);
            assert!(array.is_ok());
            Ok(())
        })
        .unwrap();
    });
}

#[test]
fn move_large_array() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.scope_with_slots(1, |_, frame| {
            let array = Array::from_vec(frame, vec![1u64], &[1, 1, 1, 1, 1, 1, 1, 1, 1][..]);
            assert!(array.is_ok());
            Ok(())
        })
        .unwrap();
    });
}

#[test]
fn borrow_large_array() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.scope_with_slots(1, |_, frame| {
            let mut data = vec![1u32];
            let array = Array::from_slice(frame, &mut data, &[1, 1, 1, 1, 1, 1, 1, 1, 1][..]);
            assert!(array.is_ok());
            Ok(())
        })
        .unwrap();
    });
}
