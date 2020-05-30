use jlrs::prelude::*;
use jlrs::util::JULIA;

#[test]
fn create_symbol() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.frame(0, |global, _frame| {
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
        jlrs.frame(1, |global, frame| {
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
