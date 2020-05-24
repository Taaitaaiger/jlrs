//! All traits used by the public API of this crate.
//!
//! A quick note: all of these traits are sealed, you can't implement them on your own types. Most
//! serve as marker traits that are used as trait bounds on some methods to enforce you can only
//! do things that make sense. The one trait that you will use directly is the [`Frame`] trait
//! which exposes methods that let you create [`Output`]s in the current frame and nest frames.
//! This trait is automatically brought into scope by using the [`prelude`] module.
//!
//! [`Frame`]: trait.Frame.html
//! [`Output`]: ../frame/struct.Output.html
//! [`prelude`]: ../prelude/index.html

use crate::error::{AllocError, JlrsError, JlrsResult};
use crate::frame::{DynamicFrame, Output, StaticFrame};
use crate::value::array::Array;
use crate::value::datatype::DataType;
use crate::value::module::Module;
use crate::value::symbol::Symbol;
use crate::value::Value;
use jl_sys::{
    jl_bool_type, jl_box_bool, jl_box_char, jl_box_float32, jl_box_float64, jl_box_int16,
    jl_box_int32, jl_box_int64, jl_box_int8, jl_box_uint16, jl_box_uint32, jl_box_uint64,
    jl_box_uint8, jl_char_type, jl_datatype_t, jl_float32_type, jl_float64_type, jl_int16_type,
    jl_int32_type, jl_int64_type, jl_int8_type, jl_pchar_to_string, jl_string_data,
    jl_string_len, jl_uint16_type, jl_uint32_type, jl_uint64_type, jl_uint8_type, jl_unbox_float32,
    jl_unbox_float64, jl_unbox_int16, jl_unbox_int32, jl_unbox_int64, jl_unbox_int8,
    jl_unbox_uint16, jl_unbox_uint32, jl_unbox_uint64, jl_unbox_uint8, jl_value_t,
};
use std::borrow::Cow;

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

/// Trait implemented by types that can be converted to a temporary `Symbol`.
pub trait TemporarySymbol: private::TemporarySymbol {}

/// Trait implemented by types that can be converted to a Julia value. Do not implement this
/// yourself. This trait can be derived for structs that are marked as `[repr(C)]` and only
/// contain fields that implement this trait by deriving `JuliaTuple`.
pub unsafe trait IntoJulia {
    /// Convert `self` into a raw Julia value. This value is not protected from garbage
    /// collection after creation and must be assigned to a slot on the GC stack before calling
    /// another allocating method from the Julia C API.
    unsafe fn into_julia(&self) -> *mut jl_value_t;
}

/// Trait implemented by types that have an associated type in Julia. Do not implement this
/// yourself. This trait can be derived for structs that are marked as `[repr(C)]` and only
/// contain fields that implement this trait by deriving `JuliaTuple`.
pub unsafe trait JuliaType {
    unsafe fn julia_type() -> *mut jl_datatype_t;
}

/// Implemented when using `#[derive(JuliaTuple)]`. Do not implement this yourself.
pub unsafe trait JuliaTuple: JuliaType + IntoJulia + Copy + Clone {}

/// Implemented when using `#[derive(JuliaTuple)]`. Do not implement this yourself.
pub unsafe trait JuliaStruct: JuliaType + IntoJulia + Copy + Clone {}

/// Trait implemented by types that have the same representation in Julia and Rust when they are
/// used as array data. Arrays whose elements are of a type that implements this trait can share
/// their contents between Julia and Rust. This includes all types that implement `JuliaType`
/// except `bool` and `char`.
pub trait ArrayDatatype: private::ArrayDatatype + JuliaType {}

/// This trait is used in combination with [`DataType::is`] and can be used to check many
/// properties of a Julia `DataType`. You should not implement this trait for your own types.
///
/// This trait is implemented for a few types from the standard library, eg `String` and `u8`. In
/// these cases, [`DataType::is`] returns true if [`Value::is`] would return `true` for that type.
///
/// [`DataType::is`]: ../value/datatype/struct.DataType.html#method.is
/// [`Value::is`]: ../value/struct.Value.html#method.is
pub unsafe trait JuliaTypecheck {
    #[doc(hidden)]
    unsafe fn julia_typecheck(t: DataType) -> bool;
}

/// This trait is implemented by types that represent special cases of [`Value`] that have a more
/// complex layout than several accessible fields. This currently includes [`Array`],
/// [`DataType`], [`Module`] and [`Symbol`].
///
/// [`Value`]: ../value/struct.Value.html
/// [`Array`]: ../value/array/struct.Array.html
/// [`DataType`]: ../value/datatype/struct.DataType.html
/// [`Module`]: ../value/module/struct.Module.html
/// [`Symbol`]: ../value/symbol/struct.Symbol.html
pub unsafe trait Cast<'frame, 'data> {
    type Output;
    #[doc(hidden)]
    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output>;

    #[doc(hidden)]
    unsafe fn cast_unchecked(value: Value<'frame, 'data>) -> Self::Output;
}

/// Functionality shared by [`StaticFrame`] and [`DynamicFrame`]. These structs let you protect
/// data from garbage collection. The lifetime of a frame is assigned to the values and outputs
/// that are created using that frame. After a frame is dropped, these items are no longer
/// protected and cannot be used.
///
/// If you need the result of a function call to be valid outside the frame where it is called,
/// you can call `Frame::output` to create an [`Output`] and call the function through
/// [`Value::call_output`] or one of the other `call*_output` methods. The result will share the
/// output's lifetime so it can be used until the output's frame goes out of scope.
///
/// [`StaticFrame`]: ../frame/struct.StaticFrame.html
/// [`DynamicFrame`]: ../frame/struct.DynamicFrame.html
/// [`Module`]: ../module/struct.Module.html
/// [`Julia::frame`]: ../struct.Julia.html#method.frame
/// [`Julia::dynamic_frame`]: ../struct.Julia.html#method.dynamic_frame
/// [`Output`]: ../frame/struct.Output.html
/// [`Value::call_output`]: ../value/struct.Value.html#method.call_output
pub trait Frame<'frame>: private::Frame<'frame> {
    /// Create a `StaticFrame` that can hold `capacity` values, and call the given closure.
    /// Returns the result of this closure, or an error if the new frame can't be created
    /// because there's not enough space on the GC stack. The number of required slots on the
    /// stack is `capacity + 2`.
    ///
    /// Returns an error if there is not enough space on the stack.
    fn frame<'nested, T, F: FnOnce(&mut StaticFrame<'nested>) -> JlrsResult<T>>(
        &'nested mut self,
        capacity: usize,
        func: F,
    ) -> JlrsResult<T>;

    /// Create a `DynamicFrame` and call the given closure.  Returns the result of this closure,
    /// or an error if the new frame can't be created because the stack is too small. The number
    /// of required slots on the stack is `2`.
    ///
    /// Returns an error if there is not enough space on the stack.
    fn dynamic_frame<'nested, T, F: FnOnce(&mut DynamicFrame<'nested>) -> JlrsResult<T>>(
        &'nested mut self,
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

p!(TemporarySymbol, String);
p!(TemporarySymbol, &dyn AsRef<str>);
p!(TemporarySymbol, &'a str, 'a);
p!(TemporarySymbol, Cow<'a, str>, 'a);
p!(TemporarySymbol, Symbol<'s>, 's);

macro_rules! impl_into_julia {
    ($type:ty, $boxer:ident) => {
        unsafe impl IntoJulia for $type {
            unsafe fn into_julia(&self) -> *mut jl_value_t {
                $boxer(*self)
            }
        }
    };
    ($type:ty, $as:ty, $boxer:ident) => {
        unsafe impl IntoJulia for $type {
            unsafe fn into_julia(&self) -> *mut jl_value_t {
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

#[cfg(not(target_pointer_width = "64"))]
unsafe impl IntoJulia for usize {
    unsafe fn into_julia(&self) -> *mut jl_value_t {
        jl_box_uint32(*self as u32)
    }
}

#[cfg(target_pointer_width = "64")]
unsafe impl IntoJulia for usize {
    unsafe fn into_julia(&self) -> *mut jl_value_t {
        jl_box_uint64(*self as u64)
    }
}

#[cfg(not(target_pointer_width = "64"))]
unsafe impl IntoJulia for isize {
    unsafe fn into_julia(&self) -> *mut jl_value_t {
        jl_box_int32(*self as i32)
    }
}

#[cfg(target_pointer_width = "64")]
unsafe impl IntoJulia for isize {
    unsafe fn into_julia(&self) -> *mut jl_value_t {
        jl_box_int64(*self as i64)
    }
}

unsafe impl<'a> IntoJulia for &'a str {
    unsafe fn into_julia(&self) -> *mut jl_value_t {
        let ptr = self.as_ptr().cast();
        let len = self.len();
        jl_pchar_to_string(ptr, len)
    }
}

unsafe impl<'a> IntoJulia for Cow<'a, str> {
    unsafe fn into_julia(&self) -> *mut jl_value_t {
        let ptr = self.as_ptr().cast();
        let len = self.len();
        jl_pchar_to_string(ptr, len)
    }
}

unsafe impl IntoJulia for String {
    unsafe fn into_julia(&self) -> *mut jl_value_t {
        let ptr = self.as_ptr().cast();
        let len = self.len();
        jl_pchar_to_string(ptr, len)
    }
}

unsafe impl IntoJulia for &dyn AsRef<str> {
    unsafe fn into_julia(&self) -> *mut jl_value_t {
        let ptr = self.as_ref().as_ptr().cast();
        let len = self.as_ref().len();
        jl_pchar_to_string(ptr, len)
    }
}

macro_rules! impl_julia_type {
    ($type:ty, $jl_type:expr) => {
        unsafe impl JuliaType for $type {
            unsafe fn julia_type() -> *mut jl_datatype_t {
                $jl_type
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

#[cfg(not(target_pointer_width = "64"))]
unsafe impl JuliaType for usize {
    unsafe fn julia_type() -> *mut jl_datatype_t {
        jl_uint32_type
    }
}

#[cfg(target_pointer_width = "64")]
unsafe impl JuliaType for usize {
    unsafe fn julia_type() -> *mut jl_datatype_t {
        jl_uint64_type
    }
}

#[cfg(not(target_pointer_width = "64"))]
unsafe impl JuliaType for isize {
    unsafe fn julia_type() -> *mut jl_datatype_t {
        jl_int32_type
    }
}

#[cfg(target_pointer_width = "64")]
unsafe impl JuliaType for isize {
    unsafe fn julia_type() -> *mut jl_datatype_t {
        jl_int64_type
    }
}

p!(ArrayDatatype, u8);
p!(ArrayDatatype, u16);
p!(ArrayDatatype, u32);
p!(ArrayDatatype, u64);
p!(ArrayDatatype, i8);
p!(ArrayDatatype, i16);
p!(ArrayDatatype, i32);
p!(ArrayDatatype, i64);
p!(ArrayDatatype, f32);
p!(ArrayDatatype, f64);
p!(ArrayDatatype, usize);
p!(ArrayDatatype, isize);

unsafe impl<'frame, 'data> Cast<'frame, 'data> for Array<'frame, 'data> {
    type Output = Self;
    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
        if value.is::<Array>() {
            return unsafe { Ok(Array::wrap(value.ptr().cast())) };
        }

        Err(JlrsError::NotAnArray)?
    }

    unsafe fn cast_unchecked<'fr, 'da>(value: Value<'frame, 'data>) -> Self::Output {
        Array::wrap(value.ptr().cast())
    }
}

unsafe impl<'frame, 'data> Cast<'frame, 'data> for DataType<'frame> {
    type Output = Self;
    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
        if value.is::<DataType>() {
            return unsafe { Ok(DataType::wrap(value.ptr().cast())) };
        }

        Err(JlrsError::NotADataType)?
    }

    unsafe fn cast_unchecked<'fr, 'da>(value: Value<'frame, 'data>) -> Self::Output {
        DataType::wrap(value.ptr().cast())
    }
}

unsafe impl<'frame, 'data> Cast<'frame, 'data> for Symbol<'frame> {
    type Output = Self;
    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
        if value.is::<Symbol>() {
            return unsafe { Ok(Symbol::wrap(value.ptr().cast())) };
        }

        Err(JlrsError::NotASymbol)?
    }

    unsafe fn cast_unchecked<'fr, 'da>(value: Value<'frame, 'data>) -> Self::Output {
        Symbol::wrap(value.ptr().cast())
    }
}

unsafe impl<'frame, 'data> Cast<'frame, 'data> for Module<'frame> {
    type Output = Self;
    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
        if value.is::<Module>() {
            return unsafe { Ok(Module::wrap(value.ptr().cast())) };
        }

        Err(JlrsError::NotAModule("This".to_string()))?
    }

    unsafe fn cast_unchecked<'fr, 'da>(value: Value<'frame, 'data>) -> Self::Output {
        Module::wrap(value.ptr().cast())
    }
}

macro_rules! impl_primitive_cast {
    ($type:ty, $unboxer:ident) => {
        unsafe impl<'frame, 'data> Cast<'frame, 'data> for $type {
            type Output = Self;

            fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
                if value.is::<$type>() {
                    return unsafe { Ok($unboxer(value.ptr().cast()) as _) };
                }

                Err(JlrsError::WrongType)?
            }

            unsafe fn cast_unchecked<'fr, 'da>(value: Value<'frame, 'data>) -> Self::Output {
                $unboxer(value.ptr().cast()) as _
            }
        }
    };
}

impl_primitive_cast!(u8, jl_unbox_uint8);
impl_primitive_cast!(u16, jl_unbox_uint16);
impl_primitive_cast!(u32, jl_unbox_uint32);
impl_primitive_cast!(u64, jl_unbox_uint64);
impl_primitive_cast!(i8, jl_unbox_int8);
impl_primitive_cast!(i16, jl_unbox_int16);
impl_primitive_cast!(i32, jl_unbox_int32);
impl_primitive_cast!(i64, jl_unbox_int64);
impl_primitive_cast!(f32, jl_unbox_float32);
impl_primitive_cast!(f64, jl_unbox_float64);

#[cfg(not(target_pointer_width = "64"))]
impl_primitive_cast!(usize, jl_unbox_uint32);

#[cfg(not(target_pointer_width = "64"))]
impl_primitive_cast!(isize, jl_unbox_int32);

#[cfg(target_pointer_width = "64")]
impl_primitive_cast!(usize, jl_unbox_uint64);

#[cfg(target_pointer_width = "64")]
impl_primitive_cast!(isize, jl_unbox_int64);

unsafe impl<'frame, 'data> Cast<'frame, 'data> for bool {
    type Output = Self;

    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
        if value.is::<bool>() {
            unsafe { return Ok(jl_unbox_int8(value.ptr()) != 0) }
        }

        Err(JlrsError::WrongType)?
    }

    unsafe fn cast_unchecked<'fr, 'da>(value: Value<'frame, 'data>) -> Self::Output {
        jl_unbox_int8(value.ptr()) != 0
    }
}

unsafe impl<'frame, 'data> Cast<'frame, 'data> for char {
    type Output = Self;

    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
        if value.is::<char>() {
            unsafe {
                return std::char::from_u32(jl_unbox_uint32(value.ptr()))
                    .ok_or(JlrsError::InvalidCharacter.into());
            }
        }

        Err(JlrsError::WrongType)?
    }

    unsafe fn cast_unchecked<'fr, 'da>(value: Value<'frame, 'data>) -> Self::Output {
        std::char::from_u32_unchecked(jl_unbox_uint32(value.ptr()))
    }
}

unsafe impl<'frame, 'data> Cast<'frame, 'data> for String {
    type Output = Self;

    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
        if value.is::<String>() {
            unsafe {
                let len = jl_string_len(value.ptr());

                if len == 0 {
                    return Ok(String::new());
                }

                // Is neither null nor dangling, we've just checked
                let raw = jl_string_data(value.ptr());
                let raw_slice = std::slice::from_raw_parts(raw, len);
                let owned_slice = Vec::from(raw_slice);
                return Ok(
                    String::from_utf8(owned_slice).map_err(|e| -> Box<JlrsError> {
                        let b: Box<dyn std::error::Error + Send + Sync> = Box::new(e);
                        b.into()
                    })?,
                );
            }
        }

        Err(JlrsError::WrongType)?
    }

    unsafe fn cast_unchecked<'fr, 'da>(value: Value<'frame, 'data>) -> Self::Output {
        let len = jl_string_len(value.ptr());

        if len == 0 {
            return String::new();
        }

        // Is neither null nor dangling, we've just checked
        let raw = jl_string_data(value.ptr());
        let raw_slice = std::slice::from_raw_parts(raw, len);
        let owned_slice = Vec::from(raw_slice);
        String::from_utf8_unchecked(owned_slice)
    }
}

impl<'frame> Frame<'frame> for StaticFrame<'frame> {
    fn frame<'nested, T, F: FnOnce(&mut StaticFrame<'nested>) -> JlrsResult<T>>(
        &'nested mut self,
        capacity: usize,
        func: F,
    ) -> JlrsResult<T> {
        let mut frame = unsafe { self.nested_frame(capacity).unwrap() };
        func(&mut frame)
    }

    fn dynamic_frame<'nested, T, F: FnOnce(&mut DynamicFrame<'nested>) -> JlrsResult<T>>(
        &'nested mut self,
        func: F,
    ) -> JlrsResult<T> {
        unsafe {
            let mut view = self.memory.nest_dynamic();
            let idx = view.new_frame()?;
            let mut frame = DynamicFrame {
                idx,
                len: 0,
                memory: view,
            };

            func(&mut frame)
        }
    }

    fn output(&mut self) -> JlrsResult<Output<'frame>> {
        if self.capacity == self.len {
            return Err(AllocError::FrameOverflow(1, self.len).into());
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

impl<'frame> Frame<'frame> for DynamicFrame<'frame> {
    fn dynamic_frame<'nested, T, F: FnOnce(&mut DynamicFrame<'nested>) -> JlrsResult<T>>(
        &'nested mut self,
        func: F,
    ) -> JlrsResult<T> {
        let mut frame = unsafe { self.nested_frame().unwrap() };
        func(&mut frame)
    }

    fn frame<'nested, T, F: FnOnce(&mut StaticFrame<'nested>) -> JlrsResult<T>>(
        &'nested mut self,
        capacity: usize,
        func: F,
    ) -> JlrsResult<T> {
        unsafe {
            let mut view = self.memory.nest_static();
            let idx = view.new_frame(capacity)?;
            let mut frame = StaticFrame {
                idx,
                capacity,
                len: 0,
                memory: view,
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
    use super::JuliaType;
    use crate::error::AllocError;
    use crate::frame::{DynamicFrame, Output, StaticFrame};
    use crate::stack::FrameIdx;
    use crate::value::symbol::Symbol;
    use crate::value::{Value, Values};
    use jl_sys::jl_symbol_n;
    use jl_sys::jl_value_t;
    use std::borrow::Cow;

    // If a trait A is used in a trait bound, the trait methods from traits that A extends become
    // available without explicitly using those base traits. By taking this struct, which can only
    // be created inside this crate, as an argument, these methods can only be called from this
    // crate.
    pub struct Internal;

    // safety: never return the symbol to the user without assigning the 'base lifetime.
    pub trait TemporarySymbol {
        unsafe fn temporary_symbol<'symbol>(&self, _: Internal) -> Symbol<'symbol>;
    }

    pub trait ArrayDatatype: JuliaType {}

    pub trait Frame<'frame> {
        // protect the value from being garbage collected while this frame is active.
        // safety: the value must be a valid Julia value
        unsafe fn protect(
            &mut self,
            value: *mut jl_value_t,
            _: Internal,
        ) -> Result<Value<'frame, 'static>, AllocError>;

        // Create and protect multiple values from being garbage collected while this frame is active.
        fn create_many<P: super::IntoJulia>(
            &mut self,
            values: &[P],
            _: Internal,
        ) -> Result<Values<'frame>, AllocError>;

        // Create and protect multiple values from being garbage collected while this frame is active.
        fn create_many_dyn(
            &mut self,
            values: &[&dyn super::IntoJulia],
            _: Internal,
        ) -> Result<Values<'frame>, AllocError>;

        // Protect a value from being garbage collected while the output's frame is active.
        fn assign_output<'output>(
            &mut self,
            output: Output<'output>,
            value: *mut jl_value_t,
            _: Internal,
        ) -> Value<'output, 'static>;
    }

    impl<'a> TemporarySymbol for &'a str {
        unsafe fn temporary_symbol<'symbol>(&self, _: Internal) -> Symbol<'symbol> {
            let symbol_ptr = self.as_ptr();
            let symbol = jl_symbol_n(symbol_ptr.cast(), self.len());
            Symbol::wrap(symbol)
        }
    }

    impl<'a> TemporarySymbol for Cow<'a, str> {
        unsafe fn temporary_symbol<'symbol>(&self, _: Internal) -> Symbol<'symbol> {
            let symbol_ptr = self.as_ptr().cast();
            let symbol = jl_symbol_n(symbol_ptr, self.len());
            Symbol::wrap(symbol)
        }
    }

    impl TemporarySymbol for String {
        unsafe fn temporary_symbol<'symbol>(&self, _: Internal) -> Symbol<'symbol> {
            let symbol_ptr = self.as_ptr().cast();
            let symbol = jl_symbol_n(symbol_ptr, self.len());
            Symbol::wrap(symbol)
        }
    }

    impl TemporarySymbol for &dyn AsRef<str> {
        unsafe fn temporary_symbol<'symbol>(&self, _: Internal) -> Symbol<'symbol> {
            let symbol_ptr = self.as_ref().as_ptr().cast();
            let symbol = jl_symbol_n(symbol_ptr, self.as_ref().len());
            Symbol::wrap(symbol)
        }
    }

    impl<'s> TemporarySymbol for Symbol<'s> {
        unsafe fn temporary_symbol<'symbol>(&self, _: Internal) -> Symbol<'symbol> {
            Symbol::wrap(self.ptr())
        }
    }

    macro_rules! impl_array_datatype {
        ($type:ty) => {
            impl ArrayDatatype for $type {}
        };
    }

    impl_array_datatype!(u8);
    impl_array_datatype!(u16);
    impl_array_datatype!(u32);
    impl_array_datatype!(u64);
    impl_array_datatype!(i8);
    impl_array_datatype!(i16);
    impl_array_datatype!(i32);
    impl_array_datatype!(i64);
    impl_array_datatype!(f32);
    impl_array_datatype!(f64);
    impl_array_datatype!(usize);
    impl_array_datatype!(isize);

    impl<'frame> Frame<'frame> for StaticFrame<'frame> {
        unsafe fn protect(
            &mut self,
            value: *mut jl_value_t,
            _: Internal,
        ) -> Result<Value<'frame, 'static>, AllocError> {
            if self.capacity == self.len {
                return Err(AllocError::FrameOverflow(1, self.len));
            }

            let out = {
                let out = self.memory.protect(self.idx, self.len, value.cast());
                self.len += 1;
                out
            };

            Ok(out)
        }

        fn create_many<P: super::IntoJulia>(
            &mut self,
            values: &[P],
            _: Internal,
        ) -> Result<Values<'frame>, AllocError> {
            unsafe {
                if self.capacity < self.len + values.len() {
                    return Err(AllocError::FrameOverflow(values.len(), self.capacity()));
                }

                let offset = self.len;
                for value in values {
                    self.memory
                        .protect(self.idx, self.len, value.into_julia().cast());
                    self.len += 1;
                }

                Ok(self.memory.as_values(self.idx, offset, values.len()))
            }
        }

        fn create_many_dyn(
            &mut self,
            values: &[&dyn super::IntoJulia],
            _: Internal,
        ) -> Result<Values<'frame>, AllocError> {
            unsafe {
                if self.capacity < self.len + values.len() {
                    return Err(AllocError::FrameOverflow(values.len(), self.capacity()));
                }

                let offset = self.len;
                for value in values {
                    self.memory
                        .protect(self.idx, self.len, value.into_julia().cast());
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
                    .protect(FrameIdx::default(), output.offset, value.cast())
            }
        }
    }

    impl<'frame> Frame<'frame> for DynamicFrame<'frame> {
        unsafe fn protect(
            &mut self,
            value: *mut jl_value_t,
            _: Internal,
        ) -> Result<Value<'frame, 'static>, AllocError> {
            let out = self.memory.protect(self.idx, value.cast())?;
            self.len += 1;
            Ok(out)
        }

        fn create_many<P: super::IntoJulia>(
            &mut self,
            values: &[P],
            _: Internal,
        ) -> Result<Values<'frame>, AllocError> {
            unsafe {
                let offset = self.len;
                // TODO: check capacity

                for value in values {
                    match self.memory.protect(self.idx, value.into_julia().cast()) {
                        Ok(_) => (),
                        Err(AllocError::StackOverflow(_, n)) => {
                            return Err(AllocError::StackOverflow(values.len(), n))
                        }
                        _ => unreachable!(),
                    }
                    self.len += 1;
                }

                Ok(self.memory.as_values(self.idx, offset, values.len()))
            }
        }

        fn create_many_dyn(
            &mut self,
            values: &[&dyn super::IntoJulia],
            _: Internal,
        ) -> Result<Values<'frame>, AllocError> {
            unsafe {
                let offset = self.len;
                // TODO: check capacity in advance

                for value in values {
                    self.memory.protect(self.idx, value.into_julia().cast())?;
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
            unsafe { self.memory.protect_output(output, value.cast()) }
        }
    }
}
