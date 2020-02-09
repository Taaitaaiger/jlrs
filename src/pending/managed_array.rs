use crate::context::AllocationContext;
use crate::dimensions::Dimensions;
use crate::error::JlrsResult;
use crate::traits::{Allocate, Call, JuliaType, ValidHandle};
use jl_sys::{
    jl_alloc_array_1d, jl_alloc_array_2d, jl_alloc_array_3d, jl_apply_array_type, jl_new_array,
    jl_value_t,
};

pub(crate) struct ManagedArray {
    datatype: *mut jl_value_t,
    dims: Dimensions,
}

impl ManagedArray {
    pub(crate) unsafe fn new<T: JuliaType, D: Into<Dimensions>>(dims: D) -> Self {
        ManagedArray {
            datatype: T::julia_type(),
            dims: dims.into(),
        }
    }
}

impl Allocate for ManagedArray {
    unsafe fn allocate(&self, mut context: AllocationContext) -> JlrsResult<*mut jl_value_t> {
        let array_type = jl_apply_array_type(self.datatype, self.dims.n_dimensions() as _);
        let array = match self.dims.n_dimensions() {
            1 => jl_alloc_array_1d(array_type, self.dims.n_elements(0) as _),
            2 => jl_alloc_array_2d(
                array_type,
                self.dims.n_elements(0) as _,
                self.dims.n_elements(1) as _,
            ),
            3 => jl_alloc_array_3d(
                array_type,
                self.dims.n_elements(0) as _,
                self.dims.n_elements(1) as _,
                self.dims.n_elements(2) as _,
            ),
            _ => {
                let dims_handle = context.new_primitives(self.dims.unwrap_many())?;
                let out = context.new_unassigned()?;
                let dims = context.execute(|exec_ctx| {
                    let module = exec_ctx.main_module().submodule("Jlrs")?;
                    let func = module.function("arraydims")?;
                    let out = func.call_primitives(exec_ctx, out, dims_handle)?;

                    Ok(out.get_value(exec_ctx))
                })?;

                jl_new_array(array_type, dims)
            }
        };

        Ok(array as _)
    }
}
