mod util;

#[cfg(test)]
#[cfg(feature = "sync-rt")]
mod tests {
    use super::util::JULIA;
    use half::f16;
    use jlrs::prelude::*;

    #[test]
    fn one_minus_one_equals_zero() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let one = Value::new(&mut frame, f16::ONE);
                    let func = Module::base(&frame).function(&mut frame, "-")?;
                    let res = func
                        .call2(&mut frame, one, one)
                        .into_jlrs_result()?
                        .unbox::<f16>()?;

                    assert_eq!(res, f16::ZERO);
                    Ok(())
                })
                .unwrap();
        });
    }
}
