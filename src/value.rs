//! Convert data from Rust to Julia and back. Call Julia functions.

use crate::array::Dimensions;
use crate::error::{Exception, JlrsError, JlrsResult};
use crate::frame::Output;
use crate::module::Module;
use crate::traits::{private::Internal, Frame, IntoJulia, JuliaType, TryUnbox};
use jl_sys::{
    jl_alloc_array_1d, jl_alloc_array_2d, jl_alloc_array_3d, jl_apply_array_type, jl_array_eltype,
    jl_call, jl_call0, jl_call1, jl_call2, jl_call3, jl_exception_occurred, jl_is_array,
    jl_new_array, jl_ptr_to_array, jl_ptr_to_array_1d, jl_typeis, jl_typeof_str, jl_value_t,
};
use std::marker::PhantomData;

/// Several values that are allocated consecutively. This can be used in combination with
/// [`Value::call_values`] and [`Value::call_values_output`].
/// 
/// [`Value::call_values`]: struct.Value.html#method.call_values
/// [`Value::call_values_output`]: struct.Value.html#method.call_values_output
#[derive(Copy, Clone)]
pub struct Values<'frame>(*mut *mut jl_value_t, usize, PhantomData<&'frame ()>);

impl<'frame> Values<'frame> {
    pub(crate) fn wrap(ptr: *mut *mut jl_value_t, n: usize) -> Self {
        Values(ptr, n, PhantomData)
    }

    /// Get a specific `Value` in this group. Returns an error if the index is out of bounds.
    pub fn value(&self, index: usize) -> JlrsResult<Value<'frame, 'static>> {
        if index >= self.1 {
            return Err(JlrsError::OutOfBounds(index, self.1).into());
        }

        unsafe {
            Ok(Value(
                *(self.0.offset(index as isize)),
                PhantomData,
                PhantomData,
            ))
        }
    }

    /// Allocate several values of the same type, this type must implement [`IntoJulia`]. The
    /// values will be protected from garbage collection inside the frame used to create them.
    /// This takes as many slots on the GC stack as values that are allocated. Returns an error if
    /// there is not enough space on the stack.
    ///
    /// [`IntoJulia`]: ../traits/trait.IntoJulia.html
    pub fn new<'base, T, V, F>(frame: &mut F, data: V) -> JlrsResult<Self>
    where
        'base: 'frame,
        T: IntoJulia,
        V: AsRef<[T]>,
        F: Frame<'base, 'frame>,
    {
        frame.create_many(data.as_ref(), Internal)
    }

    /// Allocate several values of possibly different types, these types must implement
    /// [`IntoJulia`]. The values will be protected from garbage collection inside the frame used
    /// to create them.This takes as many slots on the GC stack as values that are allocated.
    /// Returns an error if there is not enough space on the stack.
    ///
    /// [`IntoJulia`]: ../traits/trait.IntoJulia.html
    pub fn new_dyn<'value, 'base, V, F>(frame: &mut F, data: V) -> JlrsResult<Self>
    where
        'base: 'frame,
        V: AsRef<[&'value dyn IntoJulia]>,
        F: Frame<'base, 'frame>,
    {
        frame.create_many_dyn(data.as_ref(), Internal)
    }
}

/// Except modules, all Julia data is represented as a `Value` in `jlrs`.
///
/// A `Value` wraps around the raw value from the Julia C API and applies some restrictions
/// through lifetimes to ensure it can only be used while it's protected from garbage collection
/// and its contents are valid.
/// 
/// The methods that create a new `Value` come in two varieties: `method` and `method_output`. The
/// first will use a slot in the current frame to protect the value from garbage collection, while 
/// the latter uses a slot in an earlier frame.
///
/// [`Value::new`]: struct.Value.html#method.new
/// [`Module`]: ../module/struct.Module.html
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct Value<'frame, 'data>(
    *mut jl_value_t,
    PhantomData<&'frame ()>,
    PhantomData<&'data ()>,
);

impl<'frame, 'data> Value<'frame, 'data> {
    pub(crate) unsafe fn wrap(ptr: *mut jl_value_t) -> Value<'frame, 'static> {
        Value(ptr, PhantomData, PhantomData)
    }

    /// Create a new Julia value, any type that implements [`IntoJulia`] can be converted using
    /// this function. The value will be protected from garbage collection inside the frame used
    /// to create it. One free slot on the garbage collection stack is required for this function
    /// to succeed, returns an error if no slot is available.
    ///
    /// [`IntoJulia`]: ../traits/trait.IntoJulia.html
    pub fn new<'base, V, F>(frame: &mut F, value: V) -> JlrsResult<Value<'frame, 'static>>
    where
        'base: 'frame,
        V: IntoJulia,
        F: Frame<'base, 'frame>,
    {
        unsafe { frame.protect(value.into_julia(Internal), Internal) }
    }

    /// Create a new Julia value, any type that implements [`IntoJulia`] can be converted using
    /// this function. The value will be protected from garbage collection inside the frame used
    /// to create it. One free slot on the garbage collection stack is required for this function
    /// to succeed, returns an error if no slot is available.
    ///
    /// [`IntoJulia`]: ../traits/trait.IntoJulia.html
    pub fn new_output<'output, 'base, V, F>(
        frame: &mut F,
        output: Output<'output>,
        value: V,
    ) -> Value<'output, 'static>
    where
        'base: 'frame,
        V: IntoJulia,
        F: Frame<'base, 'frame>,
    {
        unsafe { frame.assign_output(output, value.into_julia(Internal), Internal) }
    }

    /// Returns true if the value is of type `T`.
    pub fn is<T: JuliaType>(&self) -> bool {
        unsafe { jl_typeis(self.0, T::julia_type(Internal) as _) }
    }

    /// Returns true if the value is an array.
    pub fn is_array(&self) -> bool {
        unsafe { jl_is_array(self.0) }
    }

    /// Returns true if the value is an array with elements of type `T`.
    pub fn is_array_of<T: JuliaType>(&self) -> bool {
        unsafe {
            self.is_array() && jl_array_eltype(self.0) as *mut jl_value_t == T::julia_type(Internal)
        }
    }

    /// If you call a function with one or more borrowed arrays as arguments, its result can only
    /// be used when all the borrows are active. If this result doesn't reference any borrowed
    /// data this function can be used to relax its second lifetime to `'static`.
    ///
    /// Safety: The value must not contain a reference any borrowed data.
    pub unsafe fn assume_owned(self) -> Value<'frame, 'static> {
        Value::wrap(self.0)
    }

    /// Extend the `Value`'s lifetime to the `Output's lifetime. The original value will still be
    /// valid after calling this method, the data will be protected from garbage collection until
    /// the `Output`'s frame goes out of scope.
    pub fn extend<'output, 'base, F>(
        self,
        frame: &mut F,
        output: Output<'output>,
    ) -> Value<'output, 'data>
    where
        'output: 'data,
        'base: 'frame,
        F: Frame<'base, 'frame>,
    {
        frame.assign_output(output, self.0 as _, Internal)
    }

    /// Allocates a new n-dimensional array in Julia.
    ///
    /// Allocating an array with one, two, or three dimensions requires one slot on the GC stack.
    /// If you allocate an array with more dimensions, an extra frame is created with `n + 1`
    /// slots, temporarily taking `n + 3` additional slots. This latter case requires that
    /// `jlrs.jl` has been included.
    ///
    /// This function returns an error if there are not enough slots available, or if `jlrs.jl`
    /// has not been included when allocating arrays with four or more dimensions.
    pub fn array<'base, T, D, F>(frame: &mut F, dimensions: D) -> JlrsResult<Value<'frame, 'static>>
    where
        'base: 'frame,
        T: JuliaType,
        D: Into<Dimensions>,
        F: Frame<'base, 'frame>,
    {
        unsafe {
            let array = array::<T, _, _>(frame, dimensions)?;
            frame.protect(array, Internal)
        }
    }

    pub fn array_output<'output, 'base, T, D, F>(
        frame: &mut F,
        output: Output<'output>,
        dimensions: D,
    ) -> JlrsResult<Value<'output, 'static>>
    where
        'base: 'frame,
        T: JuliaType,
        D: Into<Dimensions>,
        F: Frame<'base, 'frame>,
    {
        unsafe {
            let array = array::<T, _, _>(frame, dimensions)?;
            Ok(frame.assign_output(output, array, Internal))
        }
    }

    /// Borrows an n-dimensional array from Rust for use in Julia.
    ///
    /// Borrowing an array with one dimension requires one slot on the GC stack. If you borrow an
    /// array with more dimensions, an extra frame is created with `n + 1` slots, temporarily
    /// taking `n + 3` additional slots. This latter case requires that `jlrs.jl` has been
    /// included.
    ///
    /// This function returns an error if there are not enough slots available, or if `jlrs.jl`
    /// has not been included when borrowing arrays with two or more dimensions.
    ///
    /// This function is unsafe to call because you must ensure that the lifetime of this value is
    /// never extended through an `Output` by returning it from a Julia function, is never
    /// assigned to a global in Julia, and is never referenced from a value with a longer lifetime
    /// in Julia.
    pub fn borrow_array<'base, T, D, V, F>(
        frame: &mut F,
        data: &'data mut V,
        dimensions: D,
    ) -> JlrsResult<Value<'frame, 'data>>
    where
        'base: 'frame,
        T: JuliaType,
        D: Into<Dimensions>,
        V: AsMut<[T]>,
        F: Frame<'base, 'frame>,
    {
        unsafe {
            let array = borrow_array(frame, data, dimensions)?;
            frame.protect(array as _, Internal)
        }
    }

    pub fn borrow_array_output<'output, 'borrow, 'base, T, D, V, F>(
        frame: &mut F,
        output: Output<'output>,
        data: &'borrow mut V,
        dimensions: D,
    ) -> JlrsResult<Value<'output, 'borrow>>
    where
        'borrow: 'output,
        'base: 'frame,
        T: JuliaType,
        D: Into<Dimensions>,
        V: AsMut<[T]>,
        F: Frame<'base, 'frame>,
    {
        unsafe {
            let array = borrow_array(frame, data, dimensions)?;
            Ok(frame.assign_output(output, array as _, Internal))
        }
    }

    /// Moves an n-dimensional array from Rust to Julia.
    ///
    /// Moving an array with one dimension requires one slot on the GC stack. If you borrow an
    /// array with more dimensions, an extra frame is created with `n + 1` slots, temporarily
    /// taking `n + 3` additional slots. This latter case requires that `jlrs.jl` has been
    /// included.
    ///
    /// This function returns an error if there are not enough slots available, or if `jlrs.jl`
    /// has not been included when moving arrays with two or more dimensions.
    pub fn move_array<'base, T, D, F>(
        frame: &mut F,
        data: Vec<T>,
        dimensions: D,
    ) -> JlrsResult<Value<'frame, 'static>>
    where
        'base: 'frame,
        T: JuliaType,
        D: Into<Dimensions>,
        F: Frame<'base, 'frame>,
    {
        unsafe {
            let array = move_array(frame, data, dimensions)?;
            frame.protect(array as _, Internal)
        }
    }

    pub fn move_array_output<'output, 'base, T, D, F>(
        frame: &mut F,
        output: Output<'output>,
        data: Vec<T>,
        dimensions: D,
    ) -> JlrsResult<Value<'output, 'static>>
    where
        'base: 'frame,
        T: JuliaType,
        D: Into<Dimensions>,
        F: Frame<'base, 'frame>,
    {
        unsafe {
            let array = move_array(frame, data, dimensions)?;
            Ok(frame.assign_output(output, array as _, Internal))
        }
    }

    /// Try to copy data from Julia to Rust. You can only copy data if the output type implements
    /// [`TryUnbox`]; this trait is implemented by all types that implement [`IntoJulia`] and
    /// arrays whose contents implement [`ArrayData`] through [`Array`]. Returns an error if the
    /// requested type does not match the actual type of the data.
    ///
    /// [`TryUnbox`]: ../traits/trait.TryUnbox.html
    /// [`IntoJulia`]: ../traits/trait.IntoJulia.html
    /// [`ArrayData`]: ../traits/trait.ArrayData.html
    /// [`Array`]: ../array/struct.Array.html
    pub fn try_unbox<'base, T>(self) -> JlrsResult<T>
    where
        'base: 'frame,
        T: TryUnbox,
    {
        unsafe { T::try_unbox(self.0, Internal) }
    }

    /// Call this value as a function that takes zero arguments, this takes one slot on the GC
    /// stack. Returns the result of this function call if no exception is thrown, the exception
    /// if one is, or an error if no space is left on the stack.
    pub fn call0<'base, F>(self, frame: &mut F) -> JlrsResult<Value<'frame, 'static>>
    where
        'base: 'frame,
        F: Frame<'base, 'frame>,
    {
        unsafe {
            let res = jl_call0(self.0 as _);
            check_exception()?;
            frame.protect(res as _, Internal)
        }
    }

    /// Call this value as a function that takes zero arguments and use the `Output` to extend the
    /// result's lifetime. This takes no space on the GC stack. Returns the result of this
    /// function call if no exception is thrown or the exception if one is.
    pub fn call0_output<'output, 'base, F>(
        self,
        frame: &mut F,
        output: Output<'output>,
    ) -> JlrsResult<Value<'output, 'static>>
    where
        'base: 'frame,
        F: Frame<'base, 'frame>,
    {
        unsafe {
            let res = jl_call0(self.0 as _);
            check_exception()?;
            Ok(frame.assign_output(output, res as _, Internal))
        }
    }

    /// Call this value as a function that takes one argument, this takes one slot on the GC
    /// stack. Returns the result of this function call if no exception is thrown, the exception
    /// if one is, or an error if no space is left on the stack.
    pub fn call1<'borrow, 'base, F>(
        self,
        frame: &mut F,
        arg: Value<'_, 'borrow>,
    ) -> JlrsResult<Value<'frame, 'borrow>>
    where
        'base: 'frame,
        F: Frame<'base, 'frame>,
    {
        unsafe {
            let res = jl_call1(self.0 as _, arg.0 as _);
            check_exception()?;
            frame.protect(res as _, Internal)
        }
    }

    /// Call this value as a function that takes one argument and use the `Output` to extend the
    /// result's lifetime. This takes no space on the GC stack. Returns the result of this
    /// function call if no exception is thrown or the exception if one is.
    pub fn call1_output<'output, 'borrow, 'base, F>(
        self,
        frame: &mut F,
        output: Output<'output>,
        arg: Value<'_, 'borrow>,
    ) -> JlrsResult<Value<'output, 'borrow>>
    where
        'borrow: 'output,
        'base: 'frame,
        F: Frame<'base, 'frame>,
    {
        unsafe {
            let res = jl_call1(self.0 as _, arg.0 as _);
            check_exception()?;
            Ok(frame.assign_output(output, res as _, Internal))
        }
    }

    /// Call this value as a function that takes two arguments, this takes one slot on the GC
    /// stack. Returns the result of this function call if no exception is thrown, the exception
    /// if one is, or an error if no space is left on the stack.
    pub fn call2<'borrow, 'base, F>(
        self,
        frame: &mut F,
        arg0: Value<'_, 'borrow>,
        arg1: Value<'_, 'borrow>,
    ) -> JlrsResult<Value<'frame, 'borrow>>
    where
        'base: 'frame,
        F: Frame<'base, 'frame>,
    {
        unsafe {
            let res = jl_call2(self.0 as _, arg0.0 as _, arg1.0 as _);
            check_exception()?;
            frame.protect(res as _, Internal)
        }
    }

    /// Call this value as a function that takes two arguments and use the `Output` to extend the
    /// result's lifetime. This takes no space on the GC stack. Returns the result of this
    /// function call if no exception is thrown or the exception if one is.
    pub fn call2_output<'output, 'borrow, 'base, F>(
        self,
        frame: &mut F,
        output: Output<'output>,
        arg0: Value<'_, 'borrow>,
        arg1: Value<'_, 'borrow>,
    ) -> JlrsResult<Value<'output, 'borrow>>
    where
        'borrow: 'output,
        'base: 'frame,
        F: Frame<'base, 'frame>,
    {
        unsafe {
            let res = jl_call2(self.0 as _, arg0.0 as _, arg1.0 as _);
            check_exception()?;
            Ok(frame.assign_output(output, res as _, Internal))
        }
    }

    /// Call this value as a function that takes three arguments, this takes one slot on the GC
    /// stack. Returns the result of this function call if no exception is thrown, the exception
    /// if one is, or an error if no space is left on the stack.
    pub fn call3<'borrow, 'base, F>(
        self,
        frame: &mut F,
        arg0: Value<'_, 'borrow>,
        arg1: Value<'_, 'borrow>,
        arg2: Value<'_, 'borrow>,
    ) -> JlrsResult<Value<'frame, 'borrow>>
    where
        'base: 'frame,
        F: Frame<'base, 'frame>,
    {
        unsafe {
            let res = jl_call3(self.0 as _, arg0.0 as _, arg1.0 as _, arg2.0 as _);
            check_exception()?;
            frame.protect(res as _, Internal)
        }
    }

    /// Call this value as a function that takes three arguments and use the `Output` to extend
    /// the result's lifetime. This takes no space on the GC stack. Returns the result of this
    /// function call if no exception is thrown or the exception if one is.
    pub fn call3_output<'output, 'borrow, 'base, F>(
        self,
        frame: &mut F,
        output: Output<'output>,
        arg0: Value<'_, 'borrow>,
        arg1: Value<'_, 'borrow>,
        arg2: Value<'_, 'borrow>,
    ) -> JlrsResult<Value<'output, 'borrow>>
    where
        'borrow: 'output,
        'base: 'frame,
        F: Frame<'base, 'frame>,
    {
        unsafe {
            let res = jl_call3(self.0 as _, arg0.0 as _, arg1.0 as _, arg2.0 as _);
            check_exception()?;
            Ok(frame.assign_output(output, res as _, Internal))
        }
    }

    /// Call this value as a function that takes several arguments, this takes one slot on the GC
    /// stack. Returns the result of this function call if no exception is thrown, the exception
    /// if one is, or an error if no space is left on the stack.
    pub fn call<'value, 'borrow, 'base, V, F>(
        self,
        frame: &mut F,
        args: V,
    ) -> JlrsResult<Value<'frame, 'borrow>>
    where
        'base: 'frame,
        V: AsRef<[Value<'value, 'borrow>]>,
        F: Frame<'base, 'frame>,
    {
        unsafe {
            let args = args.as_ref();
            let n = args.len();
            let res = jl_call(self.0 as _, args.as_ptr() as _, n as _);
            check_exception()?;
            frame.protect(res as _, Internal)
        }
    }

    /// Call this value as a function that takes several arguments and use the `Output` to extend
    /// the result's lifetime. This takes no space on the GC stack. Returns the result of this
    /// function call if no exception is thrown or the exception if one is.
    pub fn call_output<'output, 'value, 'borrow, 'base, V, F>(
        self,
        frame: &mut F,
        output: Output<'output>,
        args: V,
    ) -> JlrsResult<Value<'output, 'static>>
    where
        'borrow: 'output,
        'base: 'frame,
        V: AsRef<[Value<'value, 'borrow>]>,
        F: Frame<'base, 'frame>,
    {
        unsafe {
            let args = args.as_ref();
            let n = args.len();
            let res = jl_call(self.0 as _, args.as_ptr() as _, n as _);
            check_exception()?;
            Ok(frame.assign_output(output, res as _, Internal))
        }
    }

    /// Call this value as a function that takes several arguments in a single `Values`, this
    /// takes one slot on the GC stack. Returns the result of this function call if no exception
    /// is thrown, the exception if one is, or an error if no space is left on the stack.
    pub fn call_values<'base, F>(
        self,
        frame: &mut F,
        args: Values,
    ) -> JlrsResult<Value<'frame, 'static>>
    where
        'base: 'frame,
        F: Frame<'base, 'frame>,
    {
        unsafe {
            let res = jl_call(self.0 as _, args.0, args.1 as _);
            check_exception()?;
            frame.protect(res as _, Internal)
        }
    }

    /// Call this value as a function that takes several arguments in a single `Values` and use
    /// the `Output` to extend the result's lifetime. This takes no space on the GC stack. Returns
    /// the result of this function call if no exception is thrown or the exception if one is.
    pub fn call_values_output<'output, 'base, F>(
        self,
        frame: &mut F,
        output: Output<'output>,
        args: Values,
    ) -> JlrsResult<Value<'output, 'static>>
    where
        'base: 'frame,
        F: Frame<'base, 'frame>,
    {
        unsafe {
            let res = jl_call(self.0 as _, args.0, args.1 as _);
            check_exception()?;
            Ok(frame.assign_output(output, res as _, Internal))
        }
    }
}

unsafe fn check_exception() -> JlrsResult<()> {
    let exc = jl_exception_occurred();
    if !exc.is_null() {
        let exc = Exception::new(jl_typeof_str(exc));
        return Err(JlrsError::ExceptionOccurred(exc).into());
    }

    Ok(())
}

unsafe fn array<'base, 'frame, T, D, F>(frame: &mut F, dimensions: D) -> JlrsResult<*mut jl_value_t>
where
    'base: 'frame,
    T: JuliaType,
    D: Into<Dimensions>,
    F: Frame<'base, 'frame>,
{
    let dims = dimensions.into();
    let array_type = jl_apply_array_type(T::julia_type(Internal), dims.n_dimensions() as _);

    match dims.n_dimensions() {
        1 => Ok(jl_alloc_array_1d(array_type, dims.n_elements(0) as _).cast()),
        2 => Ok(
            jl_alloc_array_2d(array_type, dims.n_elements(0) as _, dims.n_elements(1) as _).cast(),
        ),
        3 => Ok(jl_alloc_array_3d(
            array_type,
            dims.n_elements(0) as _,
            dims.n_elements(1) as _,
            dims.n_elements(2) as _,
        )
        .cast()),
        n => frame.frame(n as usize + 1, |frame| {
            let func = Module::main(frame)
                .submodule("Jlrs")?
                .function("arraydims")?;

            let v = Values::new(frame, dims.as_slice())?;
            let dims = func.call_values(frame, v)?;
            Ok(jl_new_array(array_type, dims.0).cast())
        }),
    }
}

unsafe fn borrow_array<'base, 'data, 'frame, T, D, V, F>(
    frame: &mut F,
    data: &'data mut V,
    dimensions: D,
) -> JlrsResult<*mut jl_value_t>
where
    'base: 'frame,
    T: JuliaType,
    D: Into<Dimensions>,
    V: AsMut<[T]>,
    F: Frame<'base, 'frame>,
{
    let dims = dimensions.into();
    let array_type = jl_apply_array_type(T::julia_type(Internal), dims.n_dimensions() as _);

    match dims.n_dimensions() {
        1 => Ok(jl_ptr_to_array_1d(
            array_type,
            data.as_mut().as_mut_ptr() as _,
            dims.n_elements(0) as _,
            0,
        )
        .cast()),
        n => frame.frame(n as usize + 1, |frame| {
            let func = Module::main(frame)
                .submodule("Jlrs")?
                .function("arraydims")?;
            let v = Values::new(frame, dims.as_slice())?;
            let dims = func.call_values(frame, v)?;
            Ok(jl_ptr_to_array(array_type, data.as_mut().as_mut_ptr().cast(), dims.0, 0).cast())
        }),
    }
}

unsafe fn move_array<'base, 'frame, T, D, F>(
    frame: &mut F,
    data: Vec<T>,
    dimensions: D,
) -> JlrsResult<*mut jl_value_t>
where
    'base: 'frame,
    T: JuliaType,
    D: Into<Dimensions>,
    F: Frame<'base, 'frame>,
{
    let dims = dimensions.into();
    let array_type = jl_apply_array_type(T::julia_type(Internal), dims.n_dimensions() as _);

    match dims.n_dimensions() {
        1 => Ok(jl_ptr_to_array_1d(
            array_type,
            Box::into_raw(data.into_boxed_slice()).cast(),
            dims.n_elements(0) as _,
            1,
        )
        .cast()),
        n => frame.frame(n as usize + 1, |frame| {
            let func = Module::main(frame)
                .submodule("Jlrs")?
                .function("arraydims")?;
            let v = Values::new(frame, dims.as_slice())?;
            let dims = func.call_values(frame, v)?;
            Ok(jl_ptr_to_array(
                array_type,
                Box::into_raw(data.into_boxed_slice()).cast(),
                dims.0,
                1,
            )
            .cast())
        }),
    }
}
