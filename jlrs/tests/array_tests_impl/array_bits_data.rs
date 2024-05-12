#[cfg(feature = "local-rt")]
pub(crate) mod tests {
    use jlrs::{
        data::{managed::array::data::accessor::Accessor, types::construct_type::ConstantBool},
        prelude::*,
    };

    use crate::util::JULIA;

    fn bits_data() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let data = vec![1.0, 2.0];
                        let arr = TypedArray::<f32>::from_vec_unchecked(&mut frame, data, (1, 2));
                        let accessor = arr.bits_data();
                        assert_eq!(accessor[[0, 0]], 1.0);
                        assert_eq!(accessor[[0, 1]], 2.0);
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn bits_data_get() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let data = vec![1.0, 2.0];
                        let arr = TypedArray::<f32>::from_vec_unchecked(&mut frame, data, (1, 2));
                        let accessor = arr.bits_data();
                        assert_eq!(accessor.get([0, 0]), Some(&1.0));
                        assert_eq!(accessor.get([0, 1]), Some(&2.0));
                        assert_eq!(accessor.get([1, 1]), None);
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn bits_data_as_slice() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let data = vec![1.0, 2.0];
                        let arr = TypedArray::<f32>::from_vec_unchecked(&mut frame, data, (1, 2));
                        let accessor = arr.bits_data();
                        assert_eq!(accessor.as_slice(), &[1.0, 2.0]);
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn bits_data_into_slice() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let data = vec![1.0, 2.0];
                        let arr = TypedArray::<f32>::from_vec_unchecked(&mut frame, data, (1, 2));
                        let accessor = arr.bits_data();
                        assert_eq!(accessor.into_slice(), &[1.0, 2.0]);
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn bits_data_get_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let data = vec![1.0, 2.0];
                        let arr = TypedArray::<f32>::from_vec_unchecked(&mut frame, data, (1, 2));
                        let accessor = arr.bits_data();
                        assert_eq!(accessor.get_unchecked([0, 0]), &1.0);
                        assert_eq!(accessor.get_unchecked([0, 1]), &2.0);
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn bits_data_get_value() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let data = vec![1.0, 2.0];
                        let arr = TypedArray::<f32>::from_vec_unchecked(&mut frame, data, (1, 2));
                        let accessor = arr.bits_data();
                        assert_eq!(
                            accessor
                                .get_value(&mut frame, [0, 0])
                                .unwrap()
                                .unwrap()
                                .unbox::<f32>()
                                .unwrap(),
                            1.0
                        );
                        assert_eq!(
                            accessor
                                .get_value(&mut frame, [0, 1])
                                .unwrap()
                                .unwrap()
                                .unbox::<f32>()
                                .unwrap(),
                            2.0
                        );
                        assert!(accessor.get_value(&mut frame, [1, 1]).is_none());
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn bits_data_get_value_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let data = vec![1.0, 2.0];
                        let arr = TypedArray::<f32>::from_vec_unchecked(&mut frame, data, (1, 2));
                        let accessor = arr.bits_data();
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

    fn bits_data_with_layout() {
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
                        let arr = TypedArray::<ABDETypeConstructor<ConstantBool<true>>>::from_vec_unchecked(&mut frame, data, (1, 2));
                        let accessor = arr.bits_data_with_layout();
                        assert_eq!(accessor[[0, 0]], ABDE{a: 1});
                        assert_eq!(accessor[[0, 1]], ABDE{a: 2});
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn try_bits_data() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let data = vec![1.0, 2.0];
                        let arr = TypedArray::<f32>::from_vec_unchecked(&mut frame, data, (1, 2));
                        let accessor = arr.try_bits_data::<f32>()?;
                        assert_eq!(accessor[[0, 0]], 1.0);
                        assert_eq!(accessor[[0, 1]], 2.0);
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn try_bits_data_err() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let data = vec![1.0, 2.0];
                        let arr = TypedArray::<f32>::from_vec_unchecked(&mut frame, data, (1, 2));
                        let accessor = arr.try_bits_data::<f64>();
                        assert!(accessor.is_err());
                    }

                    Ok(())
                })
                .unwrap();
        });
    }
    fn bits_data_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let data = vec![1.0, 2.0];
                        let arr = TypedArray::<f32>::from_vec_unchecked(&mut frame, data, (1, 2));
                        let accessor = arr.bits_data_unchecked::<f32>();
                        assert_eq!(accessor[[0, 0]], 1.0);
                        assert_eq!(accessor[[0, 1]], 2.0);
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    pub(crate) fn array_bits_data_tests() {
        bits_data();
        bits_data_get();
        bits_data_get_unchecked();
        bits_data_get_value();
        bits_data_get_value_unchecked();
        bits_data_as_slice();
        bits_data_into_slice();
        bits_data_with_layout();
        try_bits_data();
        try_bits_data_err();
        bits_data_unchecked();
    }
}
