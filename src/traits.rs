//! All traits used by the public API of this crate.
//!
//! A quick note: all of these traits are sealed, you can't implement them on your own types. Most
//! serve as marker traits that are used as trait bounds on some methods to enforce you can only
//! do things that make sense. The one trait that you will use directly is the [`Frame`] trait
//! which exposes methods that let you create [`Output`]s in the current frame, and nest frames.
//! This trait is automatically brought into scope by using the [`prelude`] module.
//!
//! [`Frame`]: trait.Frame.html
//! [`Output`]: ../value/struct.Output.html
//! [`prelude`]: ../prelude/index.html

use crate::array::Array;
use crate::error::{JlrsError, JlrsResult};
use crate::frame::{DynamicFrame, StaticFrame, Scope};
use crate::value::Output;
use std::borrow::Cow;
use std::marker::PhantomData;

// All these traits have a public and a private side. In order to prevent users from using methods
// that are for internal use only, all traits extend a base trait with the same name from the
// crate-public private module that contains the methods intended for private use and serves as a
// sealing trait. In order to retain information about what types implement a trait in the docs,
// the public trait is explicitly implemented for each type that implements the associated private
// trait rather than with a single blanket implementation.
macro_rules! p {
    ($trait:ident, $type:ty, $($bounds:tt)+) => {
        impl<$($bounds)+> $trait for $type {}
    };
    ($trait:ident, $type:ty) => {
        impl $trait for $type {}
    };
}

/// Trait implemented by types that can be converted to a Julia value.
pub trait IntoJulia: private::IntoJulia {}

/// Trait implemented by types that have an associated type in Julia.
pub trait JuliaType: private::JuliaType {}

/// Trait implemented by types that have the same representation in Julia and Rust when they are
/// used as array data. Arrays whose elements are of a type that implements this trait can share
/// their contents between Julia and Rust. This includes all types that implement `JuliaType`
/// except `bool` and `char`.
pub trait ArrayData: private::ArrayData {}

/// Trait implemented by types that can be created from a Julia value.
pub trait TryUnbox: private::TryUnbox {}

/// Functionality shared by [`StaticFrame`] and [`DynamicFrame`].
///
/// Frames use two lifetimes, `'base` and `'frame`. The first is used to ensure global Julia
/// values, like [`Module`]s and functions accessed through modules, can be used freely across
/// frames, but can't be returned from the frame created through [`Runtime::frame`] or
/// [`Runtime::dynamic_frame`]. Other values get the `'frame` lifetime which ensures those values
/// can live as long as their frames.
///
/// If you need the result of a function call to be valid outside the frame where it is called,
/// you can call `Frame::output` to create an [`Output`] and call the function through
/// [`Value::call_output`] or one of the other `call*_output` methods. The result will share the
/// output's lifetime so it can be used until the output's frame goes out of scope.
///
/// [`StaticFrame`]: ../frame/struct.StaticFrame.html
/// [`DynamicFrame`]: ../frame/struct.DynamicFrame.html
/// [`Module`]: ../module/struct.Module.html
/// [`Runtime::frame`]: ../struct.Runtime.html#method.frame
/// [`Runtime::dynamic_frame`]: ../struct.Runtime.html#method.dynamic_frame
/// [`Output`]: ../value/struct.Output.html
/// [`Value::call_output`]: ../value/struct.Value.html#method.call_output
pub trait Frame<'base: 'frame, 'frame>: private::Frame<'base, 'frame> {
    /// Create a `StaticFrame` that can hold `capacity` values, and call the given closure.
    /// Returns the result of this closure, or an error if the new frame can't be created
    /// because there's not enough space on the GC stack. The number of required slots on the
    /// stack is `capacity + 2`.
    ///
    /// Returns an error if there is not enough space on the stack.
    fn frame<T, F: FnOnce(&mut StaticFrame<'base, '_>) -> JlrsResult<T>>(
        &mut self,
        capacity: usize,
        func: F,
    ) -> JlrsResult<T>;

    /// Create a `DynamicFrame` and call the given closure.  Returns the result of this closure,
    /// or an error if the new frame can't be created because the stack is too small. The number
    /// of required slots on the stack is `2`.
    ///
    /// Returns an error if there is not enough space on the stack.
    fn dynamic_frame<T, F: FnOnce(&mut DynamicFrame<'base, '_>) -> JlrsResult<T>>(
        &mut self,
        func: F,
    ) -> JlrsResult<T>;

    /// Returns a new `Output`, this takes one slot on the GC stack. A function that uses this
    /// output will not use a slot on the GC stack, but the one associated with this output. This
    /// extends the lifetime of that value to be valid until the frame that created the output
    /// goes out of scope.
    ///
    /// Returns an error if there is not enough space on the stack.
    fn output(&mut self) -> JlrsResult<Output<'frame>>;

    /// Returns the number of values belonging to this frame.
    fn size(&self) -> usize;

    #[doc(hidden)]
    // Exists for debugging purposes, prints the contents of the GC stack.
    fn print_memory(&self);
}

p!(IntoJulia, bool);
p!(IntoJulia, char);
p!(IntoJulia, u8);
p!(IntoJulia, u16);
p!(IntoJulia, u32);
p!(IntoJulia, u64);
p!(IntoJulia, i8);
p!(IntoJulia, i16);
p!(IntoJulia, i32);
p!(IntoJulia, i64);
p!(IntoJulia, f32);
p!(IntoJulia, f64);
p!(IntoJulia, usize);
p!(IntoJulia, isize);
p!(IntoJulia, String);
p!(IntoJulia, &dyn AsRef<str>);
p!(IntoJulia, &'a str, 'a);
p!(IntoJulia, Cow<'a, str>, 'a);

p!(JuliaType, bool);
p!(JuliaType, char);
p!(JuliaType, u8);
p!(JuliaType, u16);
p!(JuliaType, u32);
p!(JuliaType, u64);
p!(JuliaType, i8);
p!(JuliaType, i16);
p!(JuliaType, i32);
p!(JuliaType, i64);
p!(JuliaType, f32);
p!(JuliaType, f64);
p!(JuliaType, usize);
p!(JuliaType, isize);

p!(ArrayData, u8);
p!(ArrayData, u16);
p!(ArrayData, u32);
p!(ArrayData, u64);
p!(ArrayData, i8);
p!(ArrayData, i16);
p!(ArrayData, i32);
p!(ArrayData, i64);
p!(ArrayData, f32);
p!(ArrayData, f64);
p!(ArrayData, usize);
p!(ArrayData, isize);

p!(TryUnbox, bool);
p!(TryUnbox, char);
p!(TryUnbox, u8);
p!(TryUnbox, u16);
p!(TryUnbox, u32);
p!(TryUnbox, u64);
p!(TryUnbox, i8);
p!(TryUnbox, i16);
p!(TryUnbox, i32);
p!(TryUnbox, i64);
p!(TryUnbox, f32);
p!(TryUnbox, f64);
p!(TryUnbox, usize);
p!(TryUnbox, isize);
p!(TryUnbox, String);
p!(TryUnbox, Array<A>, A: ArrayData);

impl<'base: 'frame, 'frame> Frame<'base, 'frame> for StaticFrame<'base, 'frame> {
    fn frame<T, F: FnOnce(&mut StaticFrame<'base, '_>) -> JlrsResult<T>>(
        &mut self,
        capacity: usize,
        func: F,
    ) -> JlrsResult<T> {
        let mut scope = Scope;
        let mut frame = unsafe { self.nested_frame(capacity, &mut scope).unwrap() };
        func(&mut frame)
    }

    fn dynamic_frame<T, F: FnOnce(&mut DynamicFrame<'base, '_>) -> JlrsResult<T>>(
        &mut self,
        func: F,
    ) -> JlrsResult<T> {
        unsafe {
            let mut scope = Scope;
            let mut view = self.memory.nest_dynamic(&mut scope);
            let idx = view.new_frame()?;
            let mut frame = DynamicFrame {
                idx,
                len: 0,
                memory: view,
                _guard: PhantomData,
            };

            func(&mut frame)
        }
    }

    fn output(&mut self) -> JlrsResult<Output<'frame>> {
        if self.capacity == self.len {
            return Err(JlrsError::FrameSizeExceeded(self.len).into());
        }

        let out = unsafe {
            let out = self.memory.new_output(self.idx, self.len);
            self.len += 1;
            out
        };

        Ok(out)
    }

    fn size(&self) -> usize {
        self.len
    }

    fn print_memory(&self) {
        self.memory.print_memory()
    }
}

impl<'base: 'frame, 'frame> Frame<'base, 'frame> for DynamicFrame<'base, 'frame> {
    fn dynamic_frame<T, F: FnOnce(&mut DynamicFrame<'base, '_>) -> JlrsResult<T>>(
        &mut self,
        func: F,
    ) -> JlrsResult<T> {
        let mut scope = Scope;
        let mut frame = unsafe { self.nested_frame(&mut scope).unwrap() };
        func(&mut frame)
    }

    fn frame<T, F: FnOnce(&mut StaticFrame<'base, '_>) -> JlrsResult<T>>(
        &mut self,
        capacity: usize,
        func: F,
    ) -> JlrsResult<T> {
        unsafe {
            let mut scope = Scope;
            let mut view = self.memory.nest_static(&mut scope);
            let idx = view.new_frame(capacity)?;
            let mut frame = StaticFrame {
                idx,
                capacity,
                len: 0,
                memory: view,
                _guard: PhantomData,
            };

            func(&mut frame)
        }
    }

    fn output(&mut self) -> JlrsResult<Output<'frame>> {
        unsafe {
            let out = self.memory.new_output(self.idx)?;
            self.len += 1;
            Ok(out)
        }
    }

    fn size(&self) -> usize {
        self.len
    }

    fn print_memory(&self) {
        self.memory.print_memory()
    }
}

pub(crate) mod private {
    use crate::array::Array;
    use crate::array::Dimensions;
    use crate::error::{JlrsError, JlrsResult};
    use crate::frame::{DynamicFrame, StaticFrame};
    use crate::memory::FrameIdx;
    use crate::value::{Output, Value, Values};
    use jl_sys::{
        jl_array_data, jl_array_dim, jl_array_dims, jl_array_eltype, jl_array_ndims,
        jl_array_nrows, jl_bool_type, jl_box_bool, jl_box_char, jl_box_float32, jl_box_float64,
        jl_box_int16, jl_box_int32, jl_box_int64, jl_box_int8, jl_box_uint16, jl_box_uint32,
        jl_box_uint64, jl_box_uint8, jl_char_type, jl_float32_type, jl_float64_type, jl_int16_type,
        jl_int32_type, jl_int64_type, jl_int8_type, jl_is_array, jl_is_string, jl_pchar_to_string,
        jl_string_data, jl_string_len, jl_typeis, jl_uint16_type, jl_uint32_type, jl_uint64_type,
        jl_uint8_type, jl_unbox_float32, jl_unbox_float64, jl_unbox_int16, jl_unbox_int32,
        jl_unbox_int64, jl_unbox_int8, jl_unbox_uint16, jl_unbox_uint32, jl_unbox_uint64,
        jl_unbox_uint8, jl_value_t,
    };
    use std::borrow::Cow;
    use std::mem::size_of;

    // If a trait A is used in a trait bound, the trait methods from traits that A extends become
    // available without explicitly using those base traits. By taking this struct, which can only
    // be created inside this crate, as an argument, these methods can only be called from this
    // crate.
    pub struct Internal;

    pub trait IntoJulia {
        // safety: Julia must have been initialized. The converted value must be protected from
        // garbage collection before calling into julia again.
        unsafe fn into_julia(&self, _: Internal) -> *mut jl_value_t;
    }

    pub trait JuliaType {
        // safety: Julia must have been initialized.
        unsafe fn julia_type(_: Internal) -> *mut jl_value_t;
    }

    pub trait ArrayData: JuliaType {}

    pub trait TryUnbox
    where
        Self: Sized,
    {
        // safety: you can't protect anything from garbage collection inside this function, so don't
        // call Julia functions and use the results. The value should be protected from garbage
        // collection when this function is called.
        unsafe fn try_unbox(value: *mut jl_value_t, _: Internal) -> JlrsResult<Self>;
    }

    pub trait Frame<'base: 'frame, 'frame> {
        // protect the value from being garbage collected while this frame is active.
        // safety: the value must be a valid Julia value
        unsafe fn protect(
            &mut self,
            value: *mut jl_value_t,
            _: Internal,
        ) -> JlrsResult<Value<'frame, 'static>>;

        // Create and protect multiple values from being garbage collected while this frame is active.
        fn create_many<P: IntoJulia>(
            &mut self,
            values: &[P],
            _: Internal,
        ) -> JlrsResult<Values<'frame>>;

        // Create and protect multiple values from being garbage collected while this frame is active.
        fn create_many_dyn(
            &mut self,
            values: &[&dyn super::IntoJulia],
            _: Internal,
        ) -> JlrsResult<Values<'frame>>;

        // Protect a value from being garbage collected while the output's frame is active.
        fn assign_output<'output>(
            &mut self,
            output: Output<'output>,
            value: *mut jl_value_t,
            _: Internal,
        ) -> Value<'output, 'static>;
    }

    macro_rules! impl_into_julia {
        ($type:ty, $boxer:ident) => {
            impl IntoJulia for $type {
                unsafe fn into_julia(&self, _: Internal) -> *mut jl_value_t {
                    $boxer(*self)
                }
            }
        };
        ($type:ty, $as:ty, $boxer:ident) => {
            impl IntoJulia for $type {
                unsafe fn into_julia(&self, _: Internal) -> *mut jl_value_t {
                    $boxer(*self as $as)
                }
            }
        };
    }

    impl_into_julia!(bool, i8, jl_box_bool);
    impl_into_julia!(char, u32, jl_box_char);
    impl_into_julia!(u8, jl_box_uint8);
    impl_into_julia!(u16, jl_box_uint16);
    impl_into_julia!(u32, jl_box_uint32);
    impl_into_julia!(u64, jl_box_uint64);
    impl_into_julia!(i8, jl_box_int8);
    impl_into_julia!(i16, jl_box_int16);
    impl_into_julia!(i32, jl_box_int32);
    impl_into_julia!(i64, jl_box_int64);
    impl_into_julia!(f32, jl_box_float32);
    impl_into_julia!(f64, jl_box_float64);

    impl IntoJulia for usize {
        unsafe fn into_julia(&self, _: Internal) -> *mut jl_value_t {
            if size_of::<usize>() == size_of::<u32>() {
                jl_box_uint32(*self as u32)
            } else {
                jl_box_uint64(*self as u64)
            }
        }
    }

    impl IntoJulia for isize {
        unsafe fn into_julia(&self, _: Internal) -> *mut jl_value_t {
            if size_of::<isize>() == size_of::<i32>() {
                jl_box_int32(*self as i32)
            } else {
                jl_box_int64(*self as i64)
            }
        }
    }

    impl<'a> IntoJulia for &'a str {
        unsafe fn into_julia(&self, _: Internal) -> *mut jl_value_t {
            let ptr = self.as_ptr() as _;
            let len = self.len();
            jl_pchar_to_string(ptr, len as _)
        }
    }

    impl<'a> IntoJulia for Cow<'a, str> {
        unsafe fn into_julia(&self, _: Internal) -> *mut jl_value_t {
            let ptr = self.as_ptr() as _;
            let len = self.len();
            jl_pchar_to_string(ptr, len as _)
        }
    }

    impl IntoJulia for String {
        unsafe fn into_julia(&self, _: Internal) -> *mut jl_value_t {
            let ptr = self.as_ptr() as _;
            let len = self.len();
            jl_pchar_to_string(ptr, len as _)
        }
    }

    impl IntoJulia for &dyn AsRef<str> {
        unsafe fn into_julia(&self, _: Internal) -> *mut jl_value_t {
            let ptr = self.as_ref().as_ptr() as _;
            let len = self.as_ref().len();
            jl_pchar_to_string(ptr, len as _)
        }
    }

    macro_rules! impl_julia_type {
        ($type:ty, $jl_type:expr) => {
            impl JuliaType for $type {
                unsafe fn julia_type(_: Internal) -> *mut jl_value_t {
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
        unsafe fn julia_type(_: Internal) -> *mut jl_value_t {
            if size_of::<usize>() == size_of::<u32>() {
                jl_uint32_type as _
            } else {
                jl_uint64_type as _
            }
        }
    }

    impl JuliaType for isize {
        unsafe fn julia_type(_: Internal) -> *mut jl_value_t {
            if size_of::<isize>() == size_of::<i32>() {
                jl_int32_type as _
            } else {
                jl_int64_type as _
            }
        }
    }

    macro_rules! impl_array_data {
        ($type:ty) => {
            impl ArrayData for $type {}
        };
    }

    impl_array_data!(u8);
    impl_array_data!(u16);
    impl_array_data!(u32);
    impl_array_data!(u64);
    impl_array_data!(i8);
    impl_array_data!(i16);
    impl_array_data!(i32);
    impl_array_data!(i64);
    impl_array_data!(f32);
    impl_array_data!(f64);
    impl_array_data!(usize);
    impl_array_data!(isize);

    macro_rules! impl_try_unbox {
        ($type:ty, $jl_type:expr, $unboxer:path) => {
            impl TryUnbox for $type {
                unsafe fn try_unbox(value: *mut jl_value_t, _: Internal) -> JlrsResult<Self> {
                    if jl_typeis(value, $jl_type) {
                        return Ok($unboxer(value));
                    }
                    Err(JlrsError::WrongType.into())
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
        unsafe fn try_unbox(value: *mut jl_value_t, _: Internal) -> JlrsResult<Self> {
            if jl_typeis(value, jl_bool_type) {
                return Ok(jl_unbox_int8(value) != 0);
            }
            Err(JlrsError::WrongType.into())
        }
    }

    impl TryUnbox for char {
        unsafe fn try_unbox(value: *mut jl_value_t, _: Internal) -> JlrsResult<Self> {
            if jl_typeis(value, jl_char_type) {
                return std::char::from_u32(jl_unbox_uint32(value))
                    .ok_or(JlrsError::InvalidCharacter.into());
            }

            Err(JlrsError::WrongType.into())
        }
    }

    impl TryUnbox for usize {
        unsafe fn try_unbox(value: *mut jl_value_t, _: Internal) -> JlrsResult<Self> {
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

    impl TryUnbox for isize {
        unsafe fn try_unbox(value: *mut jl_value_t, _: Internal) -> JlrsResult<Self> {
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

    impl TryUnbox for String {
        unsafe fn try_unbox(value: *mut jl_value_t, _: Internal) -> JlrsResult<String> {
            if !jl_is_string(value) {
                return Err(JlrsError::NotAString.into());
            }

            let len = jl_string_len(value);

            if len == 0 {
                return Ok(String::new());
            }

            // Is neither null nor dangling, we've just checked
            let raw = jl_string_data(value);
            let raw_slice = std::slice::from_raw_parts(raw, len);
            let owned_slice = Vec::from(raw_slice);
            Ok(String::from_utf8(owned_slice)?)
        }
    }

    impl<T: ArrayData> TryUnbox for Array<T> {
        unsafe fn try_unbox(value: *mut jl_value_t, _: Internal) -> JlrsResult<Self> {
            if !jl_is_array(value) {
                return Err(JlrsError::NotAnArray.into());
            }
            if jl_array_eltype(value) as *mut jl_value_t != T::julia_type(Internal) {
                return Err(JlrsError::WrongType.into());
            }
            let jl_data = jl_array_data(value) as *const T;
            let n_dims = jl_array_ndims(value as _);
            let dimensions: Dimensions = match n_dims {
                0 => return Err(JlrsError::ZeroDimension.into()),
                1 => Into::into(jl_array_nrows(value as _) as u64),
                2 => Into::into((
                    jl_array_dim(value as _, 0) as _,
                    jl_array_dim(value as _, 1) as _,
                )),
                3 => Into::into((
                    jl_array_dim(value as _, 0) as _,
                    jl_array_dim(value as _, 1) as _,
                    jl_array_dim(value as _, 2) as _,
                )),
                ndims => Into::into(jl_array_dims(value as _, ndims as _)),
            };
            let sz = dimensions.size();
            let mut data = Vec::with_capacity(sz as _);
            let ptr = data.as_mut_ptr();
            std::ptr::copy_nonoverlapping(jl_data, ptr, sz as _);
            data.set_len(sz as _);
            Ok(Array::new(data, dimensions))
        }
    }

    impl<'base: 'frame, 'frame> Frame<'base, 'frame> for StaticFrame<'base, 'frame> {
        unsafe fn protect(
            &mut self,
            value: *mut jl_value_t,
            _: Internal,
        ) -> JlrsResult<Value<'frame, 'static>> {
            if self.capacity == self.len {
                return Err(JlrsError::FrameSizeExceeded(self.len).into());
            }

            let out = {
                let out = self.memory.protect(self.idx, self.len, value as _);
                self.len += 1;
                out
            };

            Ok(out)
        }

        fn create_many<P: IntoJulia>(
            &mut self,
            values: &[P],
            _: Internal,
        ) -> JlrsResult<Values<'frame>> {
            unsafe {
                if self.capacity < self.len + values.len() {
                    return Err(JlrsError::FrameSizeExceeded(self.capacity).into());
                }

                let offset = self.len;
                for value in values {
                    self.memory
                        .protect(self.idx, self.len, value.into_julia(Internal) as _);
                    self.len += 1;
                }

                Ok(self.memory.as_values(self.idx, offset, values.len()))
            }
        }

        fn create_many_dyn(
            &mut self,
            values: &[&dyn super::IntoJulia],
            _: Internal,
        ) -> JlrsResult<Values<'frame>> {
            unsafe {
                if self.capacity < self.len + values.len() {
                    return Err(JlrsError::FrameSizeExceeded(self.len).into());
                }

                let offset = self.len;
                for value in values {
                    self.memory
                        .protect(self.idx, self.len, value.into_julia(Internal) as _);
                    self.len += 1;
                }

                Ok(self.memory.as_values(self.idx, offset, values.len()))
            }
        }

        fn assign_output<'output>(
            &mut self,
            output: Output<'output>,
            value: *mut jl_value_t,
            _: Internal,
        ) -> Value<'output, 'static> {
            unsafe {
                self.memory
                    .protect(FrameIdx::default(), output.offset, value as _)
            }
        }
    }

    impl<'base: 'frame, 'frame> Frame<'base, 'frame> for DynamicFrame<'base, 'frame> {
        unsafe fn protect(
            &mut self,
            value: *mut jl_value_t,
            _: Internal,
        ) -> JlrsResult<Value<'frame, 'static>> {
            let out = self.memory.protect(self.idx, value as _)?;
            self.len += 1;
            Ok(out)
        }

        fn create_many<P: IntoJulia>(
            &mut self,
            values: &[P],
            _: Internal,
        ) -> JlrsResult<Values<'frame>> {
            unsafe {
                let offset = self.len;

                for value in values {
                    self.memory
                        .protect(self.idx, value.into_julia(Internal) as _)?;
                    self.len += 1;
                }

                Ok(self.memory.as_values(self.idx, offset, values.len()))
            }
        }

        fn create_many_dyn(
            &mut self,
            values: &[&dyn super::IntoJulia],
            _: Internal,
        ) -> JlrsResult<Values<'frame>> {
            unsafe {
                let offset = self.len;

                for value in values {
                    self.memory
                        .protect(self.idx, value.into_julia(Internal) as _)?;
                    self.len += 1;
                }

                Ok(self.memory.as_values(self.idx, offset, values.len()))
            }
        }

        fn assign_output<'output>(
            &mut self,
            output: Output<'output>,
            value: *mut jl_value_t,
            _: Internal,
        ) -> Value<'output, 'static> {
            unsafe { self.memory.protect_output(output, value as _) }
        }
    }
}
