mod util;
#[cfg(feature = "sync-rt")]
#[cfg(not(feature = "lts"))]
mod tests {
    use super::util::JULIA;
    use jlrs::prelude::*;

    #[test]
    fn read_atomic_field() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            jlrs.scope(|global, frame| {
                let ty = unsafe {
                    Module::main(global)
                        .submodule_ref("JlrsStableTests")?
                        .wrapper_unchecked()
                        .global_ref("WithAtomic")?
                        .value_unchecked()
                };

                let arg1 = Value::new(&mut *frame, 3u32)?;
                let instance = ty
                    .cast::<DataType>()?
                    .instantiate(&mut *frame, &mut [arg1])?
                    .into_jlrs_result()?;

                let a = instance.get_raw_field::<u32, _>("a")?;
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
            jlrs.scope(|global, frame| {
                let ty = unsafe {
                    Module::main(global)
                        .submodule_ref("JlrsStableTests")?
                        .wrapper_unchecked()
                        .global_ref("WithLargeAtomic")?
                        .value_unchecked()
                };

                let tup = Value::new(&mut *frame, Tuple4(1usize, 2usize, 3usize, 4usize))?;

                let instance = ty
                    .cast::<DataType>()?
                    .instantiate(&mut *frame, &mut [tup])?
                    .into_jlrs_result()?;

                let a = instance.get_raw_field::<Tuple4<usize, usize, usize, usize>, _>("a")?;
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
            jlrs.scope(|global, frame| {
                let ty = unsafe {
                    Module::main(global)
                        .submodule_ref("JlrsStableTests")?
                        .wrapper_unchecked()
                        .global_ref("WithOddlySizedAtomic")?
                        .value_unchecked()
                };

                let tup = Value::new(&mut *frame, Tuple2(1u32, 2u16))?;

                let instance = ty
                    .cast::<DataType>()?
                    .instantiate(&mut *frame, &mut [tup])?
                    .into_jlrs_result()?;

                let a = instance.get_raw_field::<Tuple2<u32, u16>, _>("a")?;
                assert_eq!(a, Tuple2(1, 2));

                Ok(())
            })
            .unwrap();
        })
    }
}
