//! Support for values with the `Core.Method` type.

use super::Value;
use super::symbol::Symbol;
use super::module::Module;
use super::array::Array;
use super::method_instance::MethodInstance;
use super::simple_vector::SimpleVector;
use crate::error::{JlrsError, JlrsResult};
use crate::traits::Cast;
use crate::{impl_julia_type, impl_julia_typecheck, impl_valid_layout};
use jl_sys::{jl_method_t, jl_method_type};
use std::marker::PhantomData;

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct Method<'frame>(*mut jl_method_t, PhantomData<&'frame ()>);

impl<'frame> Method<'frame> {
    pub(crate) unsafe fn wrap(method: *mut jl_method_t) -> Self {
        Method(method, PhantomData)
    }

    #[doc(hidden)]
    pub unsafe fn ptr(self) -> *mut jl_method_t {
        self.0
    }

    pub fn name(self) -> Symbol<'frame> {
        unsafe {
            Symbol::wrap((&*self.ptr()).name)
        }
    }

    pub fn module(self) -> Module<'frame> {
        unsafe {
            Module::wrap((&*self.ptr()).module)
        }
    }

    pub fn file(self) -> Symbol<'frame> {
        unsafe {
            Symbol::wrap((&*self.ptr()).file)
        }
    }

    pub fn line(self) -> i32 {
        unsafe {
            (&*self.ptr()).line
        }
    }

    pub fn primary_world(self) -> usize {
        unsafe {
            (&*self.ptr()).primary_world
        }
    }

    pub fn deleted_world(self) -> usize {
        unsafe {
            (&*self.ptr()).deleted_world
        }
    }

    pub fn signature(self) -> Value<'frame, 'static> {
        unsafe {
            Value::wrap((&*self.ptr()).sig)
        }
    }

    pub fn ambiguous(self) -> Value<'frame, 'static> {
        unsafe {
            Value::wrap((&*self.ptr()).ambig)
        }
    }

    pub fn resorted(self) -> Value<'frame, 'static> {
        unsafe {
            Value::wrap((&*self.ptr()).resorted)
        }
    }

    pub fn specializations(self) -> SimpleVector<'frame> {
        unsafe {
            SimpleVector::wrap((&*self.ptr()).specializations)
        }
    }

    pub fn speckeyset(self) -> Array<'frame, 'static> {
        unsafe {
            Array::wrap((&*self.ptr()).speckeyset)
        }
    }

    pub fn slot_syms(self) -> Value<'frame, 'static> {
        unsafe {
            Value::wrap((&*self.ptr()).slot_syms)
        }
    }

    pub fn source(self) -> Option<Value<'frame, 'static>> {
        unsafe {
            let source = (&*self.ptr()).source;
            if source.is_null() {
                None
            } else {
                Some(Value::wrap(source))
            }
        }
    }

    pub fn unspecialized(self) -> Option<MethodInstance<'frame>> {
        unsafe {
            let unspecialized = (&*self.ptr()).unspecialized;
            if unspecialized.is_null() {
                None
            } else {
                Some(MethodInstance::wrap(unspecialized))
            }
        }
    }

    pub fn generator(self) -> Option<Value<'frame, 'static>> {
        unsafe {
            let generator = (&*self.ptr()).generator;
            if generator.is_null() {
                None
            } else {
                Some(Value::wrap(generator))
            }
        }
    }

    pub fn roots(self) -> Option<Array<'frame, 'static>> {
        unsafe {
            let roots = (&*self.ptr()).roots;
            if roots.is_null() {
                None
            } else {
                Some(Array::wrap(roots))
            }
        }
    }

    pub fn ccallable(self) -> Option<SimpleVector<'frame>> {
        unsafe {
            let ccallable = (&*self.ptr()).ccallable;
            if ccallable.is_null() {
                None
            } else {
                Some(SimpleVector::wrap(ccallable))
            }
        }
    }

    pub fn invokes(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap((&*self.ptr()).invokes) }
    }

    pub fn n_args(self) -> i32 {
        unsafe {
            (&*self.ptr()).nargs
        }
    }

    pub fn called(self) -> i32 {
        unsafe {
            (&*self.ptr()).called
        }
    }

    pub fn nospecialize(self) -> i32 {
        unsafe {
            (&*self.ptr()).nospecialize
        }
    }

    pub fn nkw(self) -> i32 {
        unsafe {
            (&*self.ptr()).nkw
        }
    }

    pub fn is_varargs(self) -> bool {
        unsafe {
            (&*self.ptr()).isva != 0
        }
    }

    pub fn pure(self) -> bool {
        unsafe {
            (&*self.ptr()).pure_ != 0
        }
    }
}

impl<'frame> Into<Value<'frame, 'static>> for Method<'frame> {
    fn into(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap(self.ptr().cast()) }
    }
}

unsafe impl<'frame, 'data> Cast<'frame, 'data> for Method<'frame> {
    type Output = Self;
    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
        if value.is::<Self::Output>() {
            return unsafe { Ok(Self::cast_unchecked(value)) };
        }

        Err(JlrsError::NotAMethod)?
    }

    unsafe fn cast_unchecked(value: Value<'frame, 'data>) -> Self::Output {
        Self::wrap(value.ptr().cast())
    }
}

impl_julia_typecheck!(Method<'frame>, jl_method_type, 'frame);
impl_julia_type!(Method<'frame>, jl_method_type, 'frame);
impl_valid_layout!(Method<'frame>, 'frame);
