use jlrs::layout::valid_layout::ValidLayout;
use jlrs::prelude::*;
use jlrs::util::JULIA;

macro_rules! impl_valid_layout_test {
    ($name:ident, $t:ty, $v:expr) => {
        #[test]
        fn $name() {
            JULIA.with(|j| {
                let mut jlrs = j.borrow_mut();
                jlrs.scope_with_slots(1, |_global, frame| {
                    let i = $v;
                    let v = Value::new(frame, i)?;
                    assert!(<$t>::valid_layout(v.datatype().as_value()));
                    Ok(())
                })
                .unwrap();
            })
        }
    };
}

impl_valid_layout_test!(valid_layout_u8, u8, 1u8);
impl_valid_layout_test!(valid_layout_u16, u16, 1u16);
impl_valid_layout_test!(valid_layout_u32, u32, 1u32);
impl_valid_layout_test!(valid_layout_u64, u64, 1u64);
impl_valid_layout_test!(valid_layout_usize, usize, 1usize);
impl_valid_layout_test!(valid_layout_i8, i8, 1i8);
impl_valid_layout_test!(valid_layout_i16, i16, 1i16);
impl_valid_layout_test!(valid_layout_i32, i32, 1i32);
impl_valid_layout_test!(valid_layout_i64, i64, 1i64);
impl_valid_layout_test!(valid_layout_isize, isize, 1isize);
impl_valid_layout_test!(valid_layout_f32, f32, 1.0f32);
impl_valid_layout_test!(valid_layout_f64, f64, 1.0f64);
impl_valid_layout_test!(valid_layout_bool, bool, true);
impl_valid_layout_test!(valid_layout_char, char, 'a');

#[test]
fn valid_layout_array() {
    JULIA.with(|j| {
        let mut jlrs = j.borrow_mut();
        jlrs.scope(|global, frame| {
            unsafe {
                let v = Array::new::<i32, _, _, _>(frame, (2, 2))?.into_jlrs_result()?;
                assert!(Array::valid_layout(v.datatype().as_value()));

                let ua = Module::base(global)
                    .global_ref("Array")?
                    .wrapper_unchecked();

                assert!(Array::valid_layout(ua));
            }
            Ok(())
        })
        .unwrap();
    })
}
