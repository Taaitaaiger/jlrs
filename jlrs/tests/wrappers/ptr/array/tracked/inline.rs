#[cfg(feature = "sync-rt")]
mod tests {
    use crate::util::JULIA;
    use jlrs::{prelude::*, call::Args};
    
    #[test]
    fn args_test() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|_, mut frame| {
                let v1 = Value::new(&mut frame, 0usize);

                frame.scope(|mut frame| {
                    let v2 = Value::new(&mut frame, 0usize);
                    let args = Args::zero().arg(v1)?.arg(v2)?.arg(v1)?.arg(v2);
                    let args2 = Args::zero().arg(v2)?.arg(v1)?.arg(v1);

                    assert!(args.is_ok());
                    assert!(args2.is_ok());
                    
                    Ok(())
                })?;

                Ok(())
            })
            .unwrap();
        });
    }
}