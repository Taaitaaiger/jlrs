use jlrs::prelude::*;
use jlrs::util::JULIA;

#[test]
fn bounds_error() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.scope_with_slots(0, |global, frame| {
            frame.scope_with_slots(8, |frame| {
                let idx = Value::new(&mut *frame, 4usize)?;
                let data = vec![1.0f64, 2., 3.];
                let array = Value::move_array(&mut *frame, data, 3)?;
                let func = Module::base(global)
                    .function("getindex")?
                    .attach_stacktrace(&mut *frame)?
                    .unwrap();
                let out = func.call2(&mut *frame, array, idx)?.unwrap_err();

                assert_eq!(out.type_name().unwrap(), "TracedException");

                let field_names = out.field_names();
                let f0: String = field_names[0].as_string().unwrap();
                assert_eq!(f0, "exc");
                let f1: String = field_names[1].as_string().unwrap();
                assert_eq!(f1, "stacktrace");

                let stacktrace = out.get_field(&mut *frame, "stacktrace");
                assert!(stacktrace.is_ok());

                let getindex = Module::base(global).function("getindex")?;
                let idx = Value::new(&mut *frame, 1usize)?;
                let base = getindex.call2(&mut *frame, stacktrace?, idx)?.unwrap();
                assert!(base
                    .get_field(&mut *frame, "from_c")?
                    .unbox::<bool>()
                    .is_ok());

                Ok(())
            })
        })
        .unwrap();
    });
}
