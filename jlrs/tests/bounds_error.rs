use jlrs::prelude::*;
use jlrs::util::JULIA;

#[test]
fn bounds_error() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        let oob_idx = jlrs
            .frame_with_slots(0, |global, frame| {
                frame.frame_with_slots(5, |frame| {
                    let idx = Value::new(&mut *frame, 4usize)?;
                    let data = vec![1.0f64, 2., 3.];
                    let array = Value::move_array(&mut *frame, data, 3)?;
                    let func = Module::base(global).function("getindex")?;
                    let out = func.call2(&mut *frame, array, idx)?.unwrap_err();

                    assert_eq!(out.type_name(), "BoundsError");

                    let field_names = out.field_names();
                    let f0: String = field_names[0].into();
                    assert_eq!(f0, "a");
                    let f1: String = field_names[1].into();
                    assert_eq!(f1, "i");

                    out.get_field(&mut *frame, field_names[1])?
                        .get_nth_field(&mut *frame, 0)?
                        .cast::<isize>()
                })
            })
            .unwrap();

        assert_eq!(oob_idx, 4);
    });
}
