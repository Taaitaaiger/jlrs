mod util;

#[cfg(feature = "local-rt")]
mod tests {
    use std::collections::HashSet;

    use jlrs::{
        memory::gc::{Gc, GcCollection},
        prelude::*,
    };

    use super::util::JULIA;

    fn create_symbol() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|frame| {
                    let smb = Symbol::new(&frame, "a");
                    smb.extend(&frame);

                    Ok(())
                })
                .unwrap();
        })
    }

    fn function_returns_symbol() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let smb = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")?
                        .as_managed()
                        .function(&frame, "symbol")?
                        .as_managed();
                    let smb_val = smb.call0(&mut frame).unwrap();

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

    fn symbols_are_reused() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|frame| {
                    let s1 = Symbol::new(&frame, "foo");
                    let s2 = Symbol::new(&frame, "foo");

                    assert_eq!(s1.as_str().unwrap(), s2.as_str().unwrap());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn symbols_are_not_collected() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|frame| {
                    let s1 = Symbol::new(&frame, "foo");

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

    fn jl_string_to_symbol() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let string = JuliaString::new(&mut frame, "+");
                    assert!(Module::base(&frame).function(&frame, string).is_ok());

                    Ok(())
                })
                .unwrap();
        })
    }

    fn bytes_to_symbol() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let sym = Symbol::new_bytes(&mut frame, &[1]).into_jlrs_result();
                    assert!(sym.is_ok());
                    Ok(())
                })
                .unwrap();
        })
    }

    fn bytes_to_symbol_err() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let sym = Symbol::new_bytes(&mut frame, &[1, 0, 1]).into_jlrs_result();
                    assert!(sym.is_err());
                    Ok(())
                })
                .unwrap();
        })
    }

    fn bytes_to_symbol_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|frame| {
                    let sym = unsafe { Symbol::new_bytes_unchecked(&frame, &[129]) };
                    assert_eq!(sym.clone().as_cstr().to_bytes().len(), 1);
                    assert_eq!(sym.as_bytes().len(), 1);
                    assert!(sym.as_str().is_err());
                    Ok(())
                })
                .unwrap();
        })
    }

    fn extend_lifetime() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame).scope(|mut frame| {
                let output = frame.output();

                frame.scope(|frame| {
                    let sym = Symbol::new(&frame, "a");
                    sym.root(output)
                });
            });
        })
    }

    fn symbol_implements_hash() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|frame| {
                    let mut map = HashSet::new();
                    map.insert(Symbol::new(&frame, "foo"));

                    assert!(map.contains(&Symbol::new(&frame, "foo")));

                    Ok(())
                })
                .unwrap();
        })
    }

    #[test]
    fn symbol_tests() {
        create_symbol();
        function_returns_symbol();
        symbols_are_reused();
        symbols_are_not_collected();
        jl_string_to_symbol();
        bytes_to_symbol_unchecked();
        extend_lifetime();
        symbol_implements_hash();
        bytes_to_symbol();
        bytes_to_symbol_err();
    }
}
