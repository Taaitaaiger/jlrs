use jlrs::prelude::*;

#[test]
fn managed_array_1d() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    let unboxed = jlrs
        .session(|session| {
            let array = session.new_managed_array::<f32, _>(3)?;
            session.execute(|exec_ctx| {
                let array = array.set_all(exec_ctx, 2.0)?;
                exec_ctx.try_unbox::<UnboxedArray<f32>>(&array)
            })
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 1);
    assert_eq!(dims.n_elements(0), 3);

    assert_eq!(data, vec![2.0; 3]);
}

#[test]
fn managed_array_1d_from_context() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    let unboxed = jlrs
        .session(|session| {
            session.with_temporaries(|mut alloc_ctx| {
                let array = alloc_ctx.new_managed_array::<f32, _>(3)?;

                alloc_ctx.execute(|exec_ctx| {
                    let array = array.set_all(exec_ctx, 2.0)?;
                    exec_ctx.try_unbox::<UnboxedArray<f32>>(&array)
                })
            })
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 1);
    assert_eq!(dims.n_elements(0), 3);

    assert_eq!(data, vec![2.0; 3]);
}

#[test]
fn push_managed_array_1d() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    let unboxed = jlrs
        .session(|session| {
            let array = session.new_managed_array::<u64, _>(3)?;
            let el = session.new_primitive(5u64)?;
            let out = session.new_unassigned()?;

            let base = session.base_module();
            let func = base.function("push!")?;

            session.execute(|exec_ctx| {
                let array = array.set_all(exec_ctx, 2)?;
                func.call2(exec_ctx, out, array, el)?;
                exec_ctx.try_unbox::<UnboxedArray<u64>>(&array)
            })
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 1);
    assert_eq!(dims.n_elements(0), 4);

    assert_eq!(data, vec![2, 2, 2, 5]);
}

#[test]
fn managed_array_2d() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    let unboxed = jlrs
        .session(|session| {
            let array = session.new_managed_array::<f32, _>((3, 4))?;
            session.execute(|exec_ctx| {
                let array = array.set_from(
                    exec_ctx,
                    [1.0, 2.0, 1.0, 2.0, 1.0, 2.0, 1.0, 2.0, 1.0, 2.0, 1.0, 2.0],
                )?;
                exec_ctx.try_unbox::<UnboxedArray<f32>>(&array)
            })
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 2);
    assert_eq!(dims.n_elements(0), 3);
    assert_eq!(dims.n_elements(1), 4);

    assert_eq!(
        data,
        vec![1.0, 2.0, 1.0, 2.0, 1.0, 2.0, 1.0, 2.0, 1.0, 2.0, 1.0, 2.0]
    );
}

#[test]
fn managed_array_3d() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    let unboxed = jlrs
        .session(|session| {
            let array = session.new_managed_array::<f32, _>((3, 4, 5))?;
            session.execute(|exec_ctx| {
                let array = array.set_all(exec_ctx, 2.0)?;
                exec_ctx.try_unbox::<UnboxedArray<f32>>(&array)
            })
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 3);
    assert_eq!(dims.n_elements(0), 3);
    assert_eq!(dims.n_elements(1), 4);
    assert_eq!(dims.n_elements(2), 5);

    assert_eq!(data, vec![2.0; 60]);
}

#[test]
fn managed_array_nd() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    let unboxed = jlrs
        .session(|session| {
            let array = session.new_managed_array::<f32, _>((2, 3, 4, 5))?;
            session.execute(|exec_ctx| {
                let array = array.set_all(exec_ctx, 2.0)?;
                exec_ctx.try_unbox::<UnboxedArray<f32>>(&array)
            })
        })
        .unwrap();

    let (data, dims) = unboxed.splat();
    assert_eq!(dims.n_dimensions(), 4);
    assert_eq!(dims.n_elements(0), 2);
    assert_eq!(dims.n_elements(1), 3);
    assert_eq!(dims.n_elements(2), 4);
    assert_eq!(dims.n_elements(3), 5);

    assert_eq!(data, vec![2.0; 120]);
}
