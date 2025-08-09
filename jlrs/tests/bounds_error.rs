mod util;
#[cfg(feature = "local-rt")]
mod tests {
    use jlrs::prelude::*;

    use super::util::JULIA;

    #[test]
    fn bounds_error() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                let oob_idx = stack
                    .scope(|mut frame| {
                        frame.scope(|mut frame| unsafe {
                            let idx = Value::new(&mut frame, 4usize);
                            let data = vec![1.0f64, 2., 3.];
                            let array = TypedArray::<f64>::from_vec(&mut frame, data, 3)?
                                .into_jlrs_result()?;
                            let func = Module::base(&frame)
                                .global(&frame, "getindex")?
                                .as_managed();
                            let out = func.call2(&mut frame, array.as_value(), idx).unwrap_err();

                            assert_eq!(out.datatype_name(), "BoundsError");

                            let field_names = out.field_names();
                            let f0: String = field_names[0].as_string().unwrap();
                            assert_eq!(f0, "a");
                            let f1 = field_names[1].as_str().unwrap();
                            assert_eq!(f1, "i");

                            out.get_field(&mut frame, field_names[1])?
                                .get_nth_field(&mut frame, 0)?
                                .unbox::<isize>()
                        })
                    })
                    .unwrap();

                assert_eq!(oob_idx, 4);
            });
        });
    }
}
