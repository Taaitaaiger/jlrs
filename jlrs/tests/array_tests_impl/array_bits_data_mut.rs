#[cfg(feature = "local-rt")]
pub(crate) mod tests {
    use jlrs::{
        data::{
            managed::array::{
                data::accessor::{Accessor, AccessorMut},
                TypedRankedArray,
            },
            types::construct_type::ConstantBool,
        },
        prelude::*,
    };

    use crate::util::JULIA;

    fn bits_data_mut() {
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
                        let mut accessor = arr.bits_data_mut();
                        accessor[[0, 0]] = 2.0;
                        accessor[[0, 1]] = 3.0;
                        assert_eq!(accessor[[0, 0]], 2.0);
                        assert_eq!(accessor[[0, 1]], 3.0);
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn bits_data_mut_rank0() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let data = vec![1.0];
                        let mut arr = TypedRankedArray::<f32, 0>::from_vec(&mut frame, data, [])
                            .unwrap()
                            .unwrap();
                        let mut accessor = arr.bits_data_mut();
                        accessor[[]] = 2.0;
                        assert_eq!(accessor[[]], 2.0);
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn bits_data_mut_set() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let data = vec![1.0, 2.0];
                        let mut arr =
                            TypedRankedArray::<f32, 2>::from_vec(&mut frame, data, (1, 2))
                                .unwrap()
                                .unwrap();
                        let mut accessor = arr.bits_data_mut();
                        assert!(accessor.set([0, 0], 2.0).is_ok());
                        assert!(accessor.set([0, 1], 3.0).is_ok());
                        assert_eq!(accessor.get([0, 0]), Some(&2.0));
                        assert_eq!(accessor.get([0, 1]), Some(&3.0));
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn bits_data_mut_as_mut_slice() {
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
                        let mut accessor = arr.bits_data_mut();
                        let slice = accessor.as_mut_slice();
                        slice[0] = 2.0;
                        slice[1] = 3.0;
                        assert_eq!(accessor.as_slice(), &[2.0, 3.0]);
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn bits_data_mut_into_mut_slice() {
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
                        let accessor = arr.bits_data_mut();
                        let slice = accessor.into_mut_slice();
                        slice[0] = 2.0;
                        slice[1] = 3.0;

                        let accessor = arr.bits_data_mut();
                        assert_eq!(accessor.as_slice(), &[2.0, 3.0]);
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn bits_data_mut_set_unchecked() {
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
                        let mut accessor = arr.bits_data_mut();
                        accessor.set_unchecked([0, 0], 2.0);
                        accessor.set_unchecked([0, 1], 3.0);
                        assert_eq!(accessor.get_unchecked([0, 0]), &2.0);
                        assert_eq!(accessor.get_unchecked([0, 1]), &3.0);
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn bits_data_mut_set_value() {
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
                        let mut accessor = arr.bits_data_mut();

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

    fn bits_data_mut_set_value_unchecked() {
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
                        let mut accessor = arr.bits_data_mut();

                        frame.local_scope::<_, 2>(|mut frame| {
                            let v1 = Value::new(&mut frame, 2.0f32);
                            let v2 = Value::new(&mut frame, 3.0f32);
                            accessor.set_value_unchecked([0, 0], v1);
                            accessor.set_value_unchecked([0, 1], v2);
                        });

                        assert_eq!(
                            accessor
                                .get_value_unchecked(&mut frame, [0, 0])
                                .unbox::<f32>()
                                .unwrap(),
                            2.0
                        );
                        assert_eq!(
                            accessor
                                .get_value_unchecked(&mut frame, [0, 1])
                                .unbox::<f32>()
                                .unwrap(),
                            3.0
                        );
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn bits_data_mut_with_layout() {
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
                        let mut accessor = arr.bits_data_mut_with_layout();
                        accessor[[0, 0]] = ABDE{a: 2};
                        accessor[[0, 1]] = ABDE{a: 3};
                        assert_eq!(accessor[[0, 0]], ABDE{a: 2});
                        assert_eq!(accessor[[0, 1]], ABDE{a: 3});
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn try_bits_data_mut() {
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
                        let mut accessor = arr.try_bits_data_mut::<f32>()?;
                        accessor[[0, 0]] = 2.0;
                        accessor[[0, 1]] = 3.0;
                        assert_eq!(accessor[[0, 0]], 2.0);
                        assert_eq!(accessor[[0, 1]], 3.0);
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn try_bits_data_mut_err() {
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
                        let accessor = arr.try_bits_data_mut::<f64>();
                        assert!(accessor.is_err());
                    }

                    Ok(())
                })
                .unwrap();
        });
    }
    fn bits_data_mut_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let data = vec![1.0, 2.0];
                        let mut arr =
                            TypedArray::<f32>::from_vec_unchecked(&mut frame, data, &[1, 2][..]);
                        let mut accessor = arr.bits_data_mut_unchecked::<f32>();
                        accessor[[0, 0]] = 2.0;
                        accessor[[0, 1]] = 3.0;
                        assert_eq!(accessor[[0, 0]], 2.0);
                        assert_eq!(accessor[[0, 1]], 3.0);
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    pub(crate) fn array_bits_data_mut_tests() {
        bits_data_mut();
        bits_data_mut_rank0();
        bits_data_mut_set();
        bits_data_mut_set_unchecked();
        bits_data_mut_set_value();
        bits_data_mut_set_value_unchecked();
        bits_data_mut_as_mut_slice();
        bits_data_mut_into_mut_slice();
        bits_data_mut_with_layout();
        try_bits_data_mut();
        try_bits_data_mut_err();
        bits_data_mut_unchecked();
    }
}
