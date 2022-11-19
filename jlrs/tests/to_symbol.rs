mod util;

#[cfg(feature = "sync-rt")]
mod tests {
    use jlrs::{prelude::*, wrappers::ptr::symbol::Symbol};

    use super::util::JULIA;

    fn use_string_to_symbol() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    assert!(Module::base(&frame).function(&mut frame, "+").is_ok());
                    Ok(())
                })
                .unwrap();
        });
    }

    fn use_julia_string_to_symbol() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let plus = JuliaString::new(&mut frame, "+");
                    assert!(Module::base(&frame).function(&mut frame, plus).is_ok());
                    Ok(())
                })
                .unwrap();
        });
    }

    fn use_symbol_to_symbol() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let plus = Symbol::new(&frame, "+");
                    assert!(Module::base(&frame).function(&mut frame, plus).is_ok());
                    Ok(())
                })
                .unwrap();
        });
    }

    #[test]
    fn symbol_tests() {
        use_string_to_symbol();
        use_julia_string_to_symbol();
        use_symbol_to_symbol();
    }
}
