use jlrs::prelude::*;
use jlrs::util::JULIA;

#[test]
fn create_symbol() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.frame(4, |global, _frame| {
            let smb = Symbol::new(global, "a");
            smb.extend(global);

            Ok(())
        })
        .unwrap();
    })
}
