mod util;

#[cfg(feature = "local-rt")]
mod tests {
    /*
    struct HasConstructors
        a::DataType
        b::Union{Int16, Int32}
        HasConstructors(i::Int16) = new(Int16, i)
        HasConstructors(i::Int32) = new(Int32, i)
        HasConstructors(v::Bool) = new(Bool, v ? one(Int32) : zero(Int32))
    end

    HasConstructors() = HasConstructors(false)
    */

    use jlrs::{data::managed::union_all::UnionAll, prelude::*};

    use crate::util::JULIA;

    fn call_outer_constructor() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| {
                    unsafe {
                        let ty = Module::main(&frame)
                            .submodule(&frame, "JlrsTests")
                            .unwrap()
                            .as_managed()
                            .global(&frame, "HasConstructors")
                            .unwrap()
                            .as_value();

                        assert!(ty.is::<DataType>());

                        let res = ty.call(&mut frame, []);
                        assert!(res.is_ok());
                        let value = res.unwrap();
                        let is_bool = value
                            .field_accessor()
                            .field("a")
                            .unwrap()
                            .access::<WeakDataType>()
                            .unwrap()
                            .as_managed()
                            .is::<Bool>();

                        assert!(is_bool);

                        let field_b = value
                            .field_accessor()
                            .field("b")
                            .unwrap()
                            .access::<i32>()
                            .unwrap();

                        assert_eq!(field_b, 0);
                    };
                });

                stack.scope(|mut frame| {
                    unsafe {
                        let ty = Module::main(&frame)
                            .submodule(&frame, "JlrsTests")
                            .unwrap()
                            .as_managed()
                            .global(&frame, "HasConstructors")
                            .unwrap()
                            .as_value();

                        assert!(ty.is::<DataType>());

                        let res = ty.call(&mut frame, []);
                        assert!(res.is_ok());
                        let value = res.unwrap();
                        let is_bool = value
                            .field_accessor()
                            .field("a")
                            .unwrap()
                            .access::<WeakDataType>()
                            .unwrap()
                            .as_managed()
                            .is::<Bool>();

                        assert!(is_bool);

                        let field_b = value
                            .field_accessor()
                            .field("b")
                            .unwrap()
                            .access::<i32>()
                            .unwrap();

                        assert_eq!(field_b, 0);
                    };
                })
            });
        });
    }

    fn call_inner_constructor() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| {
                    unsafe {
                        let ty = Module::main(&frame)
                            .submodule(&frame, "JlrsTests")
                            .unwrap()
                            .as_managed()
                            .global(&frame, "HasConstructors")
                            .unwrap()
                            .as_value();

                        let arg = Value::new(&mut frame, 1i16);

                        let res = ty.call(&mut frame, [arg]);
                        assert!(res.is_ok());
                        let value = res.unwrap();
                        let is_i16 = value
                            .field_accessor()
                            .field("a")
                            .unwrap()
                            .access::<WeakDataType>()
                            .unwrap()
                            .as_managed()
                            .is::<i16>();

                        assert!(is_i16);

                        let field_b = value
                            .field_accessor()
                            .field("b")
                            .unwrap()
                            .access::<i16>()
                            .unwrap();

                        assert_eq!(field_b, 1);
                    };
                })
            });
        });
    }

    fn create_dict<'target, Tgt: Target<'target>>(
        tgt: Tgt,
        key_ty: Value<'_, 'static>,
        value_ty: Value<'_, 'static>,
    ) -> ValueResult<'target, 'static, Tgt> {
        tgt.with_local_scope::<_, 2>(|tgt, mut frame| {
            let dict_ua = Module::base(&frame)
                .global(&mut frame, "Dict")
                .unwrap()
                .cast::<UnionAll>()
                .unwrap();

            unsafe {
                let dict_ty = dict_ua.apply_types(&mut frame, [key_ty, value_ty]).unwrap();
                dict_ty.call(tgt, [])
            }
        })
    }

    fn dict_constructor() {
        JULIA.with(|handle| {
            handle.borrow().local_scope::<_, 2>(|mut frame| {
                let key_ty = DataType::string_type(&frame).as_value();
                let value_ty = DataType::int32_type(&frame).as_value();

                let dict = create_dict(&mut frame, key_ty, value_ty).unwrap();
                let dict2 =
                    unsafe { Value::eval_string(&mut frame, "Dict{String, Int32}()").unwrap() };

                assert_eq!(dict.datatype(), dict2.datatype());
            })
        });
    }

    #[test]
    fn constructor_tests() {
        call_outer_constructor();
        call_inner_constructor();
        dict_constructor();
    }
}
