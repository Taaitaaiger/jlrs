/*use jlrs::prelude::*;

#[test]
fn borrow_array_1d() {
    let mut jlrs = unsafe { Runtime::testing_instance() };
    let mut data = vec![1u64, 2, 3, 4];

    let unboxed = jlrs
        .session(|session| {
            let array = unsafe { session.borrow_array(&mut data, 4) }?;
            session.execute(|exec_ctx| exec_ctx.try_unbox::<UnboxedArray<u64>>(&array))
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 1);
    assert_eq!(dims.n_elements(0), 4);

    assert_eq!(data, vec![1, 2, 3, 4]);
}

#[test]
fn borrow_array_1d_from_context() {
    let mut jlrs = unsafe { Runtime::testing_instance() };
    let mut data = vec![1u64, 2, 3, 4];

    let unboxed = jlrs
        .session(|session| {
            session.with_temporaries(|mut alloc_ctx| {
                let array = unsafe { alloc_ctx.borrow_array(&mut data, 4) }?;
                alloc_ctx.execute(|exec_ctx| exec_ctx.try_unbox::<UnboxedArray<u64>>(&array))
            })
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 1);
    assert_eq!(dims.n_elements(0), 4);

    assert_eq!(data, vec![1, 2, 3, 4]);
}

#[test]
fn borrow_array_2d() {
    let mut jlrs = unsafe { Runtime::testing_instance() };
    let mut data = vec![1u64, 2, 3, 4];

    let unboxed = jlrs
        .session(|session| {
            let array = unsafe { session.borrow_array(&mut data, (2, 2)) }?;
            session.execute(|exec_ctx| exec_ctx.try_unbox::<UnboxedArray<u64>>(&array))
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 2);
    assert_eq!(dims.n_elements(0), 2);
    assert_eq!(dims.n_elements(1), 2);

    assert_eq!(data, vec![1, 2, 3, 4]);
}

#[test]
fn borrow_array_2d_multiply() {
    let mut jlrs = unsafe { Runtime::testing_instance() };
    let mut data = vec![1u64, 2, 2, 1];

    let unboxed = jlrs
        .session(|session| {
            let func = session.base_module().function("*")?;
            let array = unsafe { session.borrow_array(&mut data, (2, 2)) }?;
            let out = session.new_unassigned()?;

            session.execute(|exec_ctx| {
                let out = func.call2(exec_ctx, out, array, array)?;
                exec_ctx.try_unbox::<UnboxedArray<u64>>(&out)
            })
        })
        .unwrap();

    let (data2, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 2);
    assert_eq!(dims.n_elements(0), 2);
    assert_eq!(dims.n_elements(1), 2);

    assert_eq!(data2, vec![5, 4, 4, 5]);
    assert_eq!(data, vec![1, 2, 2, 1]);
}

#[test]
fn borrow_array_3d() {
    let mut jlrs = unsafe { Runtime::testing_instance() };
    let mut data = vec![1u64, 2, 3, 4, 5, 6, 7, 8];

    let unboxed = jlrs
        .session(|session| {
            let array = unsafe { session.borrow_array(&mut data, (2, 2, 2)) }?;
            session.execute(|exec_ctx| exec_ctx.try_unbox::<UnboxedArray<u64>>(&array))
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 3);
    assert_eq!(dims.n_elements(0), 2);
    assert_eq!(dims.n_elements(1), 2);
    assert_eq!(dims.n_elements(2), 2);

    assert_eq!(data, vec![1, 2, 3, 4, 5, 6, 7, 8]);
}

#[test]
fn borrow_array_nd() {
    let mut jlrs = unsafe { Runtime::testing_instance() };
    let mut data = vec![1u64, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4, 5, 6, 7, 8];

    let unboxed = jlrs
        .session(|session| {
            let array = unsafe { session.borrow_array(&mut data, (2, 2, 2, 2)) }?;
            session.execute(|exec_ctx| exec_ctx.try_unbox::<UnboxedArray<u64>>(&array))
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 4);
    assert_eq!(dims.n_elements(0), 2);
    assert_eq!(dims.n_elements(1), 2);
    assert_eq!(dims.n_elements(2), 2);
    assert_eq!(dims.n_elements(3), 2);

    assert_eq!(data, vec![1, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4, 5, 6, 7, 8]);
}
*/