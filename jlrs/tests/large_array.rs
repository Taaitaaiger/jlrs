mod util;
#[cfg(feature = "sync-rt")]
#[cfg(not(all(target_os = "windows", feature = "lts")))]
mod tests {
    use super::util::JULIA;
    use jlrs::prelude::*;

    #[test]
    fn create_large_array() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|_, mut frame| {
                let array =
                    Array::new::<f32, _, _, _>(&mut frame, &[1, 1, 1, 1, 1, 1, 1, 1, 1][..]);
                assert!(array.is_ok());
                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn move_large_array() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|_, mut frame| {
                let array =
                    Array::from_vec(&mut frame, vec![1u64], &[1, 1, 1, 1, 1, 1, 1, 1, 1][..]);
                assert!(array.is_ok());
                Ok(())
            })
            .unwrap();
        });
    }

    #[test]
    fn borrow_large_array() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|_, mut frame| {
                let mut data = vec![1u32];
                let array = {
                    Array::from_slice(&mut frame, &mut data, &[1, 1, 1, 1, 1, 1, 1, 1, 1][..])?
                        .into_jlrs_result()
                };
                assert!(array.is_ok());
                Ok(())
            })
            .unwrap();
        });
    }
}
