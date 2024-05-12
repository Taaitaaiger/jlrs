#[cfg(feature = "local-rt")]
pub(crate) mod tests {
    use jlrs::{data::managed::array::VectorAny, prelude::*};

    use crate::util::JULIA;

    fn vector_any_new_any(julia: &mut Julia) {
        julia
            .returning::<JlrsResult<_>>()
            .scope(|mut frame| {
                let arr = VectorAny::new_any(&mut frame, 3);
                assert!(arr.is_ok());
                Ok(())
            })
            .unwrap();
    }

    pub(crate) fn vector_any_tests() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            let mut inst = jlrs.instance(&mut frame);
            vector_any_new_any(&mut inst);
        });
    }
}
