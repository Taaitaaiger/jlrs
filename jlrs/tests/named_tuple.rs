use jlrs::prelude::*;
use jlrs::util::JULIA;

#[test]
fn create_named_tuple() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.frame(4, |_, frame| {
            let name = "foo";
            let value = Value::new(frame, 1u32)?;
            let ntt = Value::new_named_tuple(frame, &mut [name], &mut [value])?;
            assert!(ntt.is::<jlrs::value::datatype::NamedTuple>());
            assert_eq!(ntt.get_field(frame, "foo")?.cast::<u32>()?, 1u32);
            Ok(())
        })
        .unwrap();
    });
}
