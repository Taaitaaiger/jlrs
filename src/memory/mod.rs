mod pending;
mod stack;

use crate::context::AllocationContext;
use crate::context::{MemWrap, Scope};
use crate::dimensions::Dimensions;
use crate::error::JlrsResult;
use crate::handles::{
    AssignedHandle, BorrowedArrayHandle, PrimitiveHandles, UnassignedHandle, UninitArrayHandle,
};
use crate::traits::{Allocate, IntoPrimitive, JuliaType};
use jl_sys::jl_value_t;
use pending::Pending;
use stack::Stack;

pub(crate) struct Memory {
    stack: Stack,
    pending: Pending,
}

impl Memory {
    pub(crate) fn new(stack_size: usize) -> Self {
        let stack = Stack::new(stack_size);
        let pending = Pending::new(stack.free_slots(), stack.current_offset());

        Memory { stack, pending }
    }

    pub(crate) fn stack_size(&self) -> usize {
        self.stack.size()
    }

    pub(crate) fn new_unassigned<'scope>(&mut self) -> JlrsResult<UnassignedHandle<'scope>> {
        let index = self.pending.new_unassigned()?;
        Ok(unsafe { UnassignedHandle::new(index) })
    }

    pub(crate) fn new_primitive<'scope, T: IntoPrimitive>(
        &mut self,
        value: T,
    ) -> JlrsResult<AssignedHandle<'scope>> {
        let index = self.pending.new_primitive(value)?;
        Ok(unsafe { AssignedHandle::new(index) })
    }

    pub(crate) fn new_primitives<'scope, T: IntoPrimitive, P: AsRef<[T]>>(
        &mut self,
        values: P,
    ) -> JlrsResult<PrimitiveHandles<'scope>> {
        let index = self.pending.new_primitives(values)?;
        Ok(unsafe { PrimitiveHandles::new(index.0, index.1) })
    }

    pub(crate) fn new_primitives_dyn<'scope, 'input, P: AsRef<[&'input dyn IntoPrimitive]>>(
        &mut self,
        values: P,
    ) -> JlrsResult<PrimitiveHandles<'scope>> {
        let index = self.pending.new_primitives_dyn(values)?;
        Ok(unsafe { PrimitiveHandles::new(index.0, index.1) })
    }

    pub(crate) fn new_managed_array<'scope, T: JuliaType + Copy>(
        &mut self,
        dims: Dimensions,
    ) -> JlrsResult<UninitArrayHandle<'scope, T>> {
        let index = self.pending.new_managed_array::<T>(dims.clone())?;
        Ok(unsafe { UninitArrayHandle::new(index, dims) })
    }

    pub(crate) fn new_owned_array<'scope, T: JuliaType>(
        &mut self,
        data: Vec<T>,
        dims: Dimensions,
    ) -> JlrsResult<AssignedHandle<'scope>> {
        let index = self.pending.new_owned_array(data, dims)?;
        Ok(unsafe { AssignedHandle::new(index) })
    }

    pub(crate) unsafe fn new_borrowed_array<'session, 'borrow, T: JuliaType, U: AsMut<[T]>>(
        &mut self,
        data: &'borrow mut U,
        dims: Dimensions,
    ) -> JlrsResult<BorrowedArrayHandle<'session, 'borrow>> {
        let index = self.pending.new_borrowed_array(data, dims)?;
        Ok(BorrowedArrayHandle::new(index))
    }

    pub(crate) fn new_string<'scope>(
        &mut self,
        string: String,
    ) -> JlrsResult<AssignedHandle<'scope>> {
        let index = self.pending.new_string(string)?;
        Ok(unsafe { AssignedHandle::new(index) })
    }

    pub(crate) unsafe fn get_value(&self, index: usize) -> *mut jl_value_t {
        self.stack.get_value(index) as _
    }

    pub(crate) unsafe fn get_values(&self, index: usize, n: usize) -> *mut *mut jl_value_t {
        self.stack.get_values(index, n) as _
    }

    pub(crate) unsafe fn assign(&mut self, index: usize, value: *mut jl_value_t) {
        self.stack.set_value(index, value as _)
    }

    pub(crate) unsafe fn push_nonempty_frame(&mut self) -> JlrsResult<()> {
        // Push the frame first so everything is protected
        if self.pending.slots() == 0 {
            return Ok(());
        }

        self.stack.push_frame(self.pending.slots())?;
        self.pending
            .new_frame(self.stack.free_slots(), self.stack.current_offset());

        let pending = self.pending.take_pending();

        for item in pending {
            let val = {
                let scope = Scope;
                let mut mw = MemWrap::new(self);
                let ctx = AllocationContext::new(&mut mw, &scope);
                item.allocate(ctx)?
            };
            self.stack.set_value(item.index(), val as _);
        }

        Ok(())
    }

    pub(crate) unsafe fn push_frame(&mut self) -> JlrsResult<()> {
        // Push the frame first so everything is protected
        self.stack.push_frame(self.pending.slots())?;
        self.pending
            .new_frame(self.stack.free_slots(), self.stack.current_offset());

        let pending = self.pending.take_pending();

        for item in pending {
            let val = {
                let scope = Scope;
                let mut mw = MemWrap::new(self);
                let ctx = AllocationContext::new(&mut mw, &scope);
                item.allocate(ctx)?
            };
            self.stack.set_value(item.index(), val as _);
        }

        Ok(())
    }

    pub(crate) unsafe fn clear_pending(&mut self) {
        self.pending.clear();
    }

    pub(crate) unsafe fn pop_frame(&mut self) {
        self.stack.pop_frame();
        self.pending
            .new_frame(self.stack.free_slots(), self.stack.current_offset());
    }

    pub(crate) unsafe fn pop_all_frames(&mut self) {
        self.stack.pop_all();
        self.pending
            .new_frame(self.stack.free_slots(), self.stack.current_offset());
    }
}
