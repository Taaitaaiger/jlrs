mod util;
#[cfg(feature = "local-rt")]
mod tests {
    use jlrs::{convert::to_symbol::ToSymbol, data::types::typecheck::NamedTuple, prelude::*};

    use super::util::JULIA;

    fn create_named_tuple() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let name = "foo";
                    let value = Value::new(&mut frame, 1u32);
                    let name = name.to_symbol(&frame);
                    let nt = Value::new_named_tuple(&mut frame, &[(name, value)]);
                    assert!(nt.is::<NamedTuple>());
                    assert_eq!(nt.get_field(&mut frame, "foo")?.unbox::<u32>()?, 1u32);
                    Ok(())
                })
                .unwrap();
        });
    }

    fn create_named_tuple_macro() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let a_name = "a";
                    let a_value = Value::new(&mut frame, 1u32);
                    let b_value = Value::new(&mut frame, 2u64);
                    let nt = named_tuple!(&mut frame, a_name => a_value, "b" => b_value);
                    assert!(nt.is::<NamedTuple>());
                    assert_eq!(nt.get_field(&mut frame, a_name)?.unbox::<u32>()?, 1u32);
                    assert_eq!(nt.get_field(&mut frame, "b")?.unbox::<u64>()?, 2u64);
                    Ok(())
                })
                .unwrap();
        });
    }

    #[test]
    fn named_tuple_tests() {
        create_named_tuple();
        create_named_tuple_macro();
    }
}
