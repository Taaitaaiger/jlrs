//! Wrapper for `Core.Function`, the super type of all Julia functions.
//!
//! All Julia functions are subtypes of `Core.Function`, a function can be called with the methods
//! of the [`Call`] trait. Note that you don't need to cast a [`Value`] to a [`Function`] in order
//! to call it because [`Value`] also implements [`Call`].
//!
//! [`Call`]: crate::call::Call

use jl_sys::jl_value_t;
use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
    ptr::NonNull,
};

use crate::{
    layout::{typecheck::Typecheck, valid_layout::ValidLayout},
    memory::global::Global,
    private::Private,
};

use super::{datatype::DataType, private::Wrapper as WrapperPriv, value::Value, Wrapper};

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Function<'scope, 'data> {
    inner: NonNull<jl_value_t>,
    _scope: PhantomData<&'scope ()>,
    _data: PhantomData<&'data ()>,
}

impl<'scope, 'data> Function<'scope, 'data> {
    /// Returns the `DataType` of this function. In Julia, every function has its own `DataType`.
    pub fn datatype(self) -> DataType<'scope> {
        self.as_value().datatype()
    }
}

impl<'scope, 'data> Debug for Function<'scope, 'data> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let name = self.datatype().name();
        f.write_fmt(format_args!("Function<{}>", name))
    }
}

unsafe impl ValidLayout for Function<'_, '_> {
    unsafe fn valid_layout(ty: Value) -> bool {
        let global = Global::new();
        let function_type = DataType::function_type(global);
        ty.subtype(function_type.as_value())
    }
}

unsafe impl Typecheck for Function<'_, '_> {
    unsafe fn typecheck(t: DataType) -> bool {
        <Self as ValidLayout>::valid_layout(t.as_value())
    }
}

impl<'scope, 'data> WrapperPriv<'scope, 'data> for Function<'scope, 'data> {
    type Internal = jl_value_t;

    unsafe fn wrap_non_null(inner: NonNull<Self::Internal>, _: Private) -> Self {
        Self {
            inner,
            _scope: PhantomData,
            _data: PhantomData,
        }
    }

    unsafe fn unwrap_non_null(self, _: Private) -> NonNull<Self::Internal> {
        self.inner
    }
}
