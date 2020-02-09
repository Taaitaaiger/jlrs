use jlrs::prelude::*;

#[test]
fn create_and_unbox_uint_data() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    jlrs.session(|session| {
        let p1 = session.new_primitive(1u8)?;
        let p2 = session.new_primitive(2u16)?;
        let p3 = session.new_primitive(3u32)?;
        let p4 = session.new_primitive(4u64)?;
        let p5 = session.new_primitive(5usize)?;

        session.execute(|exec_ctx| {
            let u1 = exec_ctx.try_unbox::<u8>(&p1)?;
            let u2 = exec_ctx.try_unbox::<u16>(&p2)?;
            let u3 = exec_ctx.try_unbox::<u32>(&p3)?;
            let u4 = exec_ctx.try_unbox::<u64>(&p4)?;
            let u5 = exec_ctx.try_unbox::<usize>(&p5)?;

            assert_eq!(u1, 1);
            assert_eq!(u2, 2);
            assert_eq!(u3, 3);
            assert_eq!(u4, 4);
            assert_eq!(u5, 5);

            Ok(())
        })
    })
    .unwrap();
}

#[test]
fn create_and_unbox_uint_data_from_context() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    jlrs.session(|session| {
        session.with_temporaries(|mut alloc_ctx| {
            let p1 = alloc_ctx.new_primitive(1u8)?;
            let p2 = alloc_ctx.new_primitive(2u16)?;
            let p3 = alloc_ctx.new_primitive(3u32)?;
            let p4 = alloc_ctx.new_primitive(4u64)?;
            let p5 = alloc_ctx.new_primitive(5usize)?;

            alloc_ctx.execute(|exec_ctx| {
                let u1 = exec_ctx.try_unbox::<u8>(&p1)?;
                let u2 = exec_ctx.try_unbox::<u16>(&p2)?;
                let u3 = exec_ctx.try_unbox::<u32>(&p3)?;
                let u4 = exec_ctx.try_unbox::<u64>(&p4)?;
                let u5 = exec_ctx.try_unbox::<usize>(&p5)?;

                assert_eq!(u1, 1);
                assert_eq!(u2, 2);
                assert_eq!(u3, 3);
                assert_eq!(u4, 4);
                assert_eq!(u5, 5);

                Ok(())
            })
        })
    })
    .unwrap();
}

#[test]
fn create_and_unbox_int_data() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    jlrs.session(|session| {
        let p1 = session.new_primitive(1i8)?;
        let p2 = session.new_primitive(-2i16)?;
        let p3 = session.new_primitive(3i32)?;
        let p4 = session.new_primitive(-4i64)?;
        let p5 = session.new_primitive(-5isize)?;

        session.execute(|exec_ctx| {
            let u1 = exec_ctx.try_unbox::<i8>(&p1)?;
            let u2 = exec_ctx.try_unbox::<i16>(&p2)?;
            let u3 = exec_ctx.try_unbox::<i32>(&p3)?;
            let u4 = exec_ctx.try_unbox::<i64>(&p4)?;
            let u5 = exec_ctx.try_unbox::<isize>(&p5)?;

            assert_eq!(u1, 1);
            assert_eq!(u2, -2);
            assert_eq!(u3, 3);
            assert_eq!(u4, -4);
            assert_eq!(u5, -5);

            Ok(())
        })
    })
    .unwrap();
}

#[test]
fn create_and_unbox_int_data_from_context() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    jlrs.session(|session| {
        session.with_temporaries(|mut alloc_ctx| {
            let p1 = alloc_ctx.new_primitive(1i8)?;
            let p2 = alloc_ctx.new_primitive(-2i16)?;
            let p3 = alloc_ctx.new_primitive(3i32)?;
            let p4 = alloc_ctx.new_primitive(-4i64)?;
            let p5 = alloc_ctx.new_primitive(-5isize)?;

            alloc_ctx.execute(|exec_ctx| {
                let u1 = exec_ctx.try_unbox::<i8>(&p1)?;
                let u2 = exec_ctx.try_unbox::<i16>(&p2)?;
                let u3 = exec_ctx.try_unbox::<i32>(&p3)?;
                let u4 = exec_ctx.try_unbox::<i64>(&p4)?;
                let u5 = exec_ctx.try_unbox::<isize>(&p5)?;

                assert_eq!(u1, 1);
                assert_eq!(u2, -2);
                assert_eq!(u3, 3);
                assert_eq!(u4, -4);
                assert_eq!(u5, -5);

                Ok(())
            })
        })
    })
    .unwrap();
}

#[test]
fn create_and_unbox_float_data() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    jlrs.session(|session| {
        let p1 = session.new_primitive(1.4f32)?;
        let p2 = session.new_primitive(-2.3f64)?;

        session.execute(|exec_ctx| {
            let u1 = exec_ctx.try_unbox::<f32>(&p1)?;
            let u2 = exec_ctx.try_unbox::<f64>(&p2)?;

            assert_eq!(u1, 1.4);
            assert_eq!(u2, -2.3);

            Ok(())
        })
    })
    .unwrap();
}

#[test]
fn create_and_unbox_float_data_from_context() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    jlrs.session(|session| {
        session.with_temporaries(|mut alloc_ctx| {
            let p1 = alloc_ctx.new_primitive(1.4f32)?;
            let p2 = alloc_ctx.new_primitive(-2.3f64)?;

            alloc_ctx.execute(|exec_ctx| {
                let u1 = exec_ctx.try_unbox::<f32>(&p1)?;
                let u2 = exec_ctx.try_unbox::<f64>(&p2)?;

                assert_eq!(u1, 1.4);
                assert_eq!(u2, -2.3);

                Ok(())
            })
        })
    })
    .unwrap();
}

#[test]
fn create_and_unbox_bool_data() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    jlrs.session(|session| {
        let p1 = session.new_primitive(true)?;
        let p2 = session.new_primitive(false)?;

        session.execute(|exec_ctx| {
            let u1 = exec_ctx.try_unbox::<bool>(&p1)?;
            let u2 = exec_ctx.try_unbox::<bool>(&p2)?;

            assert_eq!(u1, true);
            assert_eq!(u2, false);

            Ok(())
        })
    })
    .unwrap();
}

#[test]
fn create_and_unbox_bool_data_from_context() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    jlrs.session(|session| {
        session.with_temporaries(|mut alloc_ctx| {
            let p1 = alloc_ctx.new_primitive(true)?;
            let p2 = alloc_ctx.new_primitive(false)?;

            alloc_ctx.execute(|exec_ctx| {
                let u1 = exec_ctx.try_unbox::<bool>(&p1)?;
                let u2 = exec_ctx.try_unbox::<bool>(&p2)?;

                assert_eq!(u1, true);
                assert_eq!(u2, false);

                Ok(())
            })
        })
    })
    .unwrap();
}

#[test]
fn create_and_unbox_char_data() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    jlrs.session(|session| {
        let p1 = session.new_primitive('á')?;

        session.execute(|exec_ctx| {
            let u1 = exec_ctx.try_unbox::<char>(&p1)?;
            assert_eq!(u1, 'á');
            Ok(())
        })
    })
    .unwrap();
}

#[test]
fn create_and_unbox_char_data_from_context() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    jlrs.session(|session| {
        session.with_temporaries(|mut alloc_ctx| {
            let p1 = alloc_ctx.new_primitive('á')?;

            alloc_ctx.execute(|exec_ctx| {
                let u1 = exec_ctx.try_unbox::<char>(&p1)?;
                assert_eq!(u1, 'á');
                Ok(())
            })
        })
    })
    .unwrap();
}

#[test]
fn create_and_unbox_dyn_primitives() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    jlrs.session(|session| {
        let ps = session.new_primitives_dyn([&'á' as _, &3usize as _])?;

        session.execute(|exec_ctx| {
            let v0 = ps.get(0);
            let v1 = ps.get(1);
            let c = exec_ctx.try_unbox::<char>(&v0)?;
            let n = exec_ctx.try_unbox::<usize>(&v1)?;

            assert_eq!(c, 'á');
            assert_eq!(n, 3);

            Ok(())
        })
    })
    .unwrap();
}

#[test]
fn create_and_unbox_dyn_primitives_from_context() {
    let mut jlrs = unsafe { Runtime::testing_instance() };

    jlrs.session(|session| {
        session.with_temporaries(|mut alloc_ctx| {
            let ps = alloc_ctx.new_primitives_dyn([&'á' as _, &3usize as _])?;

            alloc_ctx.execute(|exec_ctx| {
                let v0 = ps.get(0);
                let v1 = ps.get(1);
                let c = exec_ctx.try_unbox::<char>(&v0)?;
                let n = exec_ctx.try_unbox::<usize>(&v1)?;

                assert_eq!(c, 'á');
                assert_eq!(n, 3);

                Ok(())
            })
        })
    })
    .unwrap();
}
