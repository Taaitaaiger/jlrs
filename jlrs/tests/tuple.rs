mod util;
#[cfg(feature = "sync-rt")]
mod tests {
    use super::util::JULIA;
    use jlrs::prelude::*;

    #[test]
    fn create_cast_tuple0() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let t0 = Tuple0();
                    let v = Value::new(&mut frame, t0);
                    assert!(v.is::<Tuple0>());
                    assert!(v.unbox::<Tuple0>().is_ok());
                    Ok(())
                })
                .unwrap();
        })
    }

    #[test]
    fn create_cast_tuple1() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let t1 = Tuple1(1u64);
                    let v = Value::new(&mut frame, t1);
                    assert!(v.is::<Tuple1<u64>>());
                    assert!(v.unbox::<Tuple1<u64>>().is_ok());
                    Ok(())
                })
                .unwrap();
        })
    }

    #[test]
    fn create_cast_tuple2() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();
            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let t2 = Tuple2(1u64, -3i32);
                    let v = Value::new(&mut frame, t2);
                    assert!(v.is::<Tuple2<u64, i32>>());
                    assert!(v.unbox::<Tuple2<u64, i32>>().is_ok());
                    Ok(())
                })
                .unwrap();
        })
    }
}
