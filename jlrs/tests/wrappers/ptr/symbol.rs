#[cfg(feature = "sync-rt")]
mod tests {
    use std::collections::HashSet;

    use crate::util::JULIA;
    use jlrs::memory::gc::{Gc, GcCollection};
    use jlrs::prelude::*;

    #[test]
    fn create_symbol() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope_with_capacity(0, |global, _frame| {
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
            jlrs.scope_with_capacity(1, |global, mut frame| unsafe {
                let smb = Module::main(global)
                    .submodule_ref("JlrsTests")?
                    .wrapper_unchecked()
                    .function_ref("symbol")?
                    .wrapper_unchecked();
                let smb_val = smb.call0(&mut frame)?.unwrap();

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
            jlrs.scope_with_capacity(0, |global, _frame| {
                let s1 = Symbol::new(global, "foo");
                let s2 = Symbol::new(global, "foo");

                assert_eq!(s1.as_str().unwrap(), s2.as_str().unwrap());

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn symbols_are_not_collected() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope_with_capacity(0, |global, mut frame| {
                let s1 = Symbol::new(global, "foo");

                {
                    frame.gc_collect(GcCollection::Full);
                    let s1: String = s1.as_string().unwrap();
                    assert_eq!(s1, String::from("foo"));
                }

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn jl_string_to_symbol() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope_with_capacity(1, |global, mut frame| {
                let string = JuliaString::new(&mut frame, "+")?;
                assert!(Module::base(global).function_ref(string).is_ok());

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    fn bytes_to_symbol() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope_with_capacity(1, |_, mut frame| {
                let sym = Symbol::new_bytes(&mut frame, &[1])?.into_jlrs_result();
                assert!(sym.is_ok());
                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    #[cfg(not(all(target_os = "windows", feature = "lts")))]
    fn bytes_to_symbol_err() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope_with_capacity(1, |_, mut frame| {
                let sym = Symbol::new_bytes(&mut frame, &[1, 0, 1])?.into_jlrs_result();
                assert!(sym.is_err());
                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn bytes_to_symbol_unchecked() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope_with_capacity(1, |global, _| {
                let sym = unsafe { Symbol::new_bytes_unchecked(global, &[129]) };
                assert_eq!(sym.clone().as_cstr().to_bytes().len(), 1);
                assert_eq!(sym.as_bytes().len(), 1);
                assert!(sym.as_str().is_err());
                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn leak_symbol() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            let leaked = jlrs
                .scope_with_capacity(0, |global, _frame| {
                    let sym = Symbol::new(global, "a");
                    Ok(sym.as_leaked())
                })
                .unwrap();

            jlrs.scope(|global, _| {
                let unleak = unsafe { leaked.as_value(global) };
                assert!(unleak.is::<Symbol>());
                Ok(())
            })
            .unwrap()
        })
    }

    #[test]
    fn extend_lifetime() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope_with_capacity(0, |_, mut frame| {
                let output = frame.output()?;

                frame
                    .scope(|mut frame| {
                        let sym = Symbol::new(frame.as_scope().global(), "a");
                        Ok(sym.root(output))
                    })
                    .unwrap();

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn symbol_implements_hash() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope_with_capacity(0, |global, _| {
                let mut map = HashSet::new();
                map.insert(Symbol::new(global, "foo"));

                assert!(map.contains(&Symbol::new(global, "foo")));

                Ok(())
            })
            .unwrap();
        })
    }
}
