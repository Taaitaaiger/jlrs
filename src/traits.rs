//! Every trait used by this crate.

use crate::context::{AllocationContext, ExecutionContext};
use crate::error::{Exception, JlrsError, JlrsResult};
use crate::handles::{AssignedHandle, PrimitiveHandles, UnassignedHandle};
use crate::pending::primitive::Primitive;
use jl_sys::{
    jl_bool_type, jl_call, jl_call0, jl_call1, jl_call2, jl_call3, jl_char_type,
    jl_exception_occurred, jl_float32_type, jl_float64_type, jl_int16_type, jl_int32_type,
    jl_int64_type, jl_int8_type, jl_is_string, jl_pchar_to_string, jl_string_data, jl_string_len,
    jl_typeis, jl_typeof_str, jl_uint16_type, jl_uint32_type, jl_uint64_type, jl_uint8_type,
    jl_unbox_float32, jl_unbox_float64, jl_unbox_int16, jl_unbox_int32, jl_unbox_int64,
    jl_unbox_int8, jl_unbox_uint16, jl_unbox_uint32, jl_unbox_uint64, jl_unbox_uint8, jl_value_t,
};
use private::Sealed;
use std::mem::size_of;
use std::slice;

mod private {
    use crate::handles::{AssignedHandle, BorrowedArrayHandle, GlobalHandle, UninitArrayHandle};
    use crate::unboxed_array::UnboxedArray;

    pub trait Sealed {}
    impl Sealed for bool {}
    impl Sealed for char {}
    impl Sealed for u8 {}
    impl Sealed for u16 {}
    impl Sealed for u32 {}
    impl Sealed for u64 {}
    impl Sealed for usize {}
    impl Sealed for i8 {}
    impl Sealed for i16 {}
    impl Sealed for i32 {}
    impl Sealed for i64 {}
    impl Sealed for isize {}
    impl Sealed for f32 {}
    impl Sealed for f64 {}
    impl Sealed for String {}
    impl<T> Sealed for UnboxedArray<T> {}
    impl<'scope> Sealed for AssignedHandle<'scope> {}
    impl<'scope, T> Sealed for UninitArrayHandle<'scope, T> {}
    impl<'scope, 'borrow> Sealed for BorrowedArrayHandle<'scope, 'borrow> {}
    impl<'scope> Sealed for GlobalHandle<'scope> {}
}

/// Implemented by handles that can be used as function arguments.
pub trait ValidHandle: Sealed {
    #[doc(hidden)]
    unsafe fn get_value(&self, context: &ExecutionContext) -> *mut jl_value_t;
}

/// Implemented by valid handles whose contents you can try to copy into Rust, ie all handles that
/// implement [`ValidHandle`] except [`UninitArrayHandle`].
///
/// [`ValidHandle`]: trait.ValidHandle.html
/// [`UninitArrayHandle`]: ../handles/struct.UninitArrayHandle.html
pub trait UnboxableHandle: ValidHandle {}

/// Implemented by types that can be easily copied into Julia. This includes all scalar types
/// mentioned in the Rust Book with the exception of `u128` and `i128`.
pub trait IntoPrimitive: Sealed {
    #[doc(hidden)]
    fn into_primitive(&self) -> Primitive;
}

/// Implemented by types that have "matching types" in Rust and Julia, ie all types that implement
/// [`IntoPrimitive`].
///
/// [`IntoPrimitive`]: trait.IntoPrimitive.html
pub trait JuliaType: IntoPrimitive {
    #[doc(hidden)]
    unsafe fn julia_type() -> *mut jl_value_t;
}

/// Implemented by types that can copied from Julia to Rust, ie all types that implement
/// [`IntoPrimitive`] except `bool` and `char`, which can be unboxed as `i8` and `u32`
/// respectively.
///
/// [`IntoPrimitive`]: trait.IntoPrimitive.html
pub trait Unboxable: JuliaType {}

/// Implemented by types that Julia data can be unboxed into. This is currently restricted to all
/// types that implement [`IntoPrimitive`] and [`UnboxedArray`].
///
/// [`IntoPrimitive`]: trait.IntoPrimitive.html
/// [`UnboxedArray`]: ../unboxed_array/struct.UnboxedArray.html
pub trait TryUnbox: Sealed
where
    Self: Sized,
{
    #[doc(hidden)]
    fn try_unbox(value: *mut jl_value_t) -> JlrsResult<Self>;
}

/// Implemented by handles that can be called as functions.
pub trait Call: ValidHandle {
    /// Use `self` as a Julia function and call it with no arguments. The handle you get when this
    /// call succeeds is a handle to its result, it will have the same lifetime constraints as the
    /// [`UnassignedHandle`] that is used.
    ///
    /// [`UnassignedHandle`]: ../handles/struct.UnassignedHandle.html
    fn call0<'handle>(
        &self,
        context: &mut ExecutionContext,
        handle: UnassignedHandle<'handle>,
    ) -> JlrsResult<AssignedHandle<'handle>> {
        // Safe because all handles are valid
        unsafe {
            let func = self.get_value(context);
            let res = jl_call0(func);
            let exc = jl_exception_occurred();
            if !exc.is_null() {
                let exc = Exception::new(jl_typeof_str(exc));
                return Err(JlrsError::ExceptionOccurred(exc).into());
            }
            Ok(handle.assign(context, res))
        }
    }

    /// Use `self` as a Julia function and call it with one argument. The handle you get when this
    /// call succeeds is a handle to its result, it will have the same lifetime constraints as the
    /// [`UnassignedHandle`] that is used.
    ///
    /// [`UnassignedHandle`]: ../handles/struct.UnassignedHandle.html
    fn call1<'handle, H: ValidHandle>(
        &self,
        context: &mut ExecutionContext,
        handle: UnassignedHandle<'handle>,
        arg: H,
    ) -> JlrsResult<AssignedHandle<'handle>> {
        // Safe because all handles are valid
        unsafe {
            let func = self.get_value(context);
            let res = jl_call1(func, arg.get_value(context));
            let exc = jl_exception_occurred();
            if !exc.is_null() {
                let exc = Exception::new(jl_typeof_str(exc));
                return Err(JlrsError::ExceptionOccurred(exc).into());
            }
            Ok(handle.assign(context, res))
        }
    }

    /// Use `self` as a Julia function and call it with two arguments. The handle you get when this
    /// call succeeds is a handle to its result, it will have the same lifetime constraints as the
    /// [`UnassignedHandle`] that is used.
    ///
    /// [`UnassignedHandle`]: ../handles/struct.UnassignedHandle.html
    fn call2<'handle, H0: ValidHandle, H1: ValidHandle>(
        &self,
        context: &mut ExecutionContext,
        handle: UnassignedHandle<'handle>,
        arg0: H0,
        arg1: H1,
    ) -> JlrsResult<AssignedHandle<'handle>> {
        // Safe because all handles are valid
        unsafe {
            let func = self.get_value(context);
            let res = jl_call2(func, arg0.get_value(context), arg1.get_value(context));
            let exc = jl_exception_occurred();
            if !exc.is_null() {
                let exc = Exception::new(jl_typeof_str(exc));
                return Err(JlrsError::ExceptionOccurred(exc).into());
            }
            Ok(handle.assign(context, res))
        }
    }

    /// Use `self` as a Julia function and call it with three arguments. The handle you get when
    /// this call succeeds is a handle to its result, it will have the same lifetime constraints as
    /// the [`UnassignedHandle`] that is used.
    ///
    /// [`UnassignedHandle`]: ../handles/struct.UnassignedHandle.html
    fn call3<'handle, H0: ValidHandle, H1: ValidHandle, H2: ValidHandle>(
        &self,
        context: &mut ExecutionContext,
        handle: UnassignedHandle<'handle>,
        arg0: H0,
        arg1: H1,
        arg2: H2,
    ) -> JlrsResult<AssignedHandle<'handle>> {
        // Safe because all handles are valid
        unsafe {
            let func = self.get_value(context);
            let res = jl_call3(
                func,
                arg0.get_value(context),
                arg1.get_value(context),
                arg2.get_value(context),
            );
            let exc = jl_exception_occurred();
            if !exc.is_null() {
                let exc = Exception::new(jl_typeof_str(exc));
                return Err(JlrsError::ExceptionOccurred(exc).into());
            }
            Ok(handle.assign(context, res))
        }
    }

    /// Use `self` as a Julia function and call it with any number of arguments with different
    /// handle types. The handle you get when this call succeeds is a handle to its result, it will
    /// have the same lifetime constraints as the [`UnassignedHandle`] that is used.
    ///
    /// [`UnassignedHandle`]: ../handles/struct.UnassignedHandle.html
    fn call_dyn<'handle, 'input, A: AsRef<[&'input dyn ValidHandle]>>(
        &self,
        context: &mut ExecutionContext,
        handle: UnassignedHandle<'handle>,
        args: A,
    ) -> JlrsResult<AssignedHandle<'handle>> {
        // Safe because all handles are valid
        unsafe {
            let mut raw = args
                .as_ref()
                .iter()
                .map(|a| a.get_value(context))
                .collect::<Vec<_>>();
            let nargs = raw.len() as i32;
            let func = self.get_value(context);
            let res = jl_call(func, raw.as_mut_ptr(), nargs);
            let exc = jl_exception_occurred();
            if !exc.is_null() {
                let exc = Exception::new(jl_typeof_str(exc));
                return Err(JlrsError::ExceptionOccurred(exc).into());
            }
            Ok(handle.assign(context, res))
        }
    }

    /// Use `self` as a Julia function and call it with any number of primitive arguments that have
    /// been allocated as a group. The handle you get when this call succeeds is a handle to its
    /// result, it will have the same lifetime constraints as the [`UnassignedHandle`] that is
    /// used.
    ///
    /// [`UnassignedHandle`]: ../handles/struct.UnassignedHandle.html
    fn call_primitives<'handle>(
        &self,
        context: &mut ExecutionContext,
        handle: UnassignedHandle<'handle>,
        args: PrimitiveHandles,
    ) -> JlrsResult<AssignedHandle<'handle>> {
        // Safe because all handles are valid
        unsafe {
            let func = self.get_value(context);
            let values = context.get_values(args.index(), args.len());
            let res = jl_call(func, values, args.len() as i32);
            let exc = jl_exception_occurred();
            if !exc.is_null() {
                let exc = Exception::new(jl_typeof_str(exc));
                return Err(JlrsError::ExceptionOccurred(exc).into());
            }
            Ok(handle.assign(context, res))
        }
    }

    /// Use `self` as a Julia function and call it with any number of arguments with the same
    /// handle type. The handle you get when this call succeeds is a handle to its result, it will
    /// have the same lifetime constraints as the [`UnassignedHandle`] that is used.
    ///
    /// [`UnassignedHandle`]: ../handles/struct.UnassignedHandle.html
    fn call<'handle, A: AsRef<[impl ValidHandle]>>(
        &self,
        context: &mut ExecutionContext,
        handle: UnassignedHandle<'handle>,
        args: A,
    ) -> JlrsResult<AssignedHandle<'handle>> {
        // Safe because all handles are valid
        unsafe {
            let mut raw = args
                .as_ref()
                .iter()
                .map(|a| a.get_value(context))
                .collect::<Vec<_>>();
            let nargs = raw.len() as i32;

            let func = self.get_value(context);
            let res = jl_call(func, raw.as_mut_ptr(), nargs);
            let exc = jl_exception_occurred();
            if !exc.is_null() {
                let exc = Exception::new(jl_typeof_str(exc));
                return Err(JlrsError::ExceptionOccurred(exc).into());
            }
            Ok(handle.assign(context, res))
        }
    }
}

pub(crate) trait Allocate {
    unsafe fn allocate(&self, context: AllocationContext) -> JlrsResult<*mut jl_value_t>;
}

macro_rules! impl_julia_type {
    ($type:ty, $jl_type:expr) => {
        impl JuliaType for $type {
            unsafe fn julia_type() -> *mut jl_value_t {
                $jl_type as _
            }
        }
    };
}

impl_julia_type!(u8, jl_uint8_type);
impl_julia_type!(u16, jl_uint16_type);
impl_julia_type!(u32, jl_uint32_type);
impl_julia_type!(u64, jl_uint64_type);
impl_julia_type!(i8, jl_int8_type);
impl_julia_type!(i16, jl_int16_type);
impl_julia_type!(i32, jl_int32_type);
impl_julia_type!(i64, jl_int64_type);
impl_julia_type!(f32, jl_float32_type);
impl_julia_type!(f64, jl_float64_type);
impl_julia_type!(bool, jl_bool_type);
impl_julia_type!(char, jl_char_type);

impl JuliaType for usize {
    unsafe fn julia_type() -> *mut jl_value_t {
        if size_of::<usize>() == size_of::<u32>() {
            jl_uint32_type as _
        } else {
            jl_uint64_type as _
        }
    }
}

impl JuliaType for isize {
    unsafe fn julia_type() -> *mut jl_value_t {
        if size_of::<isize>() == size_of::<i32>() {
            jl_int32_type as _
        } else {
            jl_int64_type as _
        }
    }
}

macro_rules! impl_try_unbox {
    ($type:ty, $jl_type:expr, $unboxer:path) => {
        impl TryUnbox for $type {
            fn try_unbox(value: *mut jl_value_t) -> JlrsResult<Self> {
                unsafe {
                    if jl_typeis(value, $jl_type) {
                        return Ok($unboxer(value));
                    }
                    Err(JlrsError::WrongType.into())
                }
            }
        }
    };
}

impl_try_unbox!(u8, jl_uint8_type, jl_unbox_uint8);
impl_try_unbox!(u16, jl_uint16_type, jl_unbox_uint16);
impl_try_unbox!(u32, jl_uint32_type, jl_unbox_uint32);
impl_try_unbox!(u64, jl_uint64_type, jl_unbox_uint64);
impl_try_unbox!(i8, jl_int8_type, jl_unbox_int8);
impl_try_unbox!(i16, jl_int16_type, jl_unbox_int16);
impl_try_unbox!(i32, jl_int32_type, jl_unbox_int32);
impl_try_unbox!(i64, jl_int64_type, jl_unbox_int64);
impl_try_unbox!(f32, jl_float32_type, jl_unbox_float32);
impl_try_unbox!(f64, jl_float64_type, jl_unbox_float64);

impl TryUnbox for bool {
    fn try_unbox(value: *mut jl_value_t) -> JlrsResult<Self> {
        unsafe {
            if jl_typeis(value, jl_bool_type) {
                return Ok(jl_unbox_int8(value) != 0);
            }
            Err(JlrsError::WrongType.into())
        }
    }
}

impl TryUnbox for char {
    fn try_unbox(value: *mut jl_value_t) -> JlrsResult<Self> {
        unsafe {
            if jl_typeis(value, jl_char_type) {
                return std::char::from_u32(jl_unbox_uint32(value))
                    .ok_or(JlrsError::InvalidCharacter.into());
            }

            Err(JlrsError::WrongType.into())
        }
    }
}

impl TryUnbox for usize {
    fn try_unbox(value: *mut jl_value_t) -> JlrsResult<Self> {
        unsafe {
            if size_of::<usize>() == size_of::<u32>() {
                if jl_typeis(value, jl_uint32_type) {
                    return Ok(jl_unbox_uint32(value) as usize);
                }
            } else {
                if jl_typeis(value, jl_uint64_type) {
                    return Ok(jl_unbox_uint64(value) as usize);
                }
            }
            Err(JlrsError::WrongType.into())
        }
    }
}

impl TryUnbox for isize {
    fn try_unbox(value: *mut jl_value_t) -> JlrsResult<Self> {
        unsafe {
            if size_of::<isize>() == size_of::<i32>() {
                if jl_typeis(value, jl_int32_type) {
                    return Ok(jl_unbox_int32(value) as isize);
                }
            } else {
                if jl_typeis(value, jl_int64_type) {
                    return Ok(jl_unbox_int64(value) as isize);
                }
            }
            Err(JlrsError::WrongType.into())
        }
    }
}

impl TryUnbox for String {
    fn try_unbox(value: *mut jl_value_t) -> JlrsResult<String> {
        unsafe {
            if !jl_is_string(value) {
                return Err(JlrsError::NotAString.into());
            }

            let len = jl_string_len(value);

            if len == 0 {
                return Ok(String::new());
            }

            // Is neither null nor dangling, we've just checked
            let raw = jl_string_data(value);
            let raw_slice = slice::from_raw_parts(raw, len);
            let owned_slice = Vec::from(raw_slice);
            Ok(String::from_utf8(owned_slice)?)
        }
    }
}

impl Unboxable for u8 {}
impl Unboxable for u16 {}
impl Unboxable for u32 {}
impl Unboxable for u64 {}
impl Unboxable for usize {}
impl Unboxable for i8 {}
impl Unboxable for i16 {}
impl Unboxable for i32 {}
impl Unboxable for i64 {}
impl Unboxable for isize {}
impl Unboxable for f32 {}
impl Unboxable for f64 {}

impl Allocate for String {
    unsafe fn allocate(&self, _: AllocationContext) -> JlrsResult<*mut jl_value_t> {
        let ptr = self.as_ptr() as _;
        let len = self.len();
        Ok(jl_pchar_to_string(ptr, len as _))
    }
}
