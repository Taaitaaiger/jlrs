use jlrs::util::JULIA;
use jlrs::{
    memory::traits::gc::{Gc, GcCollection},
    prelude::*,
};

#[test]
fn create_symbol() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.frame_with_slots(0, |global, _frame| {
            let smb = Symbol::new(global, "a");
            smb.extend(global);

            Ok(())
        })
        .unwrap();
    })
}

#[test]
fn function_returns_symbol() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.frame_with_slots(1, |global, frame| {
            let smb = Module::main(global)
                .submodule("JlrsTests")?
                .function("symbol")?;
            let smb_val = smb.call0(frame)?.unwrap();

            assert!(smb_val.is::<Symbol>());
            assert!(smb_val.cast::<Symbol>().is_ok());
            assert!(smb_val.cast::<Module>().is_err());
            assert!(smb_val.cast::<Array>().is_err());
            assert!(smb_val.cast::<DataType>().is_err());

            Ok(())
        })
        .unwrap();
    })
}

#[test]
fn symbols_are_reused() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.frame_with_slots(0, |global, _frame| {
            let s1 = Symbol::new(global, "foo");
            let s2 = Symbol::new(global, "foo");

            unsafe {
                assert_eq!(s1.ptr(), s2.ptr());
            }

            Ok(())
        })
        .unwrap();
    })
}

#[test]
fn symbols_are_not_collected() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.frame_with_slots(0, |global, frame| {
            let s1 = Symbol::new(global, "foo");

            unsafe {
                frame.gc_collect(GcCollection::Full);
                let s1: String = s1.into();
                assert_eq!(s1, String::from("foo"));
            }

            Ok(())
        })
        .unwrap();
    })
}
