//! Copy n-dimensional arrays from Julia to Rust.

use crate::dimensions::Dimensions;
use crate::error::{JlrsError, JlrsResult};
use crate::traits::{TryUnbox, Unboxable};
use jl_sys::{
    jl_array_data, jl_array_dim, jl_array_dims, jl_array_eltype, jl_array_ndims, jl_array_nrows,
    jl_is_array, jl_value_t,
};

/// An n-dimensional array whose contents have been copied from Julia. You can create this struct
/// by calling [`ExecutionContext::try_unbox`]. In order to unbox arrays that contain `bool`s or
/// `char`s, you can unbox them as `UnboxedArray<i8>` and `UnboxedArray<u32>` respectively.
///
/// [`ExecutionContext:try_unbox`]: ../context/struct.ExecutionContext.html#method.try_unbox
pub struct UnboxedArray<T> {
    data: Vec<T>,
    dimensions: Dimensions,
}

impl<T> UnboxedArray<T> {
    /// Turn the unboxed array into a tuple containing its data with a column-major layout and its
    /// dimensions.
    pub fn splat(self) -> (Vec<T>, Dimensions) {
        (self.data, self.dimensions)
    }
}

impl<T: Unboxable> TryUnbox for UnboxedArray<T> {
    fn try_unbox(value: *mut jl_value_t) -> JlrsResult<Self> {
        unsafe {
            if !jl_is_array(value) {
                return Err(JlrsError::NotAnArray.into());
            }

            if jl_array_eltype(value) as *mut jl_value_t != T::julia_type() {
                return Err(JlrsError::WrongType.into());
            }

            let jl_data = jl_array_data(value) as *const T;
            let n_dims = jl_array_ndims(value as _);
            let dimensions: Dimensions = match n_dims {
                0 => return Err(JlrsError::ZeroDimension.into()),
                1 => Into::into(jl_array_nrows(value as _) as u64),
                2 => Into::into((
                    jl_array_dim(value as _, 0) as _,
                    jl_array_dim(value as _, 1) as _,
                )),
                3 => Into::into((
                    jl_array_dim(value as _, 0) as _,
                    jl_array_dim(value as _, 1) as _,
                    jl_array_dim(value as _, 2) as _,
                )),
                ndims => Into::into(jl_array_dims(value as _, ndims as _)),
            };

            let sz = dimensions.size();
            let mut data = Vec::with_capacity(sz as _);
            let ptr = data.as_mut_ptr();
            std::ptr::copy_nonoverlapping(jl_data, ptr, sz as _);
            data.set_len(sz as _);
            Ok(UnboxedArray { data, dimensions })
        }
    }
}
