use jlrs::prelude::*;
use jlrs::util::JULIA;

#[test]
fn move_array_1d() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        jlrs.frame(3, |_, frame| {
            let name = "foo";
            let value = Value::new(frame, 1u32)?;
            let ntt = Value::new_named_tuple(frame, [name], &mut [value])?;
            assert!(ntt.is::<jlrs::value::datatype::NamedTuple>());
            Ok(())
        })
        .unwrap();
    });
}
