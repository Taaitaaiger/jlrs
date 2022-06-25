mod util;
#[cfg(feature = "sync-rt")]
#[cfg(any(not(feature = "lts"), feature = "all-features-override"))]
mod tests {
    use super::util::JULIA;
    use jlrs::prelude::*;

    #[test]
    fn read_atomic_field() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, mut frame| {
                let ty = unsafe {
                    Module::main(global)
                        .submodule_ref("JlrsStableTests")?
                        .wrapper_unchecked()
                        .global_ref("WithAtomic")?
                        .value_unchecked()
                };

                let arg1 = Value::new(&mut frame, 3u32)?;
                let instance = ty
                    .cast::<DataType>()?
                    .instantiate(&mut frame, &mut [arg1])?
                    .into_jlrs_result()?;

                let a = instance
                    .field_accessor(&mut frame)
                    .field("a")?
                    .access::<u32>()?;
                assert_eq!(a, 3);

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn read_large_atomic_field() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, mut frame| {
                let ty = unsafe {
                    Module::main(global)
                        .submodule_ref("JlrsStableTests")?
                        .wrapper_unchecked()
                        .global_ref("WithLargeAtomic")?
                        .value_unchecked()
                };

                let tup = Value::new(&mut frame, Tuple4(1usize, 2usize, 3usize, 4usize))?;

                let instance = ty
                    .cast::<DataType>()?
                    .instantiate(&mut frame, &mut [tup])?
                    .into_jlrs_result()?;

                let a = instance
                    .field_accessor(&mut frame)
                    .field("a")?
                    .access::<Tuple4<usize, usize, usize, usize>>()?;
                assert_eq!(a, Tuple4(1, 2, 3, 4));

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn read_oddly_sized_atomic_field() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, mut frame| {
                let ty = unsafe {
                    Module::main(global)
                        .submodule_ref("JlrsStableTests")?
                        .wrapper_unchecked()
                        .global_ref("WithOddlySizedAtomic")?
                        .value_unchecked()
                };

                let tup = Value::new(&mut frame, Tuple2(1u32, 2u16))?;

                let instance = ty
                    .cast::<DataType>()?
                    .instantiate(&mut frame, &mut [tup])?
                    .into_jlrs_result()?;

                let a = instance
                    .field_accessor(&mut frame)
                    .field("a")?
                    .access::<Tuple2<u32, u16>>()?;
                assert_eq!(a, Tuple2(1, 2));

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn atomic_union_is_pointer_field() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, _frame| {
                let ty = unsafe {
                    Module::main(global)
                        .submodule_ref("JlrsStableTests")?
                        .wrapper_unchecked()
                        .global_ref("WithAtomicUnion")?
                        .value_unchecked()
                };

                assert!(ty.cast::<DataType>()?.is_pointer_field(0)?);
                assert!(ty.cast::<DataType>()?.is_atomic_field(0)?);

                Ok(())
            })
            .unwrap();
        })
    }

    #[test]
    fn read_atomic_field_of_ptr_wrapper() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, _frame| {
                unsafe {
                    assert_eq!(
                        DataType::datatype_type(global)
                            .type_name()
                            .wrapper_unchecked()
                            .cache()
                            .wrapper_unchecked()
                            .len(),
                        0
                    )
                }

                Ok(())
            })
            .unwrap();
        })
    }
}
