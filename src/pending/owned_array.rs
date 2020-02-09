use crate::context::AllocationContext;
use crate::dimensions::Dimensions;
use crate::error::JlrsResult;
use crate::traits::{Allocate, Call, JuliaType, ValidHandle};
use jl_sys::{jl_apply_array_type, jl_ptr_to_array, jl_ptr_to_array_1d, jl_value_t};
use std::ffi::c_void;
use std::mem::ManuallyDrop;

pub(crate) struct OwnedArray {
    datatype: *mut jl_value_t,
    data: *mut c_void,
    dims: Dimensions,
}

impl OwnedArray {
    pub(crate) unsafe fn new<T: JuliaType>(data: Vec<T>, dims: Dimensions) -> Self {
        let mut data = ManuallyDrop::new(data);

        OwnedArray {
            datatype: T::julia_type(),
            data: data.as_mut_ptr() as _,
            dims: dims,
        }
    }
}

impl Allocate for OwnedArray {
    unsafe fn allocate(&self, mut context: AllocationContext) -> JlrsResult<*mut jl_value_t> {
        let array_type = jl_apply_array_type(self.datatype, self.dims.n_dimensions() as _);
        let array = match self.dims.n_dimensions() {
            1 => jl_ptr_to_array_1d(array_type, self.data, self.dims.n_elements(0) as _, 1),
            2 => {
                let dims_handle =
                    context.new_primitives([self.dims.n_elements(0), self.dims.n_elements(1)])?;
                let out = context.new_unassigned()?;
                let dims = context.execute(|exec_ctx| {
                    let module = exec_ctx.main_module().submodule("Jlrs")?;
                    let func = module.function("arraydims")?;
                    let out = func.call_primitives(exec_ctx, out, dims_handle)?;

                    Ok(out.get_value(exec_ctx))
                })?;

                jl_ptr_to_array(array_type, self.data, dims, 1)
            }
            3 => {
                let dims_handle = context.new_primitives([
                    self.dims.n_elements(0),
                    self.dims.n_elements(1),
                    self.dims.n_elements(2),
                ])?;
                let out = context.new_unassigned()?;
                let dims = context.execute(|exec_ctx| {
                    let module = exec_ctx.main_module().submodule("Jlrs")?;
                    let func = module.function("arraydims")?;
                    let out = func.call_primitives(exec_ctx, out, dims_handle)?;

                    Ok(out.get_value(exec_ctx))
                })?;

                jl_ptr_to_array(array_type, self.data, dims, 1)
            }
            _ => {
                let dims_handle = context.new_primitives(self.dims.unwrap_many())?;
                let out = context.new_unassigned()?;
                let dims = context.execute(|exec_ctx| {
                    let module = exec_ctx.main_module().submodule("Jlrs")?;
                    let func = module.function("arraydims")?;
                    let out = func.call_primitives(exec_ctx, out, dims_handle)?;

                    Ok(out.get_value(exec_ctx))
                })?;

                jl_ptr_to_array(array_type, self.data, dims, 1)
            }
        };

        Ok(array as _)
    }
}
