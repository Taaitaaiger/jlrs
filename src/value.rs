//! Convert data from Rust to Julia and back. Call Julia functions.

use crate::array::Dimensions;
use crate::error::{Exception, JlrsError, JlrsResult};
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

        unsafe { Ok(Value(*(self.0.offset(index as isize)), PhantomData, PhantomData)) }
    }

    /// Allocate several values of the same type, this type must implement [`IntoJulia`]. The
    /// values will be protected from garbage collection inside the frame used to create them.
    /// This takes as many slots on the GC stack as values that are allocated. Returns an error if
    /// there is not enough space on the stack.
    ///
    /// [`IntoJulia`]: ../traits/trait.IntoJulia.html
    pub fn new<'base: 'frame, P: IntoJulia, V: AsRef<[P]>, F: Frame<'base, 'frame>>(
        frame: &mut F,
        data: V,
    ) -> JlrsResult<Self> {
        frame.create_many(data.as_ref(), Internal)
    }

    /// Allocate several values of possibly different types, these types must implement
    /// [`IntoJulia`]. The values will be protected from garbage collection inside the frame used
    /// to create them.This takes as many slots on the GC stack as values that are allocated.
    /// Returns an error if there is not enough space on the stack.
    ///
    /// [`IntoJulia`]: ../traits/trait.IntoJulia.html
    pub fn new_dyn<'value, 'base: 'frame, V: AsRef<[&'value dyn IntoJulia]>, F: Frame<'base, 'frame>>(
        frame: &mut F,
        data: V,
    ) -> JlrsResult<Self> {
        frame.create_many_dyn(data.as_ref(), Internal)
    }
}

/// Except modules, all Julia data is represented as a `Value` in `jlrs`. You can create values
/// with [`Value::new`]. Functions, either acquired through a [`Module`] or returned from another
/// function, are also `Value`s. It's not possible to check if a `Value` is a function, as a
/// result all `Value`s can be called.
///
/// [`Value::new`]: struct.Value.html#method.new
/// [`Module`]: ../module/struct.Module.html
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct Value<'frame, 'borrow>(*mut jl_value_t, PhantomData<&'frame ()>, PhantomData<&'borrow ()>);

impl<'frame, 'borrow> Value<'frame, 'borrow> {
    pub(crate) unsafe fn wrap(ptr: *mut jl_value_t) -> Value<'frame, 'static> {
        Value(ptr, PhantomData, PhantomData)
    }

    /// Create a new Julia value, any type that implements [`IntoJulia`] can be converted using
    /// this function. The value will be protected from garbage collection inside the frame used
    /// to create it. One free slot on the garbage collection stack is required for this function
    /// to succeed, returns an error if no slot is available.
    ///
    /// [`IntoJulia`]: ../traits/trait.IntoJulia.html
    pub fn new<'base: 'frame, V: IntoJulia, F: Frame<'base, 'frame>>(
        frame: &mut F,
        value: V,
    ) -> JlrsResult<Value<'frame, 'static>> {
        unsafe { frame.protect(value.into_julia(Internal), Internal) }
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

    /// Allocates a new n-dimensional array in Julia.
    ///
    /// Allocating an array with one, two, or three dimensions requires one slot on the GC stack.
    /// If you allocate an array with more dimensions, an extra frame is created with `n + 1`
    /// slots, temporarily taking `n + 3` additional slots. This latter case requires that
    /// `jlrs.jl` has been included.
    ///
    /// This function returns an error if there are not enough slots available, or if `jlrs.jl`
    /// has not been included when allocating arrays with four or more dimensions.
    pub fn array<'base: 'frame, T: JuliaType, D: Into<Dimensions>, F: Frame<'base, 'frame>>(
        frame: &mut F,
        dimensions: D,
    ) -> JlrsResult<Value<'frame, 'static>> {
        unsafe {
            let dims = dimensions.into();

            let array_type = jl_apply_array_type(T::julia_type(Internal), dims.n_dimensions() as _);
            let array = match dims.n_dimensions() {
                1 => jl_alloc_array_1d(array_type, dims.n_elements(0) as _),
                2 => {
                    jl_alloc_array_2d(array_type, dims.n_elements(0) as _, dims.n_elements(1) as _)
                }
                3 => jl_alloc_array_3d(
                    array_type,
                    dims.n_elements(0) as _,
                    dims.n_elements(1) as _,
                    dims.n_elements(2) as _,
                ),
                n => frame.frame(n as usize + 1, |frame| {
                    let func = Module::main(frame)
                        .submodule("Jlrs")?
                        .function("arraydims")?;

                    let v = Values::new(frame, dims.as_slice())?;
                    let dims = func.call_values(frame, v)?;
                    Ok(jl_new_array(array_type, dims.0))
                })?,
            };

            frame.protect(array as _, Internal)
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
    pub unsafe fn borrow_array<
        'base: 'frame,
        T: JuliaType,
        D: Into<Dimensions>,
        V: AsMut<[T]>,
        F: Frame<'base, 'frame>,
    >(
        frame: &mut F,
        data: &'borrow mut V,
        dimensions: D,
    ) -> JlrsResult<Value<'frame, 'borrow>> {
        let dims = dimensions.into();

        let array_type = jl_apply_array_type(T::julia_type(Internal), dims.n_dimensions() as _);
        let array = match dims.n_dimensions() {
            1 => jl_ptr_to_array_1d(
                array_type,
                data.as_mut().as_mut_ptr() as _,
                dims.n_elements(0) as _,
                0,
            ),
            n => frame.frame(n as usize + 1, |frame| {
                let func = Module::main(frame)
                    .submodule("Jlrs")?
                    .function("arraydims")?;
                let v = Values::new(frame, dims.as_slice())?;
                let dims = func.call_values(frame, v)?;
                Ok(jl_ptr_to_array(array_type, data.as_mut().as_mut_ptr() as _, dims.0, 0) as _)
            })?,
        };

        frame.protect(array as _, Internal)
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
    pub fn move_array<'base: 'frame, T: JuliaType, D: Into<Dimensions>, F: Frame<'base, 'frame>>(
        frame: &mut F,
        data: Vec<T>,
        dimensions: D,
    ) -> JlrsResult<Value<'frame, 'static>> {
        unsafe {
            let dims = dimensions.into();

            let array_type = jl_apply_array_type(T::julia_type(Internal), dims.n_dimensions() as _);
            let array = match dims.n_dimensions() {
                1 => jl_ptr_to_array_1d(
                    array_type,
                    Box::into_raw(data.into_boxed_slice()) as _,
                    dims.n_elements(0) as _,
                    1,
                ),
                n => frame.frame(n as usize + 1, |frame| {
                    let func = Module::main(frame)
                        .submodule("Jlrs")?
                        .function("arraydims")?;
                    let v = Values::new(frame, dims.as_slice())?;
                    let dims = func.call_values(frame, v)?;
                    Ok(jl_ptr_to_array(
                        array_type,
                        Box::into_raw(data.into_boxed_slice()) as _,
                        dims.0,
                        1,
                    ) as _)
                })?,
            };

            frame.protect(array as _, Internal)
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
    pub fn try_unbox<'base: 'frame, T: TryUnbox>(self) -> JlrsResult<T> {
        unsafe { T::try_unbox(self.0, Internal) }
    }

    /// Call this value as a function that takes zero arguments, this takes one slot on the GC
    /// stack. Returns the result of this function call if no exception is thrown, the exception
    /// if one is, or an error if no space is left on the stack.
    pub fn call0<'base: 'frame, F: Frame<'base, 'frame>>(
        self,
        frame: &mut F,
    ) -> JlrsResult<Value<'frame, 'static>> {
        unsafe {
            let res = jl_call0(self.0 as _);
            convert_exception()?;
            frame.protect(res as _, Internal)
        }
    }

    /// Call this value as a function that takes zero arguments and use the `Output` to extend the
    /// result's lifetime. This takes no space on the GC stack. Returns the result of this
    /// function call if no exception is thrown or the exception if one is.
    pub fn call0_output<'output, 'base: 'frame, F: Frame<'base, 'frame>>(
        self,
        frame: &mut F,
        output: Output<'output>,
    ) -> JlrsResult<Value<'output, 'static>> {
        unsafe {
            let res = jl_call0(self.0 as _);
            convert_exception()?;
            Ok(frame.assign_output(output, res as _, Internal))
        }
    }

    /// Call this value as a function that takes one argument, this takes one slot on the GC
    /// stack. Returns the result of this function call if no exception is thrown, the exception
    /// if one is, or an error if no space is left on the stack.
    pub fn call1<'base: 'frame, F: Frame<'base, 'frame>>(
        self,
        frame: &mut F,
        arg: Value,
    ) -> JlrsResult<Value<'frame, 'static>> {
        unsafe {
            let res = jl_call1(self.0 as _, arg.0 as _);
            convert_exception()?;
            frame.protect(res as _, Internal)
        }
    }

    /// Call this value as a function that takes one argument and use the `Output` to extend the
    /// result's lifetime. This takes no space on the GC stack. Returns the result of this
    /// function call if no exception is thrown or the exception if one is.
    pub fn call1_output<'output, 'base: 'frame, F: Frame<'base, 'frame>>(
        self,
        frame: &mut F,
        output: Output<'output>,
        arg: Value,
    ) -> JlrsResult<Value<'output, 'static>> {
        unsafe {
            let res = jl_call1(self.0 as _, arg.0 as _);
            convert_exception()?;
            Ok(frame.assign_output(output, res as _, Internal))
        }
    }

    /// Call this value as a function that takes two arguments, this takes one slot on the GC
    /// stack. Returns the result of this function call if no exception is thrown, the exception
    /// if one is, or an error if no space is left on the stack.
    pub fn call2<'base: 'frame, F: Frame<'base, 'frame>>(
        self,
        frame: &mut F,
        arg0: Value,
        arg1: Value,
    ) -> JlrsResult<Value<'frame, 'static>> {
        unsafe {
            let res = jl_call2(self.0 as _, arg0.0 as _, arg1.0 as _);
            convert_exception()?;
            frame.protect(res as _, Internal)
        }
    }

    /// Call this value as a function that takes two arguments and use the `Output` to extend the
    /// result's lifetime. This takes no space on the GC stack. Returns the result of this
    /// function call if no exception is thrown or the exception if one is.
    pub fn call2_output<'output, 'base: 'frame, F: Frame<'base, 'frame>>(
        self,
        frame: &mut F,
        output: Output<'output>,
        arg0: Value,
        arg1: Value,
    ) -> JlrsResult<Value<'output, 'static>> {
        unsafe {
            let res = jl_call2(self.0 as _, arg0.0 as _, arg1.0 as _);
            convert_exception()?;
            Ok(frame.assign_output(output, res as _, Internal))
        }
    }

    /// Call this value as a function that takes three arguments, this takes one slot on the GC
    /// stack. Returns the result of this function call if no exception is thrown, the exception
    /// if one is, or an error if no space is left on the stack.
    pub fn call3<'base: 'frame, F: Frame<'base, 'frame>>(
        self,
        frame: &mut F,
        arg0: Value,
        arg1: Value,
        arg2: Value,
    ) -> JlrsResult<Value<'frame, 'static>> {
        unsafe {
            let res = jl_call3(self.0 as _, arg0.0 as _, arg1.0 as _, arg2.0 as _);
            convert_exception()?;
            frame.protect(res as _, Internal)
        }
    }

    /// Call this value as a function that takes three arguments and use the `Output` to extend
    /// the result's lifetime. This takes no space on the GC stack. Returns the result of this
    /// function call if no exception is thrown or the exception if one is.
    pub fn call3_output<'output, 'base: 'frame, F: Frame<'base, 'frame>>(
        self,
        frame: &mut F,
        output: Output<'output>,
        arg0: Value,
        arg1: Value,
        arg2: Value,
    ) -> JlrsResult<Value<'output, 'static>> {
        unsafe {
            let res = jl_call3(self.0 as _, arg0.0 as _, arg1.0 as _, arg2.0 as _);
            convert_exception()?;
            Ok(frame.assign_output(output, res as _, Internal))
        }
    }

    /// Call this value as a function that takes several arguments, this takes one slot on the GC
    /// stack. Returns the result of this function call if no exception is thrown, the exception
    /// if one is, or an error if no space is left on the stack.
    pub fn call<'f, 'b, 'base: 'frame, V: AsRef<[Value<'f, 'b>]>, F: Frame<'base, 'frame>>(
        self,
        frame: &mut F,
        args: V,
    ) -> JlrsResult<Value<'frame, 'static>> {
        unsafe {
            let args = args.as_ref();
            let n = args.len();
            let res = jl_call(self.0 as _, args.as_ptr() as _, n as _);
            convert_exception()?;
            frame.protect(res as _, Internal)
        }
    }

    /// Call this value as a function that takes several arguments and use the `Output` to extend
    /// the result's lifetime. This takes no space on the GC stack. Returns the result of this
    /// function call if no exception is thrown or the exception if one is.
    pub fn call_output<
        'output,
        'value,
        'b,
        'base: 'frame,
        V: AsRef<[Value<'value, 'b>]>,
        F: Frame<'base, 'frame>,
    >(
        self,
        frame: &mut F,
        output: Output<'output>,
        args: V,
    ) -> JlrsResult<Value<'output, 'static>> {
        unsafe {
            let args = args.as_ref();
            let n = args.len();
            let res = jl_call(self.0 as _, args.as_ptr() as _, n as _);
            convert_exception()?;
            Ok(frame.assign_output(output, res as _, Internal))
        }
    }

    /// Call this value as a function that takes several arguments in a single `Values`, this
    /// takes one slot on the GC stack. Returns the result of this function call if no exception
    /// is thrown, the exception if one is, or an error if no space is left on the stack.
    pub fn call_values<'base: 'frame, F: Frame<'base, 'frame>>(
        self,
        frame: &mut F,
        args: Values,
    ) -> JlrsResult<Value<'frame, 'static>> {
        unsafe {
            let res = jl_call(self.0 as _, args.0, args.1 as _);
            convert_exception()?;
            frame.protect(res as _, Internal)
        }
    }

    /// Call this value as a function that takes several arguments in a single `Values` and use
    /// the `Output` to extend the result's lifetime. This takes no space on the GC stack. Returns
    /// the result of this function call if no exception is thrown or the exception if one is.
    pub fn call_values_output<'output, 'base: 'frame, F: Frame<'base, 'frame>>(
        self,
        frame: &mut F,
        output: Output<'output>,
        args: Values,
    ) -> JlrsResult<Value<'output, 'static>> {
        unsafe {
            let res = jl_call(self.0 as _, args.0, args.1 as _);
            convert_exception()?;
            Ok(frame.assign_output(output, res as _, Internal))
        }
    }
}

/// An `Output` is a slot on the GC stack in the frame that was used to create it. It can be used
/// to extend the lifetime of the result of a function call to the `Output`'s lifetime.
pub struct Output<'frame> {
    pub(crate) offset: usize,
    _marker: PhantomData<&'frame ()>,
}

impl<'frame> Output<'frame> {
    pub(crate) unsafe fn new(offset: usize) -> Self {
        Output {
            offset,
            _marker: PhantomData,
        }
    }
}

unsafe fn convert_exception() -> JlrsResult<()> {
    let exc = jl_exception_occurred();
    if !exc.is_null() {
        let exc = Exception::new(jl_typeof_str(exc));
        return Err(JlrsError::ExceptionOccurred(exc).into());
    }

    Ok(())
}
