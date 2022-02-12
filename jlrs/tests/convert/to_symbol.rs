#[cfg(feature = "sync-rt")]
mod tests {
    use super::super::super::util::JULIA;
    use jlrs::{prelude::*, wrappers::ptr::symbol::Symbol};

    #[test]
    fn use_string_to_symbol() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|global, frame| {
                assert!(Module::base(global).function(&mut *frame, "+").is_ok());
                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn use_julia_string_to_symbol() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|global, frame| {
                let plus = JuliaString::new(&mut *frame, "+")?.cast::<JuliaString>()?;
                assert!(Module::base(global).function(&mut *frame, plus).is_ok());
                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn use_symbol_to_symbol() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|global, frame| {
                let plus = Symbol::new(global, "+");
                assert!(Module::base(global).function(&mut *frame, plus).is_ok());
                Ok(())
            })
            .unwrap();
        });
    }
}
