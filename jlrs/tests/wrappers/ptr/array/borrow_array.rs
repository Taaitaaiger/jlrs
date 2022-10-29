#[cfg(feature = "sync-rt")]
#[cfg(not(all(target_os = "windows", feature = "lts")))]
mod tests {
    use crate::util::JULIA;
    use jlrs::prelude::*;
    use jlrs::wrappers::ptr::array::dimensions::Dims;

    #[test]
    fn borrow_array_1d() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            let mut data = vec![1u64, 2, 3, 4];

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| {
                    let array = Array::from_slice(frame.as_extended_target(), &mut data, 4)?
                        .into_jlrs_result()?;
                    assert!(array.contains::<u64>());
                    unsafe { array.copy_inline_data::<u64>() }
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.n_dimensions(), 1);
            assert_eq!(dims.n_elements(0), 4);
            assert_eq!(data, vec![1, 2, 3, 4].into_boxed_slice());
        });
    }

    /*
    #[test]
    fn borrow_array_1d_dynamic_type() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            struct Foo<'a> {
                slice: &'a mut [u8],
            }

            let foo = Foo {
                slice: &mut [1, 2, 3, 4, 5, 6, 7, 8],
            };

            let unboxed = jlrs
                .scope(|mut frame| {
                    let x = false;

                    let array = match x {
                        true => Array::from_slice(frame, foo.slice, 8)?,
                        false => unsafe {
                            let ptr = foo.slice.as_mut_ptr().cast::<u16>();
                            let slice = std::slice::from_raw_parts_mut(ptr, 4);
                            Array::from_slice(frame, slice, 4)?
                        },
                    };

                    assert!(array.is_array_of::<u16>());
                    array.cast::<Array>()?.copy_inline_data::<u16>()
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.n_dimensions(), 1);
            assert_eq!(dims.n_elements(0), 4);
            assert_eq!(data, vec![513, 1027, 1541, 2055]);
        });
    }

    #[test]
    fn borrow_array_1d_output() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            let mut data = vec![1u64, 2, 3, 4];

            let unboxed = jlrs
                .scope(|mut frame| {
                    let array = frame.value_scope_with_slots(0, |output, mut frame| {
                        let output = output.into_scope(frame);
                        Array::from_slice(output, &mut data, 4)
                    })?;
                    assert!(array.is_array_of::<u64>());
                    array.cast::<Array>()?.copy_inline_data::<u64>()
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.n_dimensions(), 1);
            assert_eq!(dims.n_elements(0), 4);
            assert_eq!(data, vec![1, 2, 3, 4]);
        });
    }

    #[test]
    fn borrow_array_1d_dynamic() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            let mut data = vec![1u64, 2, 3, 4];

            let unboxed = jlrs
                .scope(|mut frame| {
                    let array = Array::from_slice(frame, &mut data, 4)?;
                    array.cast::<Array>()?.copy_inline_data::<u64>()
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.n_dimensions(), 1);
            assert_eq!(dims.n_elements(0), 4);
            assert_eq!(data, vec![1, 2, 3, 4]);
        });
    }

    #[test]
    fn borrow_array_2d() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            let mut data = vec![1u64, 2, 3, 4];

            let unboxed = jlrs
                .scope(|mut frame| {
                    let array = Array::from_slice(frame, &mut data, (2, 2))?;
                    array.cast::<Array>()?.copy_inline_data::<u64>()
                })
                .unwrap();

            let (data, dims) = unboxed.splat();
            assert_eq!(dims.n_dimensions(), 2);
            assert_eq!(dims.n_elements(0), 2);
            assert_eq!(dims.n_elements(1), 2);
            assert_eq!(data, vec![1, 2, 3, 4]);
        });
    }

    #[test]
    fn borrow_array_2d_dynamic() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            let mut data = vec![1u64, 2, 3, 4];

            let unboxed = jlrs
                .scope(|mut frame| {
                    let array = Array::from_slice(frame, &mut data, (2, 2))?;
                    array.cast::<Array>()?.copy_inline_data::<u64>()
                })
                .unwrap();
            let (data, dims) = unboxed.splat();
            assert_eq!(dims.n_dimensions(), 2);
            assert_eq!(dims.n_elements(0), 2);
            assert_eq!(dims.n_elements(1), 2);
            assert_eq!(data, vec![1, 2, 3, 4]);
        });
    }

    #[test]
    fn call_function_with_borrowed() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            let mut data = vec![1u64, 2, 3, 4];

            let unboxed = jlrs
                .scope(|mut frame| unsafe {
                    let array = Array::from_slice(&mut frame, &mut data, 4)?;
                    Module::base(&frame)
                        .function(&frame, "sum")?
                        .wrapper_unchecked()
                        .call1(&mut frame, array)?
                        .unwrap()
                        .unbox::<u64>()
                })
                .unwrap();

            assert_eq!(unboxed, 10);
        });
    }
     */

    #[test]
    fn borrow_in_nested_scope() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            let mut data = vec![1u64, 2, 3, 4];

            let unboxed = jlrs
                .instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let output = frame.output();
                    let array = frame.scope(|mut frame| {
                        let borrowed = &mut data;
                        let arr = Array::from_slice(frame.as_extended_target(), borrowed, 4)?
                            .into_jlrs_result()?;
                        Ok(arr.root(output))
                    })?;

                    // uncommenting the next line must lead to a compilation error due to multiple
                    // mutable borrows:
                    // let _reborrowed = &mut data[0];

                    Module::base(&frame)
                        .function(&frame, "sum")?
                        .wrapper_unchecked()
                        .call1(&mut frame, array.as_value())
                        .unwrap()
                        .unbox::<u64>()
                })
                .unwrap();

            assert_eq!(unboxed, 10);
        });
    }
}
