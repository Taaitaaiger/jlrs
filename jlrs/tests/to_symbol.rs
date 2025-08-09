mod util;

#[cfg(feature = "local-rt")]
mod tests {
    use jlrs::{data::managed::symbol::Symbol, prelude::*};

    use super::util::JULIA;

    fn use_string_to_symbol() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|mut frame| {
                        assert!(Module::base(&frame).global(&mut frame, "+").is_ok());
                        Ok(())
                    })
                    .unwrap();
            });
        });
    }

    fn use_julia_string_to_symbol() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|mut frame| {
                        let plus = JuliaString::new(&mut frame, "+");
                        assert!(Module::base(&frame).global(&mut frame, plus).is_ok());
                        Ok(())
                    })
                    .unwrap();
            });
        });
    }

    fn use_symbol_to_symbol() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|mut frame| {
                        let plus = Symbol::new(&frame, "+");
                        assert!(Module::base(&frame).global(&mut frame, plus).is_ok());
                        Ok(())
                    })
                    .unwrap();
            });
        });
    }

    #[test]
    fn symbol_tests() {
        use_string_to_symbol();
        use_julia_string_to_symbol();
        use_symbol_to_symbol();
    }
}
