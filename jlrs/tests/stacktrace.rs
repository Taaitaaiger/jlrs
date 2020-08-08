use jlrs::prelude::*;
use jlrs::util::JULIA;

#[test]
fn bounds_error() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.frame(0, |global, frame| {
            frame.frame(8, |frame| {
                let idx = Value::new(frame, 4usize)?;
                let data = vec![1.0f64, 2., 3.];
                let array = Value::move_array(frame, data, 3)?;
                let func = Module::base(global)
                    .function("getindex")?
                    .attach_stacktrace(frame)?
                    .unwrap();
                let out = func.call2(frame, array, idx)?.unwrap_err();

                assert_eq!(out.type_name(), "TracedException");

                let field_names = out.field_names(global);
                let f0: String = field_names[0].into();
                assert_eq!(f0, "exc");
                let f1: String = field_names[1].into();
                assert_eq!(f1, "stacktrace");

                let stacktrace = out.get_field(frame, "stacktrace");
                assert!(stacktrace.is_ok());

                let getindex = Module::base(global).function("getindex")?;
                let idx = Value::new(frame, 1usize)?;
                let base = getindex.call2(frame, stacktrace?, idx)?.unwrap();
                assert!(base.get_field(frame, "from_c")?.cast::<bool>().is_ok());

                Ok(())
            })
        })
        .unwrap();
    });
}
