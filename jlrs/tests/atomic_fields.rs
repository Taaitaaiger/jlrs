mod util;
#[cfg(feature = "local-rt")]
mod tests {
    use jlrs::{
        data::layout::tuple::{Tuple2, Tuple4},
        prelude::*,
    };

    use super::util::JULIA;

    fn read_atomic_field() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|mut frame| unsafe {
                        let ty = {
                            Module::main(&frame)
                                .submodule(&frame, "JlrsStableTests")?
                                .as_managed()
                                .global(&frame, "WithAtomic")?
                                .as_value()
                        };

                        let arg1 = Value::new(&mut frame, 3u32);
                        let instance = ty.call(&mut frame, &mut [arg1]).into_jlrs_result()?;

                        let a = instance.field_accessor().field("a")?.access::<u32>()?;
                        assert_eq!(a, 3);

                        Ok(())
                    })
                    .unwrap();
            })
        })
    }

    fn read_large_atomic_field() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|mut frame| unsafe {
                        let ty = {
                            Module::main(&frame)
                                .submodule(&frame, "JlrsStableTests")?
                                .as_managed()
                                .global(&frame, "WithLargeAtomic")?
                                .as_value()
                        };

                        let tup = Value::new(&mut frame, Tuple4(1u64, 2u64, 3u64, 4u64));
                        let instance = ty.call(&mut frame, &mut [tup]).into_jlrs_result()?;

                        let a = instance
                            .field_accessor()
                            .field("a")?
                            .access::<Tuple4<u64, u64, u64, u64>>()?;
                        assert_eq!(a, Tuple4(1, 2, 3, 4));

                        Ok(())
                    })
                    .unwrap();
            })
        })
    }

    fn read_oddly_sized_atomic_field() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|mut frame| unsafe {
                        let ty = {
                            Module::main(&frame)
                                .submodule(&frame, "JlrsStableTests")?
                                .as_managed()
                                .global(&frame, "WithOddlySizedAtomic")?
                                .as_value()
                        };

                        let tup = Value::new(&mut frame, Tuple2(1u32, 2u16));

                        let instance = ty.call(&mut frame, &mut [tup]).into_jlrs_result()?;

                        let a = instance
                            .field_accessor()
                            .field("a")?
                            .access::<Tuple2<u32, u16>>()?;
                        assert_eq!(a, Tuple2(1, 2));

                        Ok(())
                    })
                    .unwrap();
            })
        })
    }

    fn atomic_union_is_pointer_field() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack
                    .returning::<JlrsResult<_>>()
                    .scope(|frame| {
                        let ty = unsafe {
                            Module::main(&frame)
                                .submodule(&frame, "JlrsStableTests")?
                                .as_managed()
                                .global(&frame, "WithAtomicUnion")?
                                .as_value()
                        };

                        assert!(ty.cast::<DataType>()?.is_pointer_field(0).unwrap());
                        assert!(ty.cast::<DataType>()?.is_atomic_field(0).unwrap());
                        assert!(ty.cast::<DataType>()?.is_atomic_field(20).is_none());

                        Ok(())
                    })
                    .unwrap();
            })
        })
    }

    #[test]
    fn atomic_field_tests() {
        read_atomic_field();
        read_large_atomic_field();
        read_oddly_sized_atomic_field();
        atomic_union_is_pointer_field();
    }
}
