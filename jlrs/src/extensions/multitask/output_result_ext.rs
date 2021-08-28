use crate::memory::output::OutputResult;
use jl_sys::jl_value_t;
use std::ptr::NonNull;

pub(super) trait OutputResultExt {
    fn unwrap_non_null(self) -> NonNull<jl_value_t>;
}

impl OutputResultExt for OutputResult<'_, '_, '_> {
    fn unwrap_non_null(self) -> NonNull<jl_value_t> {
        match self {
            Self::Ok(pov) => pov.unwrap_non_null(),
            Self::Err(pov) => pov.unwrap_non_null(),
        }
    }
}
