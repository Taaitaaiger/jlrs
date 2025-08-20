mod util;

#[cfg(test)]
#[cfg(all(feature = "local-rt", feature = "f16"))]
mod tests {
    use half::f16;
    use jlrs::prelude::*;

    use super::util::JULIA;

    #[test]
    fn one_minus_one_equals_zero() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let one = Value::new(&mut frame, f16::ONE);
                    let func = Module::base(&frame).global(&mut frame, "-").unwrap();
                    let res = func
                        .call(&mut frame, [one, one])
                        .unwrap()
                        .unbox::<f16>()
                        .unwrap();

                    assert_eq!(res, f16::ZERO);
                })
            });
        });
    }
}
