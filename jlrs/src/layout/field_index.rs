//! Field index trait.

/// Trait implemented by types that can be used in combination with a
/// [`FieldAccessor`].
///
/// [`FieldAccessor`]: crate::wrappers::ptr::value::FieldAccessor
pub trait FieldIndex: private::FieldIndexPriv {}
impl<I: private::FieldIndexPriv> FieldIndex for I {}

mod private {
    use crate::{
        convert::to_symbol::private::ToSymbolPriv,
        error::{AccessError, JlrsResult, CANNOT_DISPLAY_TYPE},
        private::Private,
        wrappers::ptr::{
            array::{dimensions::Dims, Array},
            datatype::DataType,
            string::JuliaString,
            symbol::Symbol,
            Wrapper,
        },
    };

    pub trait FieldIndexPriv {
        fn field_index(&self, ty: DataType, _: Private) -> JlrsResult<usize>;

        #[inline]
        fn array_index(&self, _data: Array, _: Private) -> JlrsResult<usize> {
            Err(AccessError::ArrayNeedsNumericalIndex)?
        }
    }

    impl FieldIndexPriv for &str {
        #[inline]
        fn field_index(&self, ty: DataType, _: Private) -> JlrsResult<usize> {
            // Safety: This method can only be called from a thread known to Julia
            ty.field_index(unsafe { self.to_symbol_priv(Private) })
        }
    }

    impl FieldIndexPriv for Symbol<'_> {
        #[inline]
        fn field_index(&self, ty: DataType, _: Private) -> JlrsResult<usize> {
            ty.field_index(*self)
        }
    }

    impl FieldIndexPriv for JuliaString<'_> {
        #[inline]
        fn field_index(&self, ty: DataType, _: Private) -> JlrsResult<usize> {
            // Safety: This method can only be called from a thread known to Julia
            ty.field_index(unsafe { self.to_symbol_priv(Private) })
        }
    }

    impl<D: Dims> FieldIndexPriv for D {
        fn field_index(&self, ty: DataType, _: Private) -> JlrsResult<usize> {
            debug_assert!(!ty.is::<Array>());

            if self.n_dimensions() != 1 {
                Err(AccessError::FieldNeedsSimpleIndex)?
            }

            let n = self.size();
            if ty.n_fields() as usize <= n {
                Err(AccessError::OutOfBoundsField {
                    idx: n,
                    n_fields: ty.n_fields() as usize,
                    value_type: ty.display_string_or(CANNOT_DISPLAY_TYPE),
                })?;
            }

            Ok(n)
        }

        #[inline]
        fn array_index(&self, data: Array, _: Private) -> JlrsResult<usize> {
            unsafe { data.dimensions().index_of(self) }
        }
    }
}
