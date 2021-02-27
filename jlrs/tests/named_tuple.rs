use jlrs::prelude::*;
use jlrs::util::JULIA;

#[test]
fn create_named_tuple() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.scope_with_slots(4, |_, frame| {
            let name = "foo";
            let value = Value::new(&mut *frame, 1u32)?;
            let nt = Value::new_named_tuple(&mut *frame, &mut [name], &mut [value])?;
            assert!(nt.is::<jlrs::value::datatype::NamedTuple>());
            assert_eq!(nt.get_field(&mut *frame, "foo")?.cast::<u32>()?, 1u32);
            Ok(())
        })
        .unwrap();
    });
}

#[test]
fn create_named_tuple_macro() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.scope_with_slots(5, |_, frame| {
            let a_name = "a";
            let a_value = Value::new(&mut *frame, 1u32)?;
            let b_value = Value::new(&mut *frame, 2u64)?;
            let nt = named_tuple!(&mut *frame, a_name => a_value, "b" => b_value)?;
            assert!(nt.is::<jlrs::value::datatype::NamedTuple>());
            assert_eq!(nt.get_field(&mut *frame, a_name)?.cast::<u32>()?, 1u32);
            assert_eq!(nt.get_field(&mut *frame, "b")?.cast::<u64>()?, 2u64);
            Ok(())
        })
        .unwrap();
    });
}
