use jlrs::prelude::*;

#[test]
fn borrow_array_1d() {
    let mut jlrs = unsafe { Julia::testing_instance() };
    let mut data = vec![1u64, 2, 3, 4];

    let unboxed = jlrs
        .frame(1, |frame| {
            let array = unsafe { Value::borrow_array(frame, &mut data, 4) }?;
            array.try_unbox::<Array<u64>>()
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 1);
    assert_eq!(dims.n_elements(0), 4);
    assert_eq!(data, vec![1, 2, 3, 4]);
}

#[test]
fn borrow_array_1d_dynamic() {
    let mut jlrs = unsafe { Julia::testing_instance() };
    let mut data = vec![1u64, 2, 3, 4];

    let unboxed = jlrs
        .dynamic_frame(|frame| {
            let array = unsafe { Value::borrow_array(frame, &mut data, 4) }?;
            array.try_unbox::<Array<u64>>()
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 1);
    assert_eq!(dims.n_elements(0), 4);
    assert_eq!(data, vec![1, 2, 3, 4]);
}

#[test]
fn borrow_array_2d() {
    let mut jlrs = unsafe { Julia::testing_instance() };
    let mut data = vec![1u64, 2, 3, 4];

    let unboxed = jlrs
        .frame(1, |frame| {
            let array = unsafe { Value::borrow_array(frame, &mut data, (2, 2)) }?;
            array.try_unbox::<Array<u64>>()
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 2);
    assert_eq!(dims.n_elements(0), 2);
    assert_eq!(dims.n_elements(1), 2);
    assert_eq!(data, vec![1, 2, 3, 4]);
}

#[test]
fn borrow_array_2d_dynamic() {
    let mut jlrs = unsafe { Julia::testing_instance() };
    let mut data = vec![1u64, 2, 3, 4];

    let unboxed = jlrs
        .dynamic_frame(|frame| {
            let array = unsafe { Value::borrow_array(frame, &mut data, (2, 2)) }?;
            array.try_unbox::<Array<u64>>()
        })
        .unwrap();
        
    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 2);
    assert_eq!(dims.n_elements(0), 2);
    assert_eq!(dims.n_elements(1), 2);
    assert_eq!(data, vec![1, 2, 3, 4]);
}
