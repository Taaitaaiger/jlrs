use jlrs::{error::JlrsResult, prelude::Bool};

pub fn returns_jlrs_result(throw_err: Bool) -> JlrsResult<i32> {
    if throw_err.as_bool() {
        Err(jlrs::error::JlrsError::exception("Error"))?
    } else {
        Ok(3)
    }
}
