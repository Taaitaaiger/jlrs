#[cfg(feature = "local-rt")]
pub(crate) mod tests {
    use jlrs::{
        data::{managed::array::data::accessor::Accessor, types::construct_type::ConstantBool},
        prelude::*,
    };

    use crate::util::JULIA;

    #[derive(ValidField, ValidLayout, Debug, Clone, Typecheck, Unbox, ConstructType)]
    #[jlrs(julia_type = "Main.IWRL")]
    #[repr(C)]
    struct IWRL<'scope, 'data> {
        pub(crate) a: i8,
        pub(crate) b: Option<ValueRef<'scope, 'data>>,
    }

    #[derive(ValidField, ValidLayout, Debug, Clone, Typecheck, Unbox)]
    #[jlrs(julia_type = "Main.AIDE")]
    #[repr(C)]
    struct AIDE<'scope, 'data> {
        pub(crate) a: u8,
        pub(crate) b: Option<ValueRef<'scope, 'data>>,
    }

    #[derive(ConstructType, HasLayout)]
    #[jlrs(julia_type = "Main.AIDE", constructor_for = "AIDE", scope_lifetime = true, data_lifetime = true, layout_params = [], elided_params = ["T"], all_params = ["T"])]
    struct AIDETypeConstructor<T> {
        _t: ::std::marker::PhantomData<T>,
    }

    fn inline_data() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let arr =
                            Value::eval_string(&mut frame, "IWRL[IWRL(1,1) IWRL(2,2)]").unwrap();
                        let arr = arr.cast::<TypedArray<IWRL>>().unwrap();
                        let accessor = arr.inline_data();

                        assert_eq!(accessor[[0, 0]].a, 1);
                        assert_eq!(accessor[[0, 1]].a, 2);
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn inline_data_get() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let arr =
                            Value::eval_string(&mut frame, "IWRL[IWRL(1,1) IWRL(2,2)]").unwrap();
                        let arr = arr.cast::<TypedArray<IWRL>>().unwrap();
                        let accessor = arr.inline_data();

                        assert!(accessor.get([0, 0]).is_some());
                        assert!(accessor.get([0, 1]).is_some());
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn inline_data_get_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let arr =
                            Value::eval_string(&mut frame, "IWRL[IWRL(1,1) IWRL(2,2)]").unwrap();
                        let arr = arr.cast::<TypedArray<IWRL>>().unwrap();
                        let accessor = arr.inline_data();

                        assert_eq!(accessor.get_unchecked([0, 0]).a, 1);
                        assert_eq!(accessor.get_unchecked([0, 1]).a, 2);
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn inline_data_as_slice() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let arr =
                            Value::eval_string(&mut frame, "IWRL[IWRL(1,1) IWRL(2,2)]").unwrap();
                        let arr = arr.cast::<TypedArray<IWRL>>().unwrap();
                        let accessor = arr.inline_data();
                        let slice = accessor.as_slice();

                        assert_eq!(slice[0].a, 1);
                        assert_eq!(slice[1].a, 2);
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn inline_data_into_slice() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let arr =
                            Value::eval_string(&mut frame, "IWRL[IWRL(1,1) IWRL(2,2)]").unwrap();
                        let arr = arr.cast::<TypedArray<IWRL>>().unwrap();
                        let accessor = arr.inline_data();
                        let slice = accessor.into_slice();

                        assert_eq!(slice[0].a, 1);
                        assert_eq!(slice[1].a, 2);
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn inline_data_get_value() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let arr =
                            Value::eval_string(&mut frame, "IWRL[IWRL(1,1) IWRL(2,2)]").unwrap();
                        let arr = arr.cast::<TypedArray<IWRL>>().unwrap();
                        let accessor = arr.inline_data();

                        assert_eq!(
                            accessor
                                .get_value(&mut frame, [0, 0])
                                .unwrap()
                                .unwrap()
                                .unbox::<IWRL>()
                                .unwrap()
                                .a,
                            1
                        );
                        assert_eq!(
                            accessor
                                .get_value(&mut frame, [0, 1])
                                .unwrap()
                                .unwrap()
                                .unbox::<IWRL>()
                                .unwrap()
                                .a,
                            2
                        );
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn inline_data_get_value_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let arr =
                            Value::eval_string(&mut frame, "IWRL[IWRL(1,1) IWRL(2,2)]").unwrap();
                        let arr = arr.cast::<TypedArray<IWRL>>().unwrap();
                        let accessor = arr.inline_data();

                        assert_eq!(
                            accessor
                                .get_value_unchecked(&mut frame, [0, 0])
                                .unbox::<IWRL>()
                                .unwrap()
                                .a,
                            1
                        );
                        assert_eq!(
                            accessor
                                .get_value_unchecked(&mut frame, [0, 1])
                                .unbox::<IWRL>()
                                .unwrap()
                                .a,
                            2
                        );
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn inline_data_with_layout() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let arr = Value::eval_string(
                            &mut frame,
                            "AIDE{true}[AIDE{true}(1,1) AIDE{true}(2,2)]",
                        )
                        .unwrap();
                        let arr = arr
                            .cast::<TypedArray<AIDETypeConstructor<ConstantBool<true>>>>()
                            .unwrap();
                        let accessor = arr.inline_data_with_layout();

                        assert_eq!(accessor.get([0, 0]).unwrap().a, 1);
                        assert_eq!(accessor.get([0, 1]).unwrap().a, 2);
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn try_inline_data() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let arr =
                            Value::eval_string(&mut frame, "IWRL[IWRL(1,1) IWRL(2,2)]").unwrap();
                        let arr = arr.cast::<TypedArray<IWRL>>().unwrap();
                        let accessor = arr.try_inline_data::<IWRL>();
                        assert!(accessor.is_ok());
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn try_inline_data_err() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let arr =
                            Value::eval_string(&mut frame, "IWRL[IWRL(1,1) IWRL(2,2)]").unwrap();
                        let arr = arr.cast::<TypedArray<IWRL>>().unwrap();
                        let accessor = arr.try_inline_data::<AIDE>();
                        assert!(accessor.is_err());
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn inline_data_unchecked() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    unsafe {
                        let arr =
                            Value::eval_string(&mut frame, "IWRL[IWRL(1,1) IWRL(2,2)]").unwrap();
                        let arr = arr.cast::<TypedArray<IWRL>>().unwrap();
                        let accessor = arr.inline_data_unchecked::<IWRL>();

                        assert_eq!(accessor[[0, 0]].a, 1);
                        assert_eq!(accessor[[0, 1]].a, 2);
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    pub(crate) fn array_inline_data_tests() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|frame| unsafe {
                    Value::eval_string(&frame, "struct IWRL a::Int8; b end").unwrap();
                    Value::eval_string(&frame, "struct AIDE{T} a::UInt8; b end").unwrap();
                    Ok(())
                })
                .unwrap();
        });

        inline_data();
        inline_data_get();
        inline_data_get_unchecked();
        inline_data_as_slice();
        inline_data_into_slice();

        inline_data_get_value();
        inline_data_get_value_unchecked();

        inline_data_with_layout();
        try_inline_data();
        try_inline_data_err();
        inline_data_unchecked();
    }
}
