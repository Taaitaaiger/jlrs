mod util;

#[cfg(feature = "local-rt")]
mod tests {
    use jlrs::{data::managed::array::dimensions::Dims, prelude::*};

    use crate::util::JULIA;

    fn borrow_array_1d() {
        let mut data = vec![1u64, 2, 3, 4];
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                let unboxed = stack.scope(|mut frame| {
                    let array = TypedArray::<u64>::from_slice(&mut frame, &mut data, 4)
                        .unwrap()
                        .unwrap();
                    assert!(array.contains::<u64>());
                    unsafe { array.bits_data().to_copied_array() }
                });

                let (data, dims) = unboxed.splat();
                assert_eq!(dims.rank(), 1);
                assert_eq!(dims.n_elements(0), Some(4));
                assert_eq!(data, vec![1, 2, 3, 4].into_boxed_slice());
            });
        });
    }

    fn borrow_in_nested_scope() {
        let mut data = vec![1u64, 2, 3, 4];
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                let unboxed = stack
                    .scope(|mut frame| unsafe {
                        let output = frame.output();
                        let array = frame.scope(|mut frame| {
                            let borrowed = &mut data;
                            let arr = TypedArray::<u64>::from_slice(&mut frame, borrowed, 4)
                                .unwrap()
                                .unwrap();
                            arr.root(output)
                        });

                        // uncommenting the next line must lead to a compilation error due to multiple
                        // mutable borrows:
                        // let _reborrowed = &mut data[0];

                        Module::base(&frame)
                            .global(&frame, "sum")
                            .unwrap()
                            .as_managed()
                            .call(&mut frame, [array.as_value()])
                            .unwrap()
                            .unbox::<u64>()
                    })
                    .unwrap();

                assert_eq!(unboxed, 10);
            });
        });
    }

    #[test]
    fn borrow_array_tests() {
        borrow_array_1d();
        borrow_in_nested_scope();
    }
}
