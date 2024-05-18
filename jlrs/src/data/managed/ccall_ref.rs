//! A Julia `Ref` used as an argument of a `ccall`ed function.
//!
//! To quote the Julia docs, a `Ref` is an object that safely references data of type `T`. This
//! type is guaranteed to point to valid, Julia-allocated memory of the correct type. The
//! underlying data is protected from freeing by the garbage collector as long as the `Ref`
//! itself is referenced.
//!
//! When a `Ref` is used as an argument of a `ccall`ed function the data that is received by that
//! function depends on several details of the referenced type `T`.
//!
//!  - `T` is allocated inline
//!    This is the case when `T` is an immutable, concrete type. When such a type is used, the
//!    `Ref` is converted to a pointer to the referenced data and can be safely dereferenced
//!    immutably. Note that this is not a `Value`, it's not possible to access the `DataType` of
//!    this data.
//!
//!  - `T` is `Any`
//!    When `T` is explicitly the `Any` type, the data is passed as a reference to the referenced
//!    `Value`.
//!
//!  - `T` is none of the above
//!    This is the case when `T` is a mutable, abstract, or not a concrete type. The referenced
//!    data is passed as a `Value`.

use std::ptr::NonNull;

use super::{union_all::UnionAll, value::typed::TypedValueRet, Managed};
use crate::{
    convert::ccall_types::{CCallArg, CCallReturn},
    data::{
        layout::valid_layout::ValidLayout,
        managed::{datatype::DataType, value::Value},
        types::{
            abstract_type::{AnyType, RefTypeConstructor},
            construct_type::ConstructType,
            typecheck::Typecheck,
        },
    },
    error::{JlrsError, JlrsResult, TypeError, CANNOT_DISPLAY_TYPE},
    memory::target::unrooted::Unrooted,
    prelude::Target,
};

#[repr(C)]
union CCallRefInner<'scope, T> {
    ptr_to_inline: NonNull<T>,
    managed_type: Value<'scope, 'static>,
    ptr_to_value: &'scope Value<'scope, 'static>,
}

/// A `Ref` used as an argument of a `ccall`ed function.
#[repr(transparent)]
pub struct CCallRef<'scope, T>(CCallRefInner<'scope, T>);

impl<'scope, T> CCallRef<'scope, T>
where
    T: ConstructType + ValidLayout,
{
    /// Access the referenced data directly.
    ///
    /// `T` must be an immutable, concrete type. Only the base type is used to check if the layout
    /// of `T` is correct.
    #[inline]
    pub fn as_ref(&self) -> JlrsResult<&'scope T> {
        unsafe {
            let unrooted = Unrooted::new();
            let Some(base_type) = T::base_type(&unrooted) else {
                Err(JlrsError::TypeError(TypeError::NoBaseType))?
            };

            if base_type.is::<DataType>() {
                let base_dt = base_type.cast_unchecked::<DataType>();
                if base_dt.is_inline_alloc() && T::valid_layout(base_type) {
                    return Ok(self.0.ptr_to_inline.as_ref());
                }
            } else if base_type.is::<UnionAll>() {
                let base_ua = base_type.cast_unchecked::<UnionAll>();
                let base_dt = base_ua.base_type();

                if base_dt.is_inline_alloc() && T::valid_layout(base_type) {
                    return Ok(self.0.ptr_to_inline.as_ref());
                }
            }

            Err(TypeError::IncompatibleBaseType {
                base_type: base_type.display_string_or(CANNOT_DISPLAY_TYPE),
            })?
        }
    }

    /// Access the referenced data directly.
    ///
    /// `T` must be an immutable, concrete type. Unlike [`CCallRef::as_ref`] this method
    /// constructs the type associated with `T` to check if the layout is correct.
    #[inline]
    pub fn as_ref_check_constructed<'target, Tgt>(&self, target: &Tgt) -> JlrsResult<&'scope T>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, _, 1>(|_, mut frame| unsafe {
            let ty = T::construct_type(&mut frame);

            if ty.is::<DataType>() {
                let base_dt = ty.cast_unchecked::<DataType>();
                if base_dt.is_inline_alloc() && T::valid_layout(ty) {
                    return Ok(self.0.ptr_to_inline.as_ref());
                }
            }

            Err(TypeError::IncompatibleBaseType {
                base_type: ty.display_string_or("<Cannot display type>"),
            })?
        })
    }
}

impl<'scope, T> CCallRef<'scope, T> {
    /// Access the referenced data directly without checking if this conversion is valid.
    ///
    /// Safety: `T` must be the layout of the referenced data.
    #[inline]
    pub unsafe fn as_ref_unchecked(&self) -> &'scope T {
        self.0.ptr_to_inline.as_ref()
    }

    /// Access the referenced data directly without checking if this conversion is valid.
    ///
    /// Safety: `U` must be the layout of the referenced data.
    #[inline]
    pub unsafe fn as_ref_to_unchecked<U>(&self) -> &'scope U {
        self.0.ptr_to_inline.cast().as_ref()
    }

    /// Access the referenced data as a `Value` without checking if this conversion is valid.
    ///
    /// Safety: `T` must not be an inline allocated type, or `Any`.
    #[inline]
    pub unsafe fn as_value_unchecked(&self) -> Value<'scope, 'static> {
        self.0.managed_type
    }
}

impl<'scope, T> CCallRef<'scope, T>
where
    T: ConstructType,
{
    /// Access the referenced data as a reference to `U`.
    ///
    /// `T` must be an immutable, concrete type. Only the base type is used to check if the layout
    /// of `U` is correct.
    #[inline]
    pub fn as_ref_to<U: ValidLayout>(&self) -> JlrsResult<&U> {
        unsafe {
            let unrooted = Unrooted::new();
            let Some(base_type) = T::base_type(&unrooted) else {
                Err(JlrsError::TypeError(TypeError::NoBaseType))?
            };

            if base_type.is::<DataType>() {
                let base_dt = base_type.cast_unchecked::<DataType>();
                if base_dt.is_inline_alloc() && U::valid_layout(base_type) {
                    return Ok(self.0.ptr_to_inline.cast().as_ref());
                }
            } else if base_type.is::<UnionAll>() {
                let base_ua = base_type.cast_unchecked::<UnionAll>();
                let base_dt = base_ua.base_type();

                if base_dt.is_inline_alloc() && U::valid_layout(base_type) {
                    return Ok(self.0.ptr_to_inline.cast().as_ref());
                }
            }

            Err(TypeError::IncompatibleBaseType {
                base_type: base_type.display_string_or("<Cannot display type>"),
            })?
        }
    }

    /// Access the referenced data as a reference to `U`.
    ///
    /// `T` must be an immutable, concrete type. Unlike [`CCallRef::as_ref_to`] this method
    /// constructs the type associated with `T` to check if the layout is correct.
    #[inline]
    pub fn as_ref_to_check_constructed<'target, U: ValidLayout, Tgt: Target<'target>>(
        &self,
        target: Tgt,
    ) -> JlrsResult<&U> {
        target.with_local_scope::<_, _, 1>(|_, mut frame| unsafe {
            let ty = T::construct_type(&mut frame);

            if ty.is::<DataType>() {
                let base_dt = ty.cast_unchecked::<DataType>();
                if base_dt.is_inline_alloc() && U::valid_layout(ty) {
                    return Ok(self.0.ptr_to_inline.cast().as_ref());
                }
            }

            Err(TypeError::IncompatibleBaseType {
                base_type: ty.display_string_or("<Cannot display type>"),
            })?
        })
    }
}

impl<'scope, T> CCallRef<'scope, T>
where
    T: ConstructType,
{
    /// Access the referenced data as a `Value`.
    ///
    /// Only the base type of `T` is used to check if the data is passed as a `Value`.
    pub fn as_value(&self) -> JlrsResult<Value<'scope, 'static>> {
        unsafe {
            let unrooted = Unrooted::new();
            let Some(base_type) = T::base_type(&unrooted) else {
                Err(JlrsError::TypeError(TypeError::NoBaseType))?
            };

            if base_type == AnyType::base_type(&unrooted).unwrap() {
                Err(TypeError::IncompatibleBaseType {
                    base_type: base_type.display_string_or("<Cannot display type>"),
                })?
            }

            if base_type.is::<DataType>() {
                let base_dt = base_type.cast_unchecked::<DataType>();
                if !base_dt.is_concrete_type() || base_dt.mutable() {
                    return Ok(self.0.managed_type);
                }
            } else if base_type.is::<UnionAll>() {
                let base_ua = base_type.cast_unchecked::<UnionAll>();
                let base_dt = base_ua.base_type();

                if !base_dt.is_concrete_type() || base_dt.mutable() {
                    return Ok(self.0.managed_type);
                }
            }

            Err(TypeError::IncompatibleBaseType {
                base_type: base_type.display_string_or("<Cannot display type>"),
            })?
        }
    }

    /// Access the referenced data as a reference to `Value`.
    ///
    /// Unlike [`CCallRef::as_value`] this method constructs the type associated with `T` to check
    /// if the layout is correct.
    pub fn as_value_check_constructed<'target, Tgt: Target<'target>>(
        &self,
        target: &Tgt,
    ) -> JlrsResult<Value<'scope, 'static>> {
        target.with_local_scope::<_, _, 1>(|_, mut frame| unsafe {
            let ty = T::construct_type(&mut frame);

            if ty.is::<DataType>() {
                let base_dt = ty.cast_unchecked::<DataType>();
                if !base_dt.is_concrete_type() || base_dt.mutable() {
                    return Ok(self.0.managed_type);
                }
            } else if ty.is::<UnionAll>() {
                let base_ua = ty.cast_unchecked::<UnionAll>();
                let base_dt = base_ua.base_type();

                if !base_dt.is_concrete_type() || base_dt.mutable() {
                    return Ok(self.0.managed_type);
                }
            }

            Err(TypeError::IncompatibleBaseType {
                base_type: ty.display_string_or("<Cannot display type>"),
            })?
        })
    }
}

impl<'scope> CCallRef<'scope, AnyType> {
    /// Access the referenced data as a reference to a `Value`.
    #[inline]
    pub fn as_value_ref(&self) -> &Value<'scope, 'static> {
        unsafe { self.0.ptr_to_value }
    }
}

impl<'scope, T> CCallRef<'scope, T>
where
    T: Managed<'scope, 'static> + Typecheck + ConstructType,
{
    /// Access the referenced data as `T`.
    #[inline]
    pub fn as_managed(&self) -> JlrsResult<T> {
        unsafe {
            let unrooted = Unrooted::new();
            let Some(base_type) = T::base_type(&unrooted) else {
                Err(JlrsError::TypeError(TypeError::NoBaseType))?
            };

            if base_type == AnyType::base_type(&unrooted).unwrap() {
                Err(TypeError::IncompatibleBaseType {
                    base_type: base_type.display_string_or("<Cannot display type>"),
                })?
            }

            if base_type.is::<DataType>() {
                let base_dt = base_type.cast_unchecked::<DataType>();
                if base_dt.is_concrete_type() && base_dt.mutable() {
                    return self.0.managed_type.cast::<T>();
                }
            } else if base_type.is::<UnionAll>() {
                let base_ua = base_type.cast_unchecked::<UnionAll>();
                let base_dt = base_ua.base_type();

                if base_dt.is_concrete_type() && base_dt.mutable() {
                    return self.0.managed_type.cast::<T>();
                }
            }

            Err(TypeError::IncompatibleBaseType {
                base_type: base_type.display_string_or("<Cannot display type>"),
            })?
        }
    }
}

impl<'scope, T> CCallRef<'scope, T>
where
    T: Managed<'scope, 'static>,
{
    /// Access the referenced data as `T` without checking if this conversion is valid.
    ///
    /// Safety: `T` must not be `Value`.
    #[inline]
    pub unsafe fn as_managed_unchecked(&self) -> T {
        self.0.managed_type.cast_unchecked::<T>()
    }
}

unsafe impl<'scope, T: ConstructType> CCallArg for CCallRef<'scope, T> {
    type CCallArgType = RefTypeConstructor<T>;
    type FunctionArgType = T;
}

/// A `Ref` used as the return type of a `ccall`ed function.
///
/// When this type is returned by a function exported with the `julia_module` macro, the
/// generated Julia function will contain a `ccall` invocation that returns `Ref{T}`, while
/// `TypedValueRet<T>` would lead to a `ccall` invocation that returns `Any`.
#[repr(transparent)]
pub struct CCallRefRet<T: ConstructType>(TypedValueRet<T>);

impl<T: ConstructType> CCallRefRet<T> {
    /// Convert a `TypedValueRet<T>` to a `CCallRefRet<T>`.
    #[inline]
    pub fn new(value: TypedValueRet<T>) -> Self {
        CCallRefRet(value)
    }

    #[inline]
    pub fn into_typed_value(self) -> TypedValueRet<T> {
        self.0
    }
}

unsafe impl<T: ConstructType> CCallReturn for CCallRefRet<T> {
    type FunctionReturnType = T;
    type CCallReturnType = RefTypeConstructor<T>;
    type ReturnAs = Self;

    #[inline]
    unsafe fn return_or_throw(self) -> Self::ReturnAs {
        self
    }
}
