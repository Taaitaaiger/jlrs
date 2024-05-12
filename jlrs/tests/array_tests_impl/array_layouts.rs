#[cfg(feature = "local-rt")]
pub(crate) mod tests {
    use jlrs::{
        data::types::{abstract_type::AnyType, construct_type::UnionTypeConstructor},
        prelude::*,
    };

    use crate::util::JULIA;

    fn bits_layout() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let arr = unsafe { TypedArray::<f32>::new_unchecked(&mut frame, (1, 2)) };

                    assert!(arr.has_bits_layout());
                    assert!(arr.has_inline_layout());
                    assert!(!arr.has_inline_with_refs_layout());
                    assert!(!arr.has_union_layout());
                    assert!(!arr.has_ptr());
                    assert!(!arr.has_value_layout());
                    assert!(!arr.has_managed_layout::<Value>());

                    Ok(())
                })
                .unwrap();
        });
    }

    fn inline_with_refs_layout() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let dt = unsafe {
                        Value::eval_string(&frame, "struct IWRL a::Int8; b end").unwrap();
                        Module::main(&frame)
                            .global(&frame, "IWRL")
                            .unwrap()
                            .as_value()
                    };

                    let arr = unsafe { Array::new_for_unchecked(&mut frame, dt, (1, 2)) };

                    assert!(!arr.has_bits_layout());
                    assert!(arr.has_inline_layout());
                    assert!(arr.has_inline_with_refs_layout());
                    assert!(!arr.has_union_layout());
                    assert!(arr.has_ptr());
                    assert!(!arr.has_value_layout());
                    assert!(!arr.has_managed_layout::<Value>());

                    Ok(())
                })
                .unwrap();
        });
    }

    fn value_layout() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let arr = unsafe { TypedArray::<AnyType>::new_unchecked(&mut frame, (1, 2)) };

                    assert!(!arr.has_bits_layout());
                    assert!(!arr.has_inline_layout());
                    assert!(!arr.has_inline_with_refs_layout());
                    assert!(!arr.has_union_layout());
                    assert!(!arr.has_ptr());
                    assert!(arr.has_value_layout());
                    assert!(arr.has_managed_layout::<Value>());
                    assert!(!arr.has_managed_layout::<Module>());

                    Ok(())
                })
                .unwrap();
        });
    }

    fn managed_layout() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let arr = unsafe { TypedArray::<Module>::new_unchecked(&mut frame, (1, 2)) };

                    assert!(!arr.has_bits_layout());
                    assert!(!arr.has_inline_layout());
                    assert!(!arr.has_inline_with_refs_layout());
                    assert!(!arr.has_union_layout());
                    assert!(!arr.has_ptr());
                    assert!(arr.has_value_layout());
                    assert!(arr.has_managed_layout::<Value>());
                    assert!(arr.has_managed_layout::<Module>());

                    Ok(())
                })
                .unwrap();
        });
    }

    fn bits_union_layout() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let arr = unsafe {
                        TypedArray::<UnionTypeConstructor<f32, f64>>::new_unchecked(
                            &mut frame,
                            (1, 2),
                        )
                    };

                    assert!(!arr.has_bits_layout());
                    assert!(!arr.has_inline_layout());
                    assert!(arr.has_union_layout());
                    assert!(!arr.has_inline_with_refs_layout());
                    // assert!(!arr.has_ptr());
                    assert!(!arr.has_value_layout());
                    assert!(!arr.has_managed_layout::<Value>());

                    Ok(())
                })
                .unwrap();
        });
    }

    fn non_bits_union_layout() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let arr = unsafe {
                        TypedArray::<UnionTypeConstructor<f32, Module>>::new_unchecked(
                            &mut frame,
                            (1, 2),
                        )
                    };

                    assert!(!arr.has_bits_layout());
                    assert!(!arr.has_inline_layout());
                    assert!(!arr.has_inline_with_refs_layout());
                    assert!(!arr.has_union_layout());
                    assert!(!arr.has_ptr());
                    assert!(arr.has_value_layout());
                    assert!(arr.has_managed_layout::<Value>());

                    Ok(())
                })
                .unwrap();
        });
    }

    pub(crate) fn array_layouts_tests() {
        bits_layout();
        inline_with_refs_layout();
        value_layout();
        managed_layout();
        bits_union_layout();
        non_bits_union_layout();
    }
}
