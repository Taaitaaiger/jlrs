mod util;

#[cfg(test)]
#[cfg(all(feature = "sync-rt", feature = "jlrs-ndarray"))]
mod tests {
    use super::util::JULIA;
    use jlrs::convert::ndarray::{NdArrayView, NdArrayViewMut};
    use jlrs::memory::stack_frame::StackFrame;
    use jlrs::wrappers::ptr::array::{Array, TypedArray};

    fn bits_array_view() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .scope(|mut frame| {
                    let mut data = vec![1usize, 2, 3, 4, 5, 6];
                    let slice = &mut data.as_mut_slice();
                    let borrowed = unsafe {
                        Array::from_slice_unchecked(frame.as_extended_target(), slice, (3, 2))?
                    };

                    let data = unsafe { borrowed.bits_data::<usize>()? };
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
                .scope(|mut frame| unsafe {
                    let mut data = vec![1usize, 2, 3, 4, 5, 6];
                    let slice = &mut data.as_mut_slice();
                    let mut borrowed =
                        Array::from_slice_unchecked(frame.as_extended_target(), slice, (3, 2))?;

                    let mut inline = borrowed.bits_data_mut::<usize>()?;
                    let x = inline[(2, 1)];

                    inline[(2, 1)] = x + 1;

                    let mut array = inline.array_view_mut();
                    assert_eq!(array[[2, 1]], x + 1);
                    array[[2, 1]] -= 1;

                    std::mem::drop(inline);
                    let inline = borrowed.bits_data_mut::<usize>()?;
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
                .scope(|mut frame| {
                    let mut data = vec![1usize, 2, 3, 4, 5, 6];
                    let slice = &mut data.as_mut_slice();
                    let borrowed = unsafe {
                        Array::from_slice_unchecked(frame.as_extended_target(), slice, (3, 2))?
                    };

                    let data = unsafe { borrowed.inline_data::<usize>()? };
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
                .scope(|mut frame| {
                    let mut data = vec![1usize, 2, 3, 4, 5, 6];
                    let slice = &mut data.as_mut_slice();
                    let borrowed = unsafe {
                        TypedArray::from_slice_unchecked(frame.as_extended_target(), slice, (3, 2))?
                    };
                    let copied = unsafe { borrowed.copy_inline_data()? };

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
                .scope(|mut frame| unsafe {
                    let mut data = vec![1usize, 2, 3, 4, 5, 6];
                    let slice = &mut data.as_mut_slice();
                    let mut borrowed =
                        Array::from_slice_unchecked(frame.as_extended_target(), slice, (3, 2))?;
                    let mut copied = borrowed.copy_inline_data()?;
                    let x = copied[(2, 1)];

                    copied[(2, 1)] = x + 1;

                    let mut array = copied.array_view_mut();
                    assert_eq!(array[[2, 1]], x + 1);
                    array[[2, 1]] -= 1;

                    let inline = borrowed.bits_data_mut::<usize>()?;
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
