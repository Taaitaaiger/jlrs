#[cfg(feature = "local-rt")]
pub(crate) mod tests {
    use jlrs::{
        data::managed::array::VectorAny, prelude::*, runtime::handle::with_stack::StackHandle,
    };

    use crate::util::JULIA;

    fn vector_any_new_any(julia: &mut StackHandle) {
        julia.scope(|mut frame| {
            let arr = VectorAny::new_any(&mut frame, 3);
            assert!(arr.is_ok());
        })
    }

    pub(crate) fn vector_any_tests() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                vector_any_new_any(&mut stack);
            });
        });
    }
}
