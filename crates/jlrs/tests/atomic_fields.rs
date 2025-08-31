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
                stack.scope(|mut frame| unsafe {
                    let ty = {
                        Module::main(&frame)
                            .submodule(&frame, "JlrsStableTests")
                            .unwrap()
                            .as_managed()
                            .global(&frame, "WithAtomic")
                            .unwrap()
                            .as_value()
                    };

                    let arg1 = Value::new(&mut frame, 3u32);
                    let instance = ty.call(&mut frame, &mut [arg1]).unwrap();

                    let a = instance
                        .field_accessor()
                        .field("a")
                        .unwrap()
                        .access::<u32>()
                        .unwrap();
                    assert_eq!(a, 3);
                })
            })
        })
    }

    fn read_large_atomic_field() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let ty = {
                        Module::main(&frame)
                            .submodule(&frame, "JlrsStableTests")
                            .unwrap()
                            .as_managed()
                            .global(&frame, "WithLargeAtomic")
                            .unwrap()
                            .as_value()
                    };

                    let tup = Value::new(&mut frame, Tuple4(1u64, 2u64, 3u64, 4u64));
                    let instance = ty.call(&mut frame, &mut [tup]).unwrap();

                    let a = instance
                        .field_accessor()
                        .field("a")
                        .unwrap()
                        .access::<Tuple4<u64, u64, u64, u64>>()
                        .unwrap();
                    assert_eq!(a, Tuple4(1, 2, 3, 4));
                })
            })
        })
    }

    fn read_oddly_sized_atomic_field() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let ty = {
                        Module::main(&frame)
                            .submodule(&frame, "JlrsStableTests")
                            .unwrap()
                            .as_managed()
                            .global(&frame, "WithOddlySizedAtomic")
                            .unwrap()
                            .as_value()
                    };

                    let tup = Value::new(&mut frame, Tuple2(1u32, 2u16));

                    let instance = ty.call(&mut frame, &mut [tup]).unwrap();

                    let a = instance
                        .field_accessor()
                        .field("a")
                        .unwrap()
                        .access::<Tuple2<u32, u16>>()
                        .unwrap();
                    assert_eq!(a, Tuple2(1, 2));
                })
            })
        })
    }

    fn atomic_union_is_pointer_field() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|frame| {
                    let ty = unsafe {
                        Module::main(&frame)
                            .submodule(&frame, "JlrsStableTests")
                            .unwrap()
                            .as_managed()
                            .global(&frame, "WithAtomicUnion")
                            .unwrap()
                            .as_value()
                    };

                    assert!(ty.cast::<DataType>().unwrap().is_pointer_field(0).unwrap());
                    assert!(ty.cast::<DataType>().unwrap().is_atomic_field(0).unwrap());
                    assert!(ty.cast::<DataType>().unwrap().is_atomic_field(20).is_none());
                })
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
