mod util;

#[cfg(feature = "local-rt")]
mod tests {
    use jlrs::{data::managed::array::dimensions::Dims, prelude::*};

    use crate::util::JULIA;

    fn borrow_array_1d() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            let mut data = vec![1u64, 2, 3, 4];

            let unboxed = jlrs
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let array = TypedArray::<u64>::from_slice(&mut frame, &mut data, 4)?
                        .into_jlrs_result()?;
                    assert!(array.contains::<u64>());
                    unsafe { Ok(array.bits_data().to_copied_array()) }
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.rank(), 1);
            assert_eq!(dims.n_elements(0), Some(4));
            assert_eq!(data, vec![1, 2, 3, 4].into_boxed_slice());
        });
    }

    fn borrow_in_nested_scope() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            let mut data = vec![1u64, 2, 3, 4];

            let unboxed = jlrs
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let output = frame.output();
                    let array = frame.returning::<JlrsResult<_>>().scope(|mut frame| {
                        let borrowed = &mut data;
                        let arr = TypedArray::<u64>::from_slice(&mut frame, borrowed, 4)?
                            .into_jlrs_result()?;
                        Ok(arr.root(output))
                    })?;

                    // uncommenting the next line must lead to a compilation error due to multiple
                    // mutable borrows:
                    // let _reborrowed = &mut data[0];

                    Module::base(&frame)
                        .function(&frame, "sum")?
                        .as_managed()
                        .call1(&mut frame, array.as_value())
                        .unwrap()
                        .unbox::<u64>()
                })
                .unwrap();

            assert_eq!(unboxed, 10);
        });
    }

    #[test]
    fn borrow_array_tests() {
        borrow_array_1d();
        borrow_in_nested_scope();
    }
}
