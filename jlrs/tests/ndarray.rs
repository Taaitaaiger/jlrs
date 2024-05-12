mod util;

#[cfg(test)]
#[cfg(all(feature = "local-rt", feature = "jlrs-ndarray"))]
mod tests {
    use jlrs::{
        convert::ndarray::{NdArrayView, NdArrayViewMut},
        data::managed::array::TypedArray,
        memory::stack_frame::StackFrame,
        prelude::*,
    };

    use super::util::JULIA;

    fn bits_array_view() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let mut data = vec![1usize, 2, 3, 4, 5, 6];
                    let slice = &mut data.as_mut_slice();
                    let borrowed = unsafe {
                        TypedArray::<usize>::from_slice_unchecked(&mut frame, slice, (3, 2))
                    };

                    let data = unsafe { borrowed.bits_data() };
                    let x = data[(2, 1)];

                    let array = data.array_view();
                    assert_eq!(array[[2, 1]], x);

                    Ok(())
                })
                .unwrap();
        });
    }

    fn bits_array_view_mut() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let mut data = vec![1usize, 2, 3, 4, 5, 6];
                    let slice = &mut data.as_mut_slice();
                    let mut borrowed =
                        TypedArray::<usize>::from_slice_unchecked(&mut frame, slice, (3, 2));

                    let mut inline = borrowed.bits_data_mut();
                    let x = inline[(2, 1)];

                    inline[(2, 1)] = x + 1;

                    let mut array = inline.array_view_mut();
                    assert_eq!(array[[2, 1]], x + 1);
                    array[[2, 1]] -= 1;

                    std::mem::drop(inline);
                    let inline = borrowed.bits_data_mut();
                    assert_eq!(inline[(2, 1)], x);
                    Ok(())
                })
                .unwrap();
        });
    }

    fn inline_array_view() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let mut data = vec![1usize, 2, 3, 4, 5, 6];
                    let slice = &mut data.as_mut_slice();
                    let borrowed = unsafe {
                        TypedArray::<usize>::from_slice_unchecked(&mut frame, slice, (3, 2))
                    };

                    let data = unsafe { borrowed.inline_data() };
                    let x = data[(2, 1)];

                    let array = data.array_view();
                    assert_eq!(array[[2, 1]], x);

                    Ok(())
                })
                .unwrap();
        });
    }

    fn copied_array_view() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let mut data = vec![1usize, 2, 3, 4, 5, 6];
                    let slice = &mut data.as_mut_slice();
                    let borrowed = unsafe {
                        TypedArray::<usize>::from_slice_unchecked(&mut frame, slice, (3, 2))
                    };
                    let copied = unsafe { borrowed.bits_data().to_copied_array() };

                    let x = copied[(2, 1)];

                    let array = copied.array_view();
                    assert_eq!(array[[2, 1]], x);

                    Ok(())
                })
                .unwrap();
        });
    }

    fn copied_array_view_mut() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let mut data = vec![1usize, 2, 3, 4, 5, 6];
                    let slice = &mut data.as_mut_slice();
                    let borrowed =
                        TypedArray::<usize>::from_slice_unchecked(&mut frame, slice, (3, 2));
                    let mut copied = borrowed.bits_data().to_copied_array();
                    let x = copied[(2, 1)];

                    copied[(2, 1)] = x + 1;

                    let mut array = copied.array_view_mut();
                    assert_eq!(array[[2, 1]], x + 1);
                    array[[2, 1]] -= 1;

                    let inline = borrowed.bits_data();
                    assert_eq!(inline[(2, 1)], x);
                    Ok(())
                })
                .unwrap();
        });
    }

    #[test]
    fn ndarray_tests() {
        bits_array_view();
        bits_array_view_mut();
        inline_array_view();
        copied_array_view();
        copied_array_view_mut();
    }
}
