//! Handles to data created by or for Julia.
//!
//! Julia is a garbage collected language; in order to prevent its garbage collector from freeing
//! data that's in use outside of Julia, users of the C API must properly manage the garbage
//! collector. Taking care of this is the major responsibility of [`jlrs`]. As a result, rather
//! than having to deal with raw pointers from the C API and the garbage collector directly, you
//! will interact with Julia mostly through the handles defined in this module. The lifetimes here
//! are used to enforce handles can only be used when the data they refer to is protected from
//! garbage collection.
//!
//! [`jlrs`]: ../index.html

use crate::context::{ExecutionContext, Scope};
use crate::dimensions::Dimensions;
use crate::error::{JlrsError, JlrsResult};
use crate::traits::{Call, JuliaType, UnboxableHandle, ValidHandle};
use jl_sys::{jl_array_data, jl_value_t};
use std::marker::PhantomData;
use std::ptr::null_mut;

/// Handle to data that has not been assigned yet. You will need to use one every time you call a
/// Julia function from Rust. You can create this handle with [`Session::new_unassigned`] and
/// [`AllocationContext::new_unassigned`].
///
/// [`Session::new_unassigned`]: ../context/struct.Session.html#method.new_unassigned
/// [`AllocationContext::new_unassigned`]: ../context/struct.AllocationContext.html#method.new_unassigned
#[must_use]
pub struct UnassignedHandle<'scope>(pub(crate) usize, PhantomData<&'scope Scope>);

impl<'scope> UnassignedHandle<'scope> {
    pub(crate) unsafe fn new(index: usize) -> Self {
        UnassignedHandle(index, PhantomData)
    }

    pub(crate) unsafe fn assign(
        self,
        context: &mut ExecutionContext,
        value: *mut jl_value_t,
    ) -> AssignedHandle<'scope> {
        context.assign(self.0, value);
        AssignedHandle(self.0, PhantomData)
    }
}

/// Handle to Julia data that has been assigned, either due to allocation or as a result of a
/// function call.
///
/// Many functions that allocate data for use in Julia return this handle, they're also the result
/// of a successful function call. This handle can be used as a argument when calling Julia
/// functions, called as a Julia function itself, and its contents can be copied to Rust with
/// [`ExecutionContext::try_unbox`] in supported cases.
///
/// [`ExecutionContext::try_unbox`]: ../context/struct.ExecutionContext.html#method.try_unbox
#[derive(Copy, Clone)]
pub struct AssignedHandle<'scope>(usize, PhantomData<&'scope Scope>);

impl<'scope> AssignedHandle<'scope> {
    pub(crate) unsafe fn new(index: usize) -> Self {
        AssignedHandle(index, PhantomData)
    }

    /// Get a reference to this handle as a trait object.
    #[inline(always)]
    pub fn as_dyn(&'scope self) -> &'scope dyn ValidHandle {
        self as _
    }
}

impl<'scope> ValidHandle for AssignedHandle<'scope> {
    unsafe fn get_value(&self, context: &ExecutionContext) -> *mut jl_value_t {
        context.get_value(self.0)
    }
}

impl<'scope> UnboxableHandle for AssignedHandle<'scope> {}

impl<'scope> Call for AssignedHandle<'scope> {}

/// Handle to several contiguous primitives.
///
/// By allocating several primitives at once with [`Session::new_primitives`],
/// [`Session::new_primitives_dyn`], [`AllocationContext::new_primitives`],
/// or [`AllocationContext::new_primitives_dyn`] you get this handle. It can be used to call a
/// function with [`Call:call_primitives`], in the case of four or more arguments this will have
/// less overhead than using [`Call:call`] to call the function.
///
/// [`Session::new_primitives`]: ../context/struct.Session.html#method.new_primitives
/// [`Session::new_primitives_dyn`]: ../context/struct.Session.html#method.new_primitives_dyn
/// [`AllocationContext::new_primitives`]: ../context/struct.AllocationContext.html#method.new_primitives
/// [`AllocationContext::new_primitives_dyn`]: ../context/struct.AllocationContext.html#method.new_primitives_dyn
/// [`Call:call_primitives`]: ../traits/trait.Call.html#method.call_primitives
/// [`Call:call`]: ../traits/trait.Call.html#method.call
#[derive(Copy, Clone)]
pub struct PrimitiveHandles<'scope>(usize, usize, PhantomData<&'scope Scope>);

impl<'scope> PrimitiveHandles<'scope> {
    pub(crate) unsafe fn new(index: usize, n: usize) -> Self {
        PrimitiveHandles(index, n, PhantomData)
    }

    /// Get the [`AssignedHandle`] at the given index. Panics if index is out of bounds.
    ///
    /// [`AssignedHandle`]: struct.AssignedHandle.html
    pub fn get(&self, index: usize) -> AssignedHandle<'scope> {
        assert!(index < self.len());
        unsafe { AssignedHandle::new(self.index() + index) }
    }

    pub(crate) fn index(&self) -> usize {
        self.0
    }

    /// The number of primitives.
    pub fn len(&self) -> usize {
        self.1
    }
}

/// Handle to an uninitialized, managed array.
///
/// You get this handle by creating a new managed array through either
/// [`Session::new_managed_array`] or [`AllocationContext::new_managed_array`]. In both cases, the
/// allocated array will contain uninitialized data. You can either set the contents directly
/// using [`UninitArrayHandle::set_all`] and [`UninitArrayHandle::set_from`], or indirectly by
/// calling some appropriate Julia function. In the latter case, you should call
/// [`UninitArrayHandle::assume_assigned`] afterwards to turn this handle into an
/// [`AssignedHandle`].
///
/// [`Session::new_managed_array`]: ../context/struct.Session.html#method.new_managed_array
/// [`AllocationContext::new_managed_array`]: ../context/struct.AllocationContext.html#method.new_managed_array
/// [`UninitArrayHandle::set_all`]: struct.UninitArrayHandle.html#method.set_all
/// [`UninitArrayHandle::set_from`]: struct.UninitArrayHandle.html#method.set_from
/// [`UninitArrayHandle::assume_assigned`]: struct.UninitArrayHandle.html#method.assume_assigned
/// [`AssignedHandle`]: struct.AssignedHandle.html
pub struct UninitArrayHandle<'scope, T>(
    usize,
    Dimensions,
    PhantomData<&'scope Scope>,
    PhantomData<T>,
);

impl<'scope, T: JuliaType + Copy> UninitArrayHandle<'scope, T> {
    pub(crate) unsafe fn new(index: usize, dims: Dimensions) -> Self {
        UninitArrayHandle(index, dims, PhantomData, PhantomData)
    }

    /// Get a reference to this handle as a trait object.
    pub fn as_dyn(&self) -> &dyn ValidHandle {
        self as _
    }

    /// Assume the data in the array is valid. You can use this if you create the array in Rust but
    /// initialize its contents in Julia.
    pub unsafe fn assume_assigned(self) -> AssignedHandle<'scope> {
        AssignedHandle(self.0, self.2)
    }

    /// Set every element of the array to `value`. Returns an error if the data pointer of the
    /// array is a null pointer.
    pub fn set_all(
        self,
        context: &mut ExecutionContext,
        value: T,
    ) -> JlrsResult<AssignedHandle<'scope>> {
        unsafe {
            let array = context.get_value(self.0);
            let data = jl_array_data(array) as *mut T;

            if data == null_mut() {
                return Err(JlrsError::NullData.into());
            }

            for i in 0..self.1.size() {
                std::ptr::write(data.offset(i as isize), value);
            }

            Ok(AssignedHandle(self.0, self.2))
        }
    }

    /// Set the array contents by copying `values` into it. Note, this data must have column-major
    /// ordering. Returns an error if the number of elements is incorrect or the data pointer of
    /// the array is a null pointer.
    pub fn set_from<U: AsRef<[T]>>(
        self,
        context: &mut ExecutionContext,
        values: U,
    ) -> JlrsResult<AssignedHandle<'scope>> {
        let v = values.as_ref();
        if self.1.size() != v.len() as _ {
            return Err(JlrsError::DifferentNumberOfElements.into());
        }
        unsafe {
            let array = context.get_value(self.0);
            let data = jl_array_data(array) as *mut T;

            if data == null_mut() {
                return Err(JlrsError::NullData.into());
            }

            std::ptr::copy_nonoverlapping(v.as_ptr(), data, v.len());
        }

        Ok(AssignedHandle(self.0, self.2))
    }
}

impl<'scope, T> ValidHandle for UninitArrayHandle<'scope, T> {
    unsafe fn get_value(&self, context: &ExecutionContext) -> *mut jl_value_t {
        context.get_value(self.0)
    }
}

/// Handle to an array that borrows its contents from Rust.
///
/// You get this handle by borrowing an array using either [`Session::borrow_array`] or
/// [`AllocationContext::borrow_array`]. Generally speaking, you need to be careful when using
/// this. In your Julia code you must never
///  - make this array globally available
///  - return this array from a function
/// as this can easily lead to undefined behavior.
///
/// [`Session::borrow_array`]: ../context/struct.Session.html#method.borrow_array
/// [`AllocationContext::borrow_array`]: ../context/struct.AllocationContext.html#method.borrow_array
#[derive(Copy, Clone)]
pub struct BorrowedArrayHandle<'scope, 'borrow>(
    usize,
    PhantomData<&'scope Scope>,
    PhantomData<&'borrow ()>,
);

impl<'scope, 'borrow> BorrowedArrayHandle<'scope, 'borrow> {
    pub(crate) unsafe fn new(index: usize) -> Self {
        BorrowedArrayHandle(index, PhantomData, PhantomData)
    }

    /// Get a reference to this handle as a trait object.
    pub fn as_dyn(&self) -> &dyn ValidHandle {
        self as _
    }
}

impl<'scope, 'borrow> ValidHandle for BorrowedArrayHandle<'scope, 'borrow> {
    unsafe fn get_value(&self, context: &ExecutionContext) -> *mut jl_value_t {
        context.get_value(self.0)
    }
}

impl<'scope, 'borrow> UnboxableHandle for BorrowedArrayHandle<'scope, 'borrow> {}

/// Handle to global Julia data.
///
/// You get these handles by calling either [`Module::function`] or [`Module::global`].
///
/// [`Module::function`]: ../data/module/struct.Module.html#method.function
/// [`Module::global`]: ../data/module/struct.Module.html#method.global
#[derive(Copy, Clone)]
pub struct GlobalHandle<'scope> {
    raw: *mut jl_value_t,
    _scope: PhantomData<&'scope Scope>,
}

impl<'scope> GlobalHandle<'scope> {
    pub(crate) fn new(raw: *mut jl_value_t, _: PhantomData<&'scope Scope>) -> Self {
        GlobalHandle {
            raw,
            _scope: PhantomData,
        }
    }

    /// Get a reference to this handle as a trait object.
    pub fn as_dyn(&self) -> &dyn ValidHandle {
        self as _
    }
}

impl<'scope> ValidHandle for GlobalHandle<'scope> {
    unsafe fn get_value(&self, _: &ExecutionContext) -> *mut jl_value_t {
        self.raw
    }
}

impl<'scope> UnboxableHandle for GlobalHandle<'scope> {}

impl<'scope> Call for GlobalHandle<'scope> {}
