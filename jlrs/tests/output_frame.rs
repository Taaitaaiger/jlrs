use jlrs::prelude::*;
use jlrs::util::JULIA;

#[test]
fn nested_value_scope() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let out = jlrs.scope_with_slots(1, |_global, frame| {
            frame
                .value_scope_with_slots(0, |output, frame| {
                    output
                        .into_scope(frame)
                        .value_scope_with_slots(0, |output, frame| {
                            let output = output.into_scope(frame);
                            Value::new(output, 1usize)
                        })
                })?
                .unbox::<usize>()
        });

        assert_eq!(out.unwrap(), 1);
    });
}

#[test]
fn nested_result_scope() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();

        let out = jlrs.scope_with_slots(1, |global, frame| {
            frame
                .result_scope_with_slots(0, |output, frame| {
                    output
                        .into_scope(frame)
                        .result_scope_with_slots(2, |output, frame| unsafe {
                            let func = Module::base(global).function_ref("+")?.wrapper_unchecked();
                            let v1 = Value::new(frame.as_scope(), 1usize)?;
                            let v2 = Value::new(frame.as_scope(), 2usize)?;
                            let output = output.into_scope(frame);
                            func.call2(output, v1, v2)
                        })
                })?
                .unwrap()
                .unbox::<usize>()
        });

        assert_eq!(out.unwrap(), 3);
    });
}
