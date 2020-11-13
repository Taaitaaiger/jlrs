use jlrs::prelude::*;
use jlrs::util::JULIA;

#[test]
fn create_named_tuple() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.frame(4, |_, frame| {
            let name = "foo";
            let value = Value::new(frame, 1u32)?;
            let nt = Value::new_named_tuple(frame, &mut [name], &mut [value])?;
            assert!(nt.is::<jlrs::value::datatype::NamedTuple>());
            assert_eq!(nt.get_field(frame, "foo")?.cast::<u32>()?, 1u32);
            Ok(())
        })
        .unwrap();
    });
}


#[test]
fn create_named_tuple_macro() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.frame(5, |_, frame| {
            let a_name = "a";
            let a_value = Value::new(frame, 1u32)?;
            let b_value = Value::new(frame, 2u64)?;
            let nt = named_tuple!(frame, a_name => a_value, "b" => b_value)?;
            assert!(nt.is::<jlrs::value::datatype::NamedTuple>());
            assert_eq!(nt.get_field(frame, a_name)?.cast::<u32>()?, 1u32);
            assert_eq!(nt.get_field(frame, "b")?.cast::<u64>()?, 2u64);
            Ok(())
        })
        .unwrap();
    });
}
