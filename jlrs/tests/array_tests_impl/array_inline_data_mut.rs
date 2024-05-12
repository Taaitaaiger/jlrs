#[cfg(feature = "local-rt")]
pub(crate) mod tests {
    use jlrs::{
        data::{
            managed::array::data::accessor::{Accessor, AccessorMut},
            types::construct_type::ConstantBool,
        },
        prelude::*,
    };

    use crate::util::JULIA;

    fn inline_data_mut() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let data = vec![1.0, 2.0];
                        let mut arr =
                            TypedArray::<f32>::from_vec_unchecked(&mut frame, data, (1, 2));
                        let accessor = arr.inline_data_mut();
                        assert_eq!(accessor[[0, 0]], 1.0);
                        assert_eq!(accessor[[0, 1]], 2.0);
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn inline_data_mut_set_value() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let data = vec![1.0, 2.0];
                        let mut arr =
                            TypedArray::<f32>::from_vec_unchecked(&mut frame, data, (1, 2));
                        let mut accessor = arr.inline_data_mut();

                        frame.local_scope::<_, 3>(|mut frame| {
                            let v1 = Value::new(&mut frame, 2.0f32);
                            let v2 = Value::new(&mut frame, 3.0f32);
                            let v3 = Value::new(&mut frame, 3.0f64);
                            accessor.set_value(&frame, [0, 0], v1).unwrap().unwrap();
                            accessor.set_value(&frame, [0, 1], v2).unwrap().unwrap();
                            assert!(accessor.set_value(&frame, [0, 2], v2).is_err());
                            assert!(accessor.set_value(&frame, [0, 1], v3).unwrap().is_err());
                        });

                        assert_eq!(
                            accessor
                                .get_value(&mut frame, [0, 0])
                                .unwrap()
                                .unwrap()
                                .unbox::<f32>()
                                .unwrap(),
                            2.0
                        );
                        assert_eq!(
                            accessor
                                .get_value(&mut frame, [0, 1])
                                .unwrap()
                                .unwrap()
                                .unbox::<f32>()
                                .unwrap(),
                            3.0
                        );
                        assert!(accessor.get_value(&mut frame, [1, 1]).is_none());
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn inline_data_mut_set_value_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let data = vec![1.0, 2.0];
                        let mut arr =
                            TypedArray::<f32>::from_vec_unchecked(&mut frame, data, (1, 2));
                        let mut accessor = arr.inline_data_mut();

                        frame.local_scope::<_, 2>(|mut frame| {
                            let v1 = Value::new(&mut frame, 1.0f32);
                            let v2 = Value::new(&mut frame, 2.0f32);
                            accessor.set_value_unchecked([0, 0], v1);
                            accessor.set_value_unchecked([0, 1], v2);
                        });

                        assert_eq!(
                            accessor
                                .get_value_unchecked(&mut frame, [0, 0])
                                .unbox::<f32>()
                                .unwrap(),
                            1.0
                        );
                        assert_eq!(
                            accessor
                                .get_value_unchecked(&mut frame, [0, 1])
                                .unbox::<f32>()
                                .unwrap(),
                            2.0
                        );
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn inline_data_mut_with_layout() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>().scope(|mut frame| {
                    unsafe {
                        #[derive(ValidField, ValidLayout, IsBits, Debug, Clone, Typecheck, Unbox, PartialEq)]
                        #[jlrs(julia_type = "Main.ABDE")]
                        #[repr(C)]
                        struct ABDE {
                            pub(crate) a: i8
                        }

                        #[derive(ConstructType, HasLayout)]
                        #[jlrs(julia_type = "Main.ABDE", constructor_for = "ABDE", scope_lifetime = false, data_lifetime = false, layout_params = [], elided_params = ["T"], all_params = ["T"])]
                        struct ABDETypeConstructor<T> {
                            _t: ::std::marker::PhantomData<T>,
                        }

                        Value::eval_string(&frame, "struct ABDE{T} a::Int8 end").unwrap();
                        let data = vec![ABDE{a: 1}, ABDE{a: 2}];

                        let mut arr = TypedArray::<ABDETypeConstructor<ConstantBool<true>>>::from_vec_unchecked(&mut frame, data, (1, 2));
                        let accessor = arr.inline_data_mut_with_layout();
                        assert_eq!(accessor[[0, 0]], ABDE{a: 1});
                        assert_eq!(accessor[[0, 1]], ABDE{a: 2});
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn try_inline_data_mut() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let data = vec![1.0, 2.0];
                        let mut arr =
                            TypedArray::<f32>::from_vec_unchecked(&mut frame, data, (1, 2));
                        let accessor = arr.try_inline_data_mut::<f32>()?;
                        assert_eq!(accessor[[0, 0]], 1.0);
                        assert_eq!(accessor[[0, 1]], 2.0);
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn try_inline_data_mut_err() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let data = vec![1.0, 2.0];
                        let mut arr =
                            TypedArray::<f32>::from_vec_unchecked(&mut frame, data, (1, 2));
                        let accessor = arr.try_inline_data_mut::<f64>();
                        assert!(accessor.is_err());
                    }

                    Ok(())
                })
                .unwrap();
        });
    }
    fn inline_data_mut_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let data = vec![1.0, 2.0];
                        let mut arr =
                            TypedArray::<f32>::from_vec_unchecked(&mut frame, data, (1, 2));
                        let accessor = arr.inline_data_mut_unchecked::<f32>();
                        assert_eq!(accessor[[0, 0]], 1.0);
                        assert_eq!(accessor[[0, 1]], 2.0);
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    pub(crate) fn array_inline_data_mut_tests() {
        inline_data_mut();
        inline_data_mut_set_value();
        inline_data_mut_set_value_unchecked();
        inline_data_mut_with_layout();
        try_inline_data_mut();
        try_inline_data_mut_err();
        inline_data_mut_unchecked();
    }
}
