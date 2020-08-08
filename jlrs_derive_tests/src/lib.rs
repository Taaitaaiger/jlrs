mod util;
mod impls;

#[cfg(test)]
mod tests {
    use super::util::JULIA;
    use super::impls::*;
    use jlrs::prelude::*;

    #[test]
    fn derive_bits_type_bool() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .dynamic_frame(|_global, frame| {
                    let s = BitsTypeBool{ a: true };
                    let v = Value::new(frame, s).unwrap();
                    let first = v.get_nth_field(frame, 0).unwrap();

                    assert_eq!(first.cast::<bool>().unwrap(), true);
                    assert!(v.is::<BitsTypeBool>());
                    assert_eq!(v.cast::<BitsTypeBool>().unwrap(), s);

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_bits_type_char() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .dynamic_frame(|_global, frame| {
                    let s = BitsTypeChar{ a: 'b' };
                    let v = Value::new(frame, s).unwrap();
                    let first = v.get_nth_field(frame, 0).unwrap();

                    assert_eq!(first.cast::<char>().unwrap(), 'b');
                    assert!(v.is::<BitsTypeChar>());
                    assert_eq!(v.cast::<BitsTypeChar>().unwrap(), s);

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_bits_type_uint8() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .dynamic_frame(|_global, frame| {
                    let s = BitsTypeUInt8{ a: 1 };
                    let v = Value::new(frame, s).unwrap();
                    let first = v.get_nth_field(frame, 0).unwrap();

                    assert_eq!(first.cast::<u8>().unwrap(), 1);
                    assert!(v.is::<BitsTypeUInt8>());
                    assert_eq!(v.cast::<BitsTypeUInt8>().unwrap(), s);

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_bits_type_uint16() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .dynamic_frame(|_global, frame| {
                    let s = BitsTypeUInt16{ a: 2 };
                    let v = Value::new(frame, s).unwrap();
                    let first = v.get_nth_field(frame, 0).unwrap();

                    assert_eq!(first.cast::<u16>().unwrap(), 2);
                    assert!(v.is::<BitsTypeUInt16>());
                    assert_eq!(v.cast::<BitsTypeUInt16>().unwrap(), s);

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_bits_type_uint32() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .dynamic_frame(|_global, frame| {
                    let s = BitsTypeUInt32{ a: 3 };
                    let v = Value::new(frame, s).unwrap();
                    let first = v.get_nth_field(frame, 0).unwrap();

                    assert_eq!(first.cast::<u32>().unwrap(), 3);
                    assert!(v.is::<BitsTypeUInt32>());
                    assert_eq!(v.cast::<BitsTypeUInt32>().unwrap(), s);

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_bits_type_uint64() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .dynamic_frame(|_global, frame| {
                    let s = BitsTypeUInt64{ a: 4 };
                    let v = Value::new(frame, s).unwrap();
                    let first = v.get_nth_field(frame, 0).unwrap();

                    assert_eq!(first.cast::<u64>().unwrap(), 4);
                    assert!(v.is::<BitsTypeUInt64>());
                    assert_eq!(v.cast::<BitsTypeUInt64>().unwrap(), s);

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_bits_type_uint() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .dynamic_frame(|_global, frame| {
                    let s = BitsTypeUInt{ a: 5 };
                    let v = Value::new(frame, s).unwrap();
                    let first = v.get_nth_field(frame, 0).unwrap();

                    assert_eq!(first.cast::<u64>().unwrap(), 5);
                    assert!(v.is::<BitsTypeUInt>());
                    assert_eq!(v.cast::<BitsTypeUInt>().unwrap(), s);

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_bits_type_int8() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .dynamic_frame(|_global, frame| {
                    let s = BitsTypeInt8{ a: -1 };
                    let v = Value::new(frame, s).unwrap();
                    let first = v.get_nth_field(frame, 0).unwrap();

                    assert_eq!(first.cast::<i8>().unwrap(), -1);
                    assert!(v.is::<BitsTypeInt8>());
                    assert_eq!(v.cast::<BitsTypeInt8>().unwrap(), s);

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_bits_type_int16() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .dynamic_frame(|_global, frame| {
                    let s = BitsTypeInt16{ a: -2 };
                    let v = Value::new(frame, s).unwrap();
                    let first = v.get_nth_field(frame, 0).unwrap();

                    assert_eq!(first.cast::<i16>().unwrap(), -2);
                    assert!(v.is::<BitsTypeInt16>());
                    assert_eq!(v.cast::<BitsTypeInt16>().unwrap(), s);

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_bits_type_int32() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .dynamic_frame(|_global, frame| {
                    let s = BitsTypeInt32{ a: -3 };
                    let v = Value::new(frame, s).unwrap();
                    let first = v.get_nth_field(frame, 0).unwrap();

                    assert_eq!(first.cast::<i32>().unwrap(), -3);
                    assert!(v.is::<BitsTypeInt32>());
                    assert_eq!(v.cast::<BitsTypeInt32>().unwrap(), s);

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_bits_type_int64() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .dynamic_frame(|_global, frame| {
                    let s = BitsTypeInt64{ a: -4 };
                    let v = Value::new(frame, s).unwrap();
                    let first = v.get_nth_field(frame, 0).unwrap();

                    assert_eq!(first.cast::<i64>().unwrap(), -4);
                    assert!(v.is::<BitsTypeInt64>());
                    assert_eq!(v.cast::<BitsTypeInt64>().unwrap(), s);

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_bits_type_int() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .dynamic_frame(|_global, frame| {
                    let s = BitsTypeInt{ a: -5 };
                    let v = Value::new(frame, s).unwrap();
                    let first = v.get_nth_field(frame, 0).unwrap();

                    assert_eq!(first.cast::<i64>().unwrap(), -5);
                    assert!(v.is::<BitsTypeInt>());
                    assert_eq!(v.cast::<BitsTypeInt>().unwrap(), s);

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_bits_type_float32() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .dynamic_frame(|_global, frame| {
                    let s = BitsTypeFloat32{ a: 1.2 };
                    let v = Value::new(frame, s).unwrap();
                    let first = v.get_nth_field(frame, 0).unwrap();

                    assert_eq!(first.cast::<f32>().unwrap(), 1.2);
                    assert!(v.is::<BitsTypeFloat32>());
                    assert_eq!(v.cast::<BitsTypeFloat32>().unwrap(), s);

                    Ok(())
                })
                .unwrap()
        })
    }

    #[test]
    fn derive_bits_type_float64() {
        JULIA.with(|j| {
            let mut julia = j.borrow_mut();

            julia
                .dynamic_frame(|_global, frame| {
                    let s = BitsTypeFloat64{ a: -2.3 };
                    let v = Value::new(frame, s).unwrap();
                    let first = v.get_nth_field(frame, 0).unwrap();

                    assert_eq!(first.cast::<f64>().unwrap(), -2.3);
                    assert!(v.is::<BitsTypeFloat64>());
                    assert_eq!(v.cast::<BitsTypeFloat64>().unwrap(), s);

                    Ok(())
                })
                .unwrap()
        })
    }
}
