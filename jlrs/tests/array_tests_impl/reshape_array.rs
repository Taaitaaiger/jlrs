#[cfg(any(
    feature = "julia-1-6",
    feature = "julia-1-7",
    feature = "julia-1-8",
    feature = "julia-1-9",
    feature = "julia-1-10"
))]
#[cfg(feature = "local-rt")]
pub(crate) mod tests {
    use jlrs::{
        data::managed::array::{data::accessor::Accessor, TypedArray},
        prelude::*,
    };

    use crate::util::JULIA;

    fn array_can_be_reshaped() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let arr_val = TypedArray::<f32>::new(&mut frame, (1, 2)).unwrap();
                    arr_val
                        .indeterminate_data()
                        .reshape_ranked::<_, _, 3>(&mut frame, (1, 1, 2))
                        .unwrap()
                        .unwrap();
                    Ok(())
                })
                .unwrap();
        });
    }

    pub(crate) fn reshape_array_tests() {
        array_can_be_reshaped();
    }
}
