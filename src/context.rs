//! Allocate data and interact with Julia.
//!
//! The three contexts defined in this module serve distinct functions:
//!  - The [`Session`] lets you allocate data that's valid until the session is dropped. After
//!    calling [`Session::execute`] you can continue using the session.
//!  - The [`AllocationContext`] lets you allocate data the same way that a [`Session`] does, this
//!    data is valid until the context goes out of scope. After calling
//!    [`AllocationContext::execute`] you can no longer use the context.
//!  - The [`ExecutionContext`] lets you call Julia functions and copy data from Julia to Rust.
//!    Within this context all handles to data you can use are guaranteed to be protected from
//!    garbage collection.
//!
//! [`Session`]: struct.Session.html
//! [`Session::execute`]: struct.Session.html#method.execute
//! [`AllocationContext`]: struct.AllocationContext.html
//! [`AllocationContext::execute`]: struct.AllocationContext.html#method.execute
//! [`ExecutionContext`]: struct.ExecutionContext.html

use crate::dimensions::Dimensions;
use crate::error::JlrsResult;
use crate::handles::{
    AssignedHandle, BorrowedArrayHandle, PrimitiveHandles, UnassignedHandle, UninitArrayHandle,
};
use crate::memory::Memory;
use crate::module::Module;
use crate::traits::{IntoPrimitive, JuliaType, TryUnbox, UnboxableHandle};
use jl_sys::jl_value_t;
use std::marker::PhantomData;

pub(crate) struct Scope;

pub(crate) struct MemWrap<'scope>(pub &'scope mut Memory);
impl<'scope> MemWrap<'scope> {
    pub(crate) fn new(memory: &'scope mut Memory) -> Self {
        MemWrap(memory)
    }
}

/// You need to use a `Session` in order to do anything with Julia, with the exception of
/// including Julia code. You get a mutable reference to this struct by calling
/// [`Runtime::session`], which takes a closure with that reference as its argument.
///
/// [`Runtime::session`]: ../struct.Runtime.html#method.session
pub struct Session<'session> {
    memory: &'session mut Memory,
}

impl<'session> Session<'session> {
    pub(crate) fn new(memory: &'session mut Memory, _: &'session Scope) -> Self {
        Session { memory }
    }

    /// Get a handle to Julia's `Main`-module. If you include your own Julia code by calling
    /// [`Runtime::include`], handles to functions, globals, and submodules defined in these
    /// included files are available through this module.
    ///
    /// [`Runtime::include`]: ../struct.Runtime.html#method.include
    pub fn main_module(&self) -> Module<'session> {
        unsafe { Module::main() }
    }

    /// Get a handle to Julia's `Core`-module.
    pub fn core_module(&self) -> Module<'session> {
        unsafe { Module::core() }
    }

    /// Get a handle to Julia's `Base`-module.
    pub fn base_module(&self) -> Module<'session> {
        unsafe { Module::base() }
    }

    /// Create a new [`UnassignedHandle`], you need one for every function you call. Returns an
    /// error if the stack size is too small.
    ///
    /// [`UnassignedHandle`]: ../handles/struct.UnassignedHandle.html
    pub fn new_unassigned(&mut self) -> JlrsResult<UnassignedHandle<'session>> {
        self.memory.new_unassigned()
    }

    /// Copy a value from Rust to Julia and get a handle to this data. Returns an error if the
    /// stack size is too small.
    ///
    /// The following types can be copied to Julia this way:
    ///  - `bool`
    ///  - `char`
    ///  - `u8`
    ///  - `u16`
    ///  - `u32`
    ///  - `u64`
    ///  - `usize`
    ///  - `i8`
    ///  - `i16`
    ///  - `i32`
    ///  - `i64`
    ///  - `isize`
    ///  - `f32`
    ///  - `f64`
    pub fn new_primitive<T: IntoPrimitive>(
        &mut self,
        value: T,
    ) -> JlrsResult<AssignedHandle<'session>> {
        self.memory.new_primitive(value)
    }

    /// Copy multiple values of the same type from Rust to Julia and get a handle to these values.
    /// Returns an error if the stack size is too small.
    ///
    /// This handle can be used in combination with [`Call:call_primitives`] to call a function
    /// with all of these values. The same types can be used for the values as those supported by
    /// [`AllocationContext::new_primitive`].
    ///
    /// [`Call:call_primitives`]: ../traits/trait.Call.html#method.call_primitives
    /// [`AllocationContext::new_primitive`]: struct.AllocationContext.html#method.new_primitive
    pub fn new_primitives<P: AsRef<[impl IntoPrimitive]>>(
        &mut self,
        values: P,
    ) -> JlrsResult<PrimitiveHandles<'session>> {
        self.memory.new_primitives(values)
    }

    /// Copy multiple values of the possibly different types from Rust to Julia and get a
    /// handle to these values. Returns an error if the stack size is too small.
    ///
    /// This handle can be used in combination with [`Call:call_primitives`] to call a function
    /// with all of these values. The same types can be used for the values as those supported by
    /// [`AllocationContext::new_primitive`].
    ///
    /// [`Call:call_primitives`]: ../traits/trait.Call.html#method.call_primitives
    /// [`AllocationContext::new_primitive`]: struct.AllocationContext.html#method.new_primitive
    pub fn new_primitives_dyn<'input, P: AsRef<[&'input dyn IntoPrimitive]>>(
        &mut self,
        values: P,
    ) -> JlrsResult<PrimitiveHandles<'session>> {
        self.memory.new_primitives_dyn(values)
    }

    /// Create a new n-dimensional array that is managed by Julia and get a handle to this array.
    /// Returns an error if the stack size is too small. If you want to create a managed array
    /// with four or more dimensions, you must include `jlrs.jl` first which you can find in the
    /// root of this crate's github repository.
    ///
    /// This array can only contain data that implements [`JuliaType`], ie all types supported by
    /// [`AllocationContext::new_primitive`] except `bool` and `char`; instead of these two types
    /// you can use `i8` and `u32` respectively. Besides the type of the array's contents you must
    /// also declare its dimensions.
    ///
    /// After the array is allocated it will contain uninitialized data. See [`UninitArrayHandle`]
    /// for information on how to initialize its contents.
    ///
    /// [`JuliaType`]: ../traits/trait.JuliaType.html
    /// [`AllocationContext::new_primitive`]: struct.AllocationContext.html#method.new_primitive
    /// [`UninitArrayHandle`]: ../handles/struct.UninitArrayHandle.html
    pub fn new_managed_array<T: JuliaType + Copy, D: Into<Dimensions>>(
        &mut self,
        dims: D,
    ) -> JlrsResult<UninitArrayHandle<'session, T>> {
        self.memory.new_managed_array(dims.into())
    }

    /// Create a new n-dimensional array where the ownership of the data is essentially transfered
    /// from Rust to the Julia garbage collector and get a handle to this array. The allocated
    /// memory will be freed  by the GC when no references exist to it in Julia. Returns an error
    /// if the stack size is too small. If you want to create an owned array with more than one
    /// dimension, you must include `jlrs.jl` first which you can find in the root of this crate's
    /// github repository.
    ///
    /// This array can only contain data that implements [`JuliaType`], ie all types supported by
    /// [`AllocationContext::new_primitive`] except `bool` and `char`; instead of these two types
    /// you can use `i8` and `u32` respectively. You must also declare its dimensions.
    ///
    /// Arrays in Julia have column-major ordering. Your data must respect this. Because the data
    /// isn't allocated from Julia, operations that change the size of the array will fail. For
    /// example, you cannot use `Base.push!`.
    ///
    /// [`JuliaType`]: ../traits/trait.JuliaType.html
    /// [`AllocationContext::new_primitive`]: struct.AllocationContext.html#method.new_primitive
    pub fn new_owned_array<T: JuliaType, D: Into<Dimensions>>(
        &mut self,
        data: Vec<T>,
        dims: D,
    ) -> JlrsResult<AssignedHandle<'session>> {
        self.memory.new_owned_array(data, dims.into())
    }

    /// Create a new n-dimensional array that borrows its data from Rust and get a
    /// handle to this array. Returns an error if the stack size is too small. This is unsafe
    /// because you're responsible for ensuring this data will never be used from Julia after the
    /// borrow has ended. In general, you should never make a global value in Julia depend on this
    /// data and never return this data as a function output. If you want to borrow an array with
    /// more than one dimension, you must include `jlrs.jl` first which you can find in the root
    /// of this crate's github repository.
    ///
    /// This array can only contain data that implements [`JuliaType`], ie all types supported by
    /// [`AllocationContext::new_primitive`] except `bool` and `char`; instead of these two types
    /// you can use `i8` and `u32` respectively. You must also declare its dimensions.
    ///
    /// Arrays in Julia have column-major ordering. Your data must respect this. Because the data
    /// isn't allocated from Julia, operations that change the size of the array will fail. For
    /// example, you cannot use `Base.push!`.
    ///
    /// [`JuliaType`]: ../traits/trait.JuliaType.html
    /// [`AllocationContext::new_primitive`]: struct.AllocationContext.html#method.new_primitive
    pub unsafe fn borrow_array<'borrow, T: JuliaType, D: Into<Dimensions>, U: AsMut<[T]>>(
        &mut self,
        data: &'borrow mut U,
        dims: D,
    ) -> JlrsResult<BorrowedArrayHandle<'session, 'borrow>> {
        self.memory.new_borrowed_array(data, dims.into())
    }

    /// Copy a string from Rust to Julia and get a handle to this data. All UTF-8 encoded strings
    /// are valid strings in Julia. Returns an error if the stack size is too small.
    pub fn new_string<S: Into<String>>(
        &mut self,
        string: S,
    ) -> JlrsResult<AssignedHandle<'session>> {
        self.memory.new_string(string.into())
    }

    /// Allocate temporary data. This method takes a closure with a single argument, an
    /// [`AllocationContext`], which you can use to allocate data that can only be used within
    /// that closure. This method returns the closure's output.
    ///
    /// [`AllocationContext`]: struct.AllocationContext.html
    pub fn with_temporaries<T, F: FnOnce(AllocationContext) -> JlrsResult<T>>(
        &mut self,
        func: F,
    ) -> JlrsResult<T> {
        unsafe {
            self.memory.push_nonempty_frame()?;
            let scope = Scope;
            let mut mem = MemWrap::new(self.memory);
            let alloc_ctx = AllocationContext::new(&mut mem, &scope);
            func(alloc_ctx)
        }
    }

    /// Stop allocating data and call the given closure. This closure takes a single argument, a
    /// mutable reference to an [`ExecutionContext`], which lets you use all handles in scope,
    /// call Julia functions and copy data from Julia to Rust. This method returns the closure's
    /// output.
    ///
    /// Unlike [`AllocationContext::execute`], which takes ownership of its context, this method
    /// borrows the [`Session`]. You can continue allocating and executing after calling this
    /// method. Data allocated with a [`Session`] is protected from garbage collection until the
    /// [`Session`] is dropped.
    ///
    /// [`ExecutionContext`]: struct.ExecutionContext.html
    /// [`AllocationContext::execute`]: struct.AllocationContext.html#method.execute
    /// [`Session`]: struct.Session.html
    pub fn execute<T, F: FnOnce(&mut ExecutionContext) -> JlrsResult<T>>(
        &mut self,
        func: F,
    ) -> JlrsResult<T> {
        unsafe {
            self.memory.push_nonempty_frame()?;
            let mut exec_ctx = ExecutionContext::new_session(self.memory)?;
            func(&mut exec_ctx)
        }
    }
}

impl<'session> Drop for Session<'session> {
    fn drop(&mut self) {
        unsafe {
            self.memory.clear_pending();
            self.memory.pop_all_frames();
        }
    }
}

/// Allocate temporary data. This offers the same functionality as [`Session`] does, but allocated
/// data only lives until this context is dropped.
///
/// You get access to an [`AllocationContext`] inside the closure that
/// [`Session::with_temporaries`] takes as its argument. You can use data you've allocated here by
/// calling [`AllocationContext::execute`].
///
/// [`Session`]: struct.Session.html
/// [`AllocationContext`]: struct.AllocationContext.html
/// [`Session::with_temporaries`]: struct.Session.html#method.with_temporaries
/// [`AllocationContext::execute`]: struct.AllocationContext.html#method.execute
pub struct AllocationContext<'block, 'session: 'block> {
    memory: &'block mut MemWrap<'session>,
}

impl<'block, 'session: 'block> AllocationContext<'block, 'session> {
    pub(crate) fn new(memory: &'block mut MemWrap<'session>, _: &'block Scope) -> Self {
        AllocationContext { memory }
    }

    /// Create a new [`UnassignedHandle`], you need one for every function you call. Returns an
    /// error if the stack size is too small.
    ///
    /// [`UnassignedHandle`]: ../handles/struct.UnassignedHandle.html
    pub fn new_unassigned(&mut self) -> JlrsResult<UnassignedHandle<'block>> {
        self.memory.0.new_unassigned()
    }

    /// Copy a value from Rust to Julia and get a handle to this data. Returns an error if the
    /// stack size is too small.
    ///
    /// The following types can be copied to Julia this way:
    ///  - `bool`
    ///  - `char`
    ///  - `u8`
    ///  - `u16`
    ///  - `u32`
    ///  - `u64`
    ///  - `usize`
    ///  - `i8`
    ///  - `i16`
    ///  - `i32`
    ///  - `i64`
    ///  - `isize`
    ///  - `f32`
    ///  - `f64`
    pub fn new_primitive<T>(&mut self, value: T) -> JlrsResult<AssignedHandle<'block>>
    where
        T: IntoPrimitive,
    {
        self.memory.0.new_primitive(value)
    }

    /// Copy multiple values of the same type from Rust to Julia and get a handle to these values.
    /// Returns an error if the stack size is too small.
    ///
    /// This handle can be used in combination with [`Call:call_primitives`] to call a function
    /// with all of these values. The same types can be used for the values as those supported by
    /// [`AllocationContext::new_primitive`].
    ///
    /// [`Call:call_primitives`]: ../traits/trait.Call.html#method.call_primitives
    /// [`AllocationContext::new_primitive`]: struct.AllocationContext.html#method.new_primitive
    pub fn new_primitives<T, P>(&mut self, values: P) -> JlrsResult<PrimitiveHandles<'block>>
    where
        T: IntoPrimitive,
        P: AsRef<[T]>,
    {
        self.memory.0.new_primitives(values)
    }

    /// Copy multiple values of the possibly different types from Rust to Julia and get a
    /// handle to these values. Returns an error if the stack size is too small.
    ///
    /// This handle can be used in combination with [`Call:call_primitives`] to call a function
    /// with all of these values. The same types can be used for the values as those supported by
    /// [`AllocationContext::new_primitive`].
    ///
    /// [`Call:call_primitives`]: ../traits/trait.Call.html#method.call_primitives
    /// [`AllocationContext::new_primitive`]: struct.AllocationContext.html#method.new_primitive
    pub fn new_primitives_dyn<P>(&mut self, values: P) -> JlrsResult<PrimitiveHandles<'block>>
    where
        P: AsRef<[&'block dyn IntoPrimitive]>,
    {
        self.memory.0.new_primitives_dyn(values)
    }

    /// Create a new n-dimensional array that is managed by Julia and get a handle to this array.
    /// Returns an error if the stack size is too small. If you want to create a managed array
    /// with four or more dimensions, you must include `jlrs.jl` first which you can find in the
    /// root of this crate's github repository.
    ///
    /// This array can only contain data that implements [`JuliaType`], ie all types supported by
    /// [`AllocationContext::new_primitive`] except `bool` and `char`; instead of these two types
    /// you can use `i8` and `u32` respectively. Besides the type of the array's contents you must
    /// also declare its dimensions.
    ///
    /// After the array is allocated it will contain uninitialized data. See [`UninitArrayHandle`]
    /// for information on how to initialize its contents.
    ///
    /// [`JuliaType`]: ../traits/trait.JuliaType.html
    /// [`AllocationContext::new_primitive`]: struct.AllocationContext.html#method.new_primitive
    /// [`UninitArrayHandle`]: ../handles/struct.UninitArrayHandle.html
    pub fn new_managed_array<T, D>(&mut self, dims: D) -> JlrsResult<UninitArrayHandle<'block, T>>
    where
        T: JuliaType + Copy,
        D: Into<Dimensions>,
    {
        self.memory.0.new_managed_array(dims.into())
    }

    /// Create a new n-dimensional array where the ownership of the data is essentially transfered
    /// from Rust to the Julia garbage collector and get a handle to this array. The allocated
    /// memory will be freed  by the GC when no references exist to it in Julia. Returns an error
    /// if the stack size is too small. If you want to create an owned array with more than one
    /// dimension, you must include `jlrs.jl` first which you can find in the root of this crate's
    /// github repository.
    ///
    /// This array can only contain data that implements [`JuliaType`], ie all types supported by
    /// [`AllocationContext::new_primitive`] except `bool` and `char`; instead of these two types
    /// you can use `i8` and `u32` respectively. You must also declare its dimensions.
    ///
    /// Arrays in Julia have column-major ordering. Your data must respect this. Because the data
    /// isn't allocated from Julia, operations that change the size of the array will fail. For
    /// example, you cannot use `Base.push!`.
    ///
    /// [`JuliaType`]: ../traits/trait.JuliaType.html
    /// [`AllocationContext::new_primitive`]: struct.AllocationContext.html#method.new_primitive
    pub fn new_owned_array<T, D>(
        &mut self,
        data: Vec<T>,
        dims: D,
    ) -> JlrsResult<AssignedHandle<'block>>
    where
        T: JuliaType,
        D: Into<Dimensions>,
    {
        self.memory.0.new_owned_array(data, dims.into())
    }

    /// Create a new n-dimensional array that borrows its data from Rust and get a
    /// handle to this array. Returns an error if the stack size is too small. This is unsafe
    /// because you're responsible for ensuring this data will never be used from Julia after the
    /// borrow has ended. In general, you should never make a global value in Julia depend on this
    /// data and never return this data as a function output. If you want to borrow an array with
    /// more than one dimension, you must include `jlrs.jl` first which you can find in the root
    /// of this crate's github repository.
    ///
    /// This array can only contain data that implements [`JuliaType`], ie all types supported by
    /// [`AllocationContext::new_primitive`] except `bool` and `char`; instead of these two types
    /// you can use `i8` and `u32` respectively. You must also declare its dimensions.
    ///
    /// Arrays in Julia have column-major ordering. Your data must respect this. Because the data
    /// isn't allocated from Julia, operations that change the size of the array will fail. For
    /// example, you cannot use `Base.push!`.
    ///
    /// [`JuliaType`]: ../traits/trait.JuliaType.html
    /// [`AllocationContext::new_primitive`]: struct.AllocationContext.html#method.new_primitive
    pub unsafe fn borrow_array<'borrow, T, D, U>(
        &mut self,
        data: &'borrow mut U,
        dims: D,
    ) -> JlrsResult<BorrowedArrayHandle<'block, 'borrow>>
    where
        T: JuliaType,
        D: Into<Dimensions>,
        U: AsMut<[T]>,
    {
        self.memory.0.new_borrowed_array(data, dims.into())
    }

    /// Copy a string from Rust to Julia and get a handle to this data. All UTF-8 encoded strings
    /// are valid strings in Julia. Returns an error if the stack size is too small.
    pub fn new_string<S>(&mut self, string: S) -> JlrsResult<AssignedHandle<'block>>
    where
        S: Into<String>,
    {
        self.memory.0.new_string(string.into())
    }

    /// Stop allocating data and call the given closure. This closure takes a single argument, a
    /// mutable reference to an [`ExecutionContext`], which lets you use all handles in scope,
    /// call Julia functions and copy data from Julia to Rust. This method returns the closure's
    /// output.
    ///
    /// [`ExecutionContext`]: struct.ExecutionContext.html
    pub fn execute<T, F>(self, func: F) -> JlrsResult<T>
    where
        F: FnOnce(&mut ExecutionContext) -> JlrsResult<T>,
    {
        let scope = Scope;
        let mut exec_ctx = ExecutionContext::new(self.memory.0, &scope)?;
        func(&mut exec_ctx)
    }
}

impl<'block, 'session: 'block> Drop for AllocationContext<'block, 'session> {
    fn drop(&mut self) {
        // clean up if nothing was actually executed
        // this is safe we won't be able to use these pending items
        unsafe { self.memory.0.clear_pending() }
    }
}

/// Use handles that are in scope to call functions and unbox data. You get access to struct by
/// calling [`Session::execute`] or [`AllocationContext::execute`].
///
/// [`Session::execute`]: struct.Session.html#method.execute
/// [`AllocationContext::execute`]: struct.AllocationContext.html#method.execute
pub struct ExecutionContext<'block, 'session: 'block> {
    memory: &'block mut Memory,
    _scope: PhantomData<&'session Scope>,
}

impl<'block, 'session: 'block> ExecutionContext<'block, 'session> {
    pub(crate) fn new(memory: &'session mut Memory, _: &'block Scope) -> JlrsResult<Self> {
        // Safe because we pop this frame when this context is dropped
        unsafe {
            memory.push_frame()?;
        }
        Ok(ExecutionContext {
            memory,
            _scope: PhantomData,
        })
    }

    pub(crate) fn new_session(memory: &'session mut Memory) -> JlrsResult<Self> {
        // Safe because we pop this frame when this context is dropped
        unsafe {
            memory.push_frame()?;
        }
        Ok(ExecutionContext {
            memory,
            _scope: PhantomData,
        })
    }

    /// Try to copy data from Julia to Rust. You can only copy data if the output type implements
    /// [`TryUnbox`]; this trait is implemented by all types that implement `IntoPrimitive`,
    /// strings, and arrays whose contents implement [`Unboxable`] through [`UnboxedArray`].
    /// Returns an error if the requested type does not match the actual type of the data.
    ///
    /// [`TryUnbox`]: ../traits/trait.TryUnbox.html
    /// [`IntoPrimitive`]: ../traits/trait.IntoPrimitive.html
    /// [`Unboxable`]: ../traits/trait.Unboxable.html
    /// [`UnboxedArray`]: ../unboxed_array/struct.UnboxedArray.html
    pub fn try_unbox<T: TryUnbox>(&self, handle: &dyn UnboxableHandle) -> JlrsResult<T> {
        // Safe because the handle is valid
        unsafe {
            let value = handle.get_value(self);
            T::try_unbox(value)
        }
    }

    /// Get a handle to Julia's `Main`-module. If you include your own Julia code by calling
    /// [`Runtime::include`], handles to functions, globals, and submodules defined in these
    /// included files are available through this module.
    ///
    /// [`Runtime::include`]: ../struct.Runtime.html#method.include
    pub fn main_module(&self) -> Module<'session> {
        // Safe because jl_init has been called
        unsafe { Module::main() }
    }

    /// Get a handle to Julia's `Core`-module.
    pub fn core_module(&self) -> Module<'session> {
        // Safe because jl_init has been called
        unsafe { Module::core() }
    }

    /// Get a handle to Julia's `Base`-module.
    pub fn base_module(&self) -> Module<'session> {
        // Safe because jl_init has been called
        unsafe { Module::base() }
    }

    // index must be from an UnassignedHandle
    pub(crate) unsafe fn assign(&mut self, index: usize, value: *mut jl_value_t) {
        self.memory.assign(index, value)
    }

    // index must be from a handle that implements ValidHandle
    pub unsafe fn get_value(&self, index: usize) -> *mut jl_value_t {
        self.memory.get_value(index)
    }

    // index must be from a PrimitivesHandle
    pub unsafe fn get_values(&self, index: usize, n: usize) -> *mut *mut jl_value_t {
        self.memory.get_values(index, n)
    }
}

impl<'block, 'session: 'block> Drop for ExecutionContext<'block, 'session> {
    fn drop(&mut self) {
        // safe because we're popping the frame that was pushed when this
        // struct was created
        unsafe { self.memory.pop_frame() }
    }
}
