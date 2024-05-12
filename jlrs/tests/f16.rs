mod util;

#[cfg(test)]
#[cfg(all(feature = "local-rt", feature = "f16"))]
mod tests {
    use half::f16;
    use jlrs::prelude::*;

    use super::util::JULIA;

    #[test]
    fn one_minus_one_equals_zero() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();
            let mut frame = StackFrame::new();

            julia
                .instance(&mut frame)
                .returning::<JlrsResult<_>>()
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
