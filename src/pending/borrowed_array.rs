use crate::context::AllocationContext;
use crate::dimensions::Dimensions;
use crate::error::JlrsResult;
use crate::traits::{Allocate, Call, JuliaType, ValidHandle};
use jl_sys::{jl_apply_array_type, jl_ptr_to_array, jl_ptr_to_array_1d, jl_value_t};
use std::ffi::c_void;

pub(crate) struct BorrowedArray {
    datatype: *mut jl_value_t,
    data: *mut c_void,
    dims: Dimensions,
}

impl BorrowedArray {
    pub(crate) unsafe fn new<T: JuliaType, U: AsMut<[T]>>(mut data: U, dims: Dimensions) -> Self {
        let data_ptr = data.as_mut().as_mut_ptr() as *mut c_void;
        BorrowedArray {
            datatype: T::julia_type(),
            data: data_ptr,
            dims: dims,
        }
    }
}

impl Allocate for BorrowedArray {
    unsafe fn allocate(&self, mut context: AllocationContext) -> JlrsResult<*mut jl_value_t> {
        let array_type = jl_apply_array_type(self.datatype, self.dims.n_dimensions() as _);
        match self.dims.n_dimensions() {
            1 => {
                Ok(jl_ptr_to_array_1d(array_type, self.data, self.dims.n_elements(0) as _, 0) as _)
            }
            2 => {
                let dims_handle =
                    context.new_primitives([self.dims.n_elements(0), self.dims.n_elements(1)])?;
                let out = context.new_unassigned()?;
                context.execute(|exec_ctx| {
                    let module = exec_ctx.main_module().submodule("Jlrs")?;
                    let func = module.function("arraydims")?;
                    let out = func.call_primitives(exec_ctx, out, dims_handle)?;

                    let dims = out.get_value(exec_ctx);
                    Ok(jl_ptr_to_array(array_type, self.data, dims, 0) as _)
                })
            }
            3 => {
                let dims_handle = context.new_primitives([
                    self.dims.n_elements(0),
                    self.dims.n_elements(1),
                    self.dims.n_elements(2),
                ])?;
                let out = context.new_unassigned()?;
                context.execute(|exec_ctx| {
                    let module = exec_ctx.main_module().submodule("Jlrs")?;
                    let func = module.function("arraydims")?;
                    let out = func.call_primitives(exec_ctx, out, dims_handle)?;

                    let dims = out.get_value(exec_ctx);
                    Ok(jl_ptr_to_array(array_type, self.data, dims, 0) as _)
                })
            }
            _ => {
                let dims_handle = context.new_primitives(self.dims.unwrap_many())?;
                let out = context.new_unassigned()?;
                context.execute(|exec_ctx| {
                    let module = exec_ctx.main_module().submodule("Jlrs")?;
                    let func = module.function("arraydims")?;
                    let out = func.call_primitives(exec_ctx, out, dims_handle)?;

                    let dims = out.get_value(exec_ctx);
                    Ok(jl_ptr_to_array(array_type, self.data, dims, 0) as _)
                })
            }
        }
    }
}
