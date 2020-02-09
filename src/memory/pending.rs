use crate::context::AllocationContext;
use crate::dimensions::Dimensions;
use crate::error::{JlrsError, JlrsResult};
use crate::pending::{
    borrowed_array::BorrowedArray, managed_array::ManagedArray, owned_array::OwnedArray,
    primitive::Primitive,
};
use crate::traits::{Allocate, IntoPrimitive, JuliaType};
use jl_sys::jl_value_t;

pub(crate) enum PendingItem {
    ManagedArray(usize, ManagedArray),
    OwnedArray(usize, OwnedArray),
    BorrowedArray(usize, BorrowedArray),
    Primitive(usize, Primitive),
    String(usize, String),
}

impl Allocate for PendingItem {
    unsafe fn allocate(&self, context: AllocationContext) -> JlrsResult<*mut jl_value_t> {
        match self {
            PendingItem::ManagedArray(_, ref array) => array.allocate(context),
            PendingItem::OwnedArray(_, ref owned) => owned.allocate(context),
            PendingItem::BorrowedArray(_, ref borrowed) => borrowed.allocate(context),
            PendingItem::Primitive(_, ref primitive) => primitive.allocate(context),
            PendingItem::String(_, ref string) => string.allocate(context),
        }
    }
}

impl PendingItem {
    pub(crate) fn index(&self) -> usize {
        match self {
            PendingItem::ManagedArray(ref idx, _) => *idx,
            PendingItem::OwnedArray(ref idx, _) => *idx,
            PendingItem::BorrowedArray(ref idx, _) => *idx,
            PendingItem::Primitive(ref idx, _) => *idx,
            PendingItem::String(ref idx, _) => *idx,
        }
    }
}

pub(crate) struct Pending {
    pending: Vec<PendingItem>,
    n: usize,
    capacity: usize,
    frame_start: usize,
}

impl Pending {
    pub(crate) fn new(capacity: usize, frame_start: usize) -> Self {
        Pending {
            pending: Vec::new(),
            n: 0,
            capacity,
            frame_start,
        }
    }

    pub(crate) fn clear(&mut self) {
        self.pending.clear();
        self.n = 0;
    }

    pub(crate) unsafe fn new_frame(&mut self, capacity: usize, frame_start: usize) {
        self.n = 0;
        self.capacity = capacity;
        self.frame_start = frame_start;
    }

    pub(crate) fn take_pending(&mut self) -> Vec<PendingItem> {
        self.pending.drain(0..).collect()
    }

    pub(crate) fn slots(&self) -> usize {
        self.n
    }

    pub(crate) fn new_unassigned(&mut self) -> JlrsResult<usize> {
        if self.n == self.capacity {
            return Err(JlrsError::StackSizeExceeded.into());
        }

        self.n += 1;
        Ok(self.frame_start + self.n - 1)
    }

    pub(crate) fn new_primitive<T: IntoPrimitive>(&mut self, value: T) -> JlrsResult<usize> {
        if self.n == self.capacity {
            return Err(JlrsError::StackSizeExceeded.into());
        }

        self.n += 1;
        self.pending.push(PendingItem::Primitive(
            self.n + self.frame_start - 1,
            value.into_primitive(),
        ));

        Ok(self.frame_start + self.n - 1)
    }

    pub(crate) fn new_primitives<T: IntoPrimitive, P: AsRef<[T]>>(
        &mut self,
        values: P,
    ) -> JlrsResult<(usize, usize)> {
        let values = values.as_ref();
        if self.n + values.len() >= self.capacity + 1 {
            return Err(JlrsError::StackSizeExceeded.into());
        }

        let n = self.n;
        for v in values {
            self.n += 1;
            self.pending.push(PendingItem::Primitive(
                self.n + self.frame_start - 1,
                v.into_primitive(),
            ));
        }

        Ok((self.frame_start + n, values.len()))
    }

    pub(crate) fn new_primitives_dyn<'input, P: AsRef<[&'input dyn IntoPrimitive]>>(
        &mut self,
        values: P,
    ) -> JlrsResult<(usize, usize)> {
        let values = values.as_ref();
        if self.n + values.len() >= self.capacity + 1 {
            return Err(JlrsError::StackSizeExceeded.into());
        }
        let n = self.n;
        for v in values {
            self.n += 1;
            self.pending.push(PendingItem::Primitive(
                self.n + self.frame_start - 1,
                v.into_primitive(),
            ));
        }
        Ok((self.frame_start + n, values.len()))
    }

    pub(crate) fn new_managed_array<T: JuliaType>(
        &mut self,
        dims: Dimensions,
    ) -> JlrsResult<usize> {
        if self.n == self.capacity {
            return Err(JlrsError::StackSizeExceeded.into());
        }

        self.n += 1;
        let array = unsafe { ManagedArray::new::<T, _>(dims) };
        self.pending.push(PendingItem::ManagedArray(
            self.n + self.frame_start - 1,
            array,
        ));

        Ok(self.frame_start + self.n - 1)
    }

    pub(crate) fn new_owned_array<T: JuliaType>(
        &mut self,
        data: Vec<T>,
        dims: Dimensions,
    ) -> JlrsResult<usize> {
        if self.n == self.capacity {
            return Err(JlrsError::StackSizeExceeded.into());
        }

        self.n += 1;
        let array = unsafe { OwnedArray::new(data, dims) };
        self.pending.push(PendingItem::OwnedArray(
            self.n + self.frame_start - 1,
            array,
        ));

        Ok(self.frame_start + self.n - 1)
    }

    pub(crate) unsafe fn new_borrowed_array<T: JuliaType, U: AsMut<[T]>>(
        &mut self,
        data: U,
        dims: Dimensions,
    ) -> JlrsResult<usize> {
        if self.n == self.capacity {
            return Err(JlrsError::StackSizeExceeded.into());
        }

        self.n += 1;
        let array = BorrowedArray::new(data, dims);
        self.pending.push(PendingItem::BorrowedArray(
            self.n + self.frame_start - 1,
            array,
        ));

        Ok(self.frame_start + self.n - 1)
    }

    pub(crate) fn new_string(&mut self, string: String) -> JlrsResult<usize> {
        if self.n == self.capacity {
            return Err(JlrsError::StackSizeExceeded.into());
        }

        self.n += 1;
        self.pending
            .push(PendingItem::String(self.n + self.frame_start - 1, string));

        Ok(self.frame_start + self.n - 1)
    }
}
