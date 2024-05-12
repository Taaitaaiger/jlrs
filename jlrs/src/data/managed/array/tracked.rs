/// Tracked references to Julia arrays
///
/// It's very easy to create multiple aliasing references to Julia array data if you use the
/// access methods defined in the `array` module. To help avoid this problem, you can track
/// arrays by calling [`ArrayBase::track_shared`] or [`ArrayBase::track_exclusive`].
///
/// If you strictly use tracking before accessing an array, it's not possible to create multiple
/// aliasing references to this data in Rust. These last two words are important: access to this
/// array in Julia is in no way affected, and you are still responsible for ensuring you don't
/// access data that is in use in Julia.
///
/// [`ArrayBase::track_shared`]: crate::data::managed::array::ArrayBase::track_shared
/// [`ArrayBase::track_exclusive`]: crate::data::managed::array::ArrayBase::track_exclusive
use std::{ops::Deref, ptr::NonNull};

use jl_sys::jlrs_array_data_owner;

use super::{
    data::accessor::{
        BitsAccessor, BitsAccessorMut, BitsUnionAccessor, BitsUnionAccessorMut,
        IndeterminateAccessor, IndeterminateAccessorMut, InlineAccessor, InlineAccessorMut,
        ManagedAccessor, ManagedAccessorMut, ValueAccessor, ValueAccessorMut,
    },
    ArrayBase, Unknown,
};
use crate::{
    data::{
        layout::{is_bits::IsBits, typed_layout::HasLayout, valid_layout::ValidField},
        managed::{array::How, private::ManagedPriv},
        types::{
            construct_type::{BitsUnionCtor, ConstructType},
            typecheck::Typecheck,
        },
    },
    error::{ArrayLayoutError, TypeError, CANNOT_DISPLAY_TYPE},
    memory::context::ledger::Ledger,
    prelude::{JlrsResult, Managed, Value},
    private::Private,
};

/// A tracked array that provides immutable access
#[repr(transparent)]
pub struct TrackedArrayBase<'scope, 'data, T, const N: isize> {
    data: ArrayBase<'scope, 'data, T, N>,
}

// Accessors
impl<'scope, 'data, T, const N: isize> TrackedArrayBase<'scope, 'data, T, N> {
    /// Create an accessor for `isbits` data.
    ///
    /// Thanks to the restrictions on `T` the data is guaranteed to be stored inline as an array
    /// of `T`s.
    pub fn bits_data<'borrow>(&'borrow self) -> BitsAccessor<'borrow, 'scope, 'data, T, T, N>
    where
        T: ConstructType + ValidField + IsBits,
    {
        // No need for checks, guaranteed to have isbits layout
        unsafe { BitsAccessor::new(&self.data) }
    }

    /// Create an accessor for `isbits` data with layout `L`.
    ///
    /// Thanks to the restrictions on `T` and `L` the elements are guaranteed to be stored inline
    /// as an array of `L`s.
    pub fn bits_data_with_layout<'borrow, L>(
        &'borrow self,
    ) -> BitsAccessor<'borrow, 'scope, 'data, T, L, N>
    where
        T: ConstructType + HasLayout<'static, 'static, Layout = L>,
        L: IsBits + ValidField,
    {
        // No need for checks, guaranteed to have isbits layout and L is the layout of T
        unsafe { BitsAccessor::new(&self.data) }
    }

    /// Try to create an accessor for `isbits` data with layout `L`.
    ///
    /// If the array doesn't have an isbits layout `ArrayLayoutError::NotBits` is returned. If `L`
    /// is not a valid field layout for the element type `TypeError::InvalidLayout` is returned.
    pub fn try_bits_data<'borrow, L>(
        &'borrow self,
    ) -> JlrsResult<BitsAccessor<'borrow, 'scope, 'data, T, L, N>>
    where
        L: IsBits + ValidField,
    {
        if !self.has_bits_layout() {
            Err(ArrayLayoutError::NotBits {
                element_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        let ty = self.element_type();
        if !L::valid_field(ty) {
            Err(TypeError::InvalidLayout {
                value_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        unsafe { Ok(BitsAccessor::new(&self.data)) }
    }

    /// Create an accessor for `isbits` data with layout `L` without checking any invariants.
    ///
    /// Safety:
    ///
    /// The element type must be an isbits type, and `L` must be a valid field layout of the
    /// element type.
    pub unsafe fn bits_data_unchecked<'borrow, L>(
        &'borrow self,
    ) -> BitsAccessor<'borrow, 'scope, 'data, T, L, N>
    where
        L: IsBits + ValidField,
    {
        BitsAccessor::new(&self.data)
    }

    /// Create an accessor for inline data.
    ///
    /// Thanks to the restrictions on `T` the data is guaranteed to be stored inline as an array
    /// of `T`s.
    pub fn inline_data<'borrow>(&'borrow self) -> InlineAccessor<'borrow, 'scope, 'data, T, T, N>
    where
        T: ConstructType + ValidField,
    {
        // No need for checks, guaranteed to have inline layout
        unsafe { InlineAccessor::new(&self.data) }
    }

    /// Create an accessor for inline data with layout `L`.
    ///
    /// Thanks to the restrictions on `T` and `L` the elements are guaranteed to be stored inline
    /// as an array of `L`s.
    pub fn inline_data_with_layout<'borrow, L>(
        &'borrow self,
    ) -> InlineAccessor<'borrow, 'scope, 'data, T, L, N>
    where
        T: ConstructType + HasLayout<'scope, 'data, Layout = L>,
        L: ValidField,
    {
        // No need for checks, guaranteed to have inline layout and L is the layout of T
        unsafe { InlineAccessor::new(&self.data) }
    }

    /// Try to create an accessor for inline data with layout `L`.
    ///
    /// If the array doesn't have an inline layout `ArrayLayoutError::NotInline` is returned. If
    /// `L` is not a valid field layout for the element type `TypeError::InvalidLayout` is
    /// returned.
    pub fn try_inline_data<'borrow, L>(
        &'borrow self,
    ) -> JlrsResult<InlineAccessor<'borrow, 'scope, 'data, T, L, N>>
    where
        L: ValidField,
    {
        if !self.has_inline_layout() {
            Err(ArrayLayoutError::NotInline {
                element_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        let ty = self.element_type();
        if !L::valid_field(ty) {
            Err(TypeError::InvalidLayout {
                value_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        unsafe { Ok(InlineAccessor::new(&self.data)) }
    }

    /// Create an accessor for inline data with layout `L` without checking any invariants.
    ///
    /// Safety:
    ///
    /// The elements must be stored inline, and `L` must be a valid field layout of the element
    /// type.
    pub unsafe fn inline_data_unchecked<'borrow, L>(
        &'borrow self,
    ) -> InlineAccessor<'borrow, 'scope, 'data, T, L, N>
    where
        L: ValidField,
    {
        InlineAccessor::new(&self.data)
    }

    /// Create an accessor for unions of isbits types.
    ///
    /// This function panics if the array doesn't have a union layout.
    pub fn union_data<'borrow>(&'borrow self) -> BitsUnionAccessor<'borrow, 'scope, 'data, T, N>
    where
        T: BitsUnionCtor,
    {
        assert!(
            self.has_union_layout(),
            "Array does not have a union layout"
        );

        unsafe { BitsUnionAccessor::new(&self.data) }
    }

    /// Try to create an accessor for unions of isbits types.
    ///
    /// If the element type is not a union of isbits types `ArrayLayoutError::NotUnion` is
    /// returned.
    pub fn try_union_data<'borrow>(
        &'borrow self,
    ) -> JlrsResult<BitsUnionAccessor<'borrow, 'scope, 'data, T, N>> {
        if !self.has_union_layout() {
            Err(ArrayLayoutError::NotUnion {
                element_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        unsafe { Ok(BitsUnionAccessor::new(&self.data)) }
    }

    /// Create an accessor for unions of isbits types without checking any invariants.
    ///
    /// Safety:
    ///
    /// The element type must be a union of isbits types.
    pub unsafe fn union_data_unchecked<'borrow>(
        &'borrow self,
    ) -> BitsUnionAccessor<'borrow, 'scope, 'data, T, N> {
        BitsUnionAccessor::new(&self.data)
    }
    /// Create an accessor for managed data.
    ///
    /// Thanks to the restrictions on `T` the data is guaranteed to be as an array of
    /// `Option<Ref<T>>`s.
    pub fn managed_data<'borrow>(&'borrow self) -> ManagedAccessor<'borrow, 'scope, 'data, T, T, N>
    where
        T: Managed<'scope, 'data> + ConstructType,
    {
        // No need for checks, guaranteed to have correct layout
        unsafe { ManagedAccessor::new(&self.data) }
    }

    /// Try to create an accessor for managed data of type `L`.
    ///
    /// If the element type is incompatible with `L` `ArrayLayoutError::NotManaged` is returned.
    pub fn try_managed_data<'borrow, L>(
        &'borrow self,
    ) -> JlrsResult<ManagedAccessor<'borrow, 'scope, 'data, T, L, N>>
    where
        L: Managed<'scope, 'data> + Typecheck,
    {
        if !self.has_managed_layout::<L>() {
            Err(ArrayLayoutError::NotManaged {
                element_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
                name: L::NAME.into(),
            })?;
        }

        unsafe { Ok(ManagedAccessor::new(&self.data)) }
    }

    /// Create an accessor for managed data of type `L` without checking any invariants.
    ///
    /// Safety:
    ///
    /// The element type must be compatible with `L`.
    pub unsafe fn managed_data_unchecked<'borrow, L>(
        &'borrow self,
    ) -> ManagedAccessor<'borrow, 'scope, 'data, T, L, N>
    where
        L: Managed<'scope, 'data>,
    {
        ManagedAccessor::new(&self.data)
    }

    /// Create an accessor for value data.
    ///
    /// Thanks to the restrictions on `T` the data is guaranteed to be as an array of
    /// `Option<Ref<Value>>`s.
    pub fn value_data<'borrow>(&'borrow self) -> ValueAccessor<'borrow, 'scope, 'data, T, N>
    where
        T: Managed<'scope, 'data> + ConstructType,
    {
        // No need for checks, guaranteed to have inline layout
        unsafe { ValueAccessor::new(&self.data) }
    }

    /// Try to create an accessor for value data.
    ///
    /// If the elements are stored inline `ArrayLayoutError::NotPointer` is returned.
    pub fn try_value_data<'borrow>(
        &'borrow self,
    ) -> JlrsResult<ValueAccessor<'borrow, 'scope, 'data, T, N>> {
        if !self.has_value_layout() {
            Err(ArrayLayoutError::NotPointer {
                element_type: self.element_type().error_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        unsafe { Ok(ValueAccessor::new(&self.data)) }
    }

    /// Create an accessor for managed data of type `L` without checking any invariants.
    ///
    /// Safety:
    ///
    /// The elements must not be stored inline.
    pub unsafe fn value_data_unchecked<'borrow>(
        &'borrow self,
    ) -> ValueAccessor<'borrow, 'scope, 'data, T, N> {
        ValueAccessor::new(&self.data)
    }

    /// Create an accessor for indeterminate data.
    pub fn indeterminate_data<'borrow>(
        &'borrow self,
    ) -> IndeterminateAccessor<'borrow, 'scope, 'data, T, N> {
        unsafe { IndeterminateAccessor::new(&self.data) }
    }
}

impl<'scope, 'data, T, const N: isize> TrackedArrayBase<'scope, 'data, T, N> {
    pub(crate) fn track_shared(array: ArrayBase<'scope, 'data, T, N>) -> JlrsResult<Self> {
        unsafe {
            let mut array_v = array.as_value();
            if array.how() == How::PointerToOwner {
                let owner = jlrs_array_data_owner(array.unwrap(Private));
                array_v = Value::wrap_non_null(NonNull::new_unchecked(owner), Private);
            }

            let success = Ledger::try_borrow_shared(array_v)?;
            assert!(success);

            Ok(TrackedArrayBase { data: array })
        }
    }
}

impl<'scope, 'data, T, const N: isize> Clone for TrackedArrayBase<'scope, 'data, T, N> {
    fn clone(&self) -> Self {
        unsafe {
            let array = self.data;
            let mut array_v = array.as_value();
            if array.how() == How::PointerToOwner {
                let owner = jlrs_array_data_owner(array.unwrap(Private));
                array_v = Value::wrap_non_null(NonNull::new_unchecked(owner), Private);
            }

            Ledger::borrow_shared_unchecked(array_v).unwrap();
        }
        Self { data: self.data }
    }
}

impl<'scope, 'data, T, const N: isize> Deref for TrackedArrayBase<'scope, 'data, T, N> {
    type Target = ArrayBase<'scope, 'data, T, N>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T, const N: isize> Drop for TrackedArrayBase<'_, '_, T, N> {
    fn drop(&mut self) {
        unsafe {
            let array = self.data;
            let mut array_v = array.as_value();
            if array.how() == How::PointerToOwner {
                let owner = jlrs_array_data_owner(array.unwrap(Private));
                array_v = Value::wrap_non_null(NonNull::new_unchecked(owner), Private);
            }

            let _success = Ledger::unborrow_shared(array_v).expect("Failed to untrack shared");
        }
    }
}

/// A tracked array that provides mutable access
#[repr(transparent)]
pub struct TrackedArrayBaseMut<'scope, 'data, T, const N: isize> {
    data: ArrayBase<'scope, 'data, T, N>,
}

impl<'scope, 'data, T, const N: isize> TrackedArrayBaseMut<'scope, 'data, T, N> {
    /// Create a mutable accessor for `isbits` data.
    ///
    /// Thanks to the restrictions on `T` the data is guaranteed to be stored inline as an array
    /// of `T`s.
    ///
    /// Safety:
    ///
    /// Mutating Julia data is generally unsafe. You must guarantee that you're allowed to mutate
    /// its content, and that no running Julia code is accessing this data.
    pub unsafe fn bits_data_mut<'borrow>(
        &'borrow mut self,
    ) -> BitsAccessorMut<'borrow, 'scope, 'data, T, T, N>
    where
        T: ConstructType + ValidField + IsBits,
    {
        // No need for checks, guaranteed to have isbits layout
        BitsAccessorMut::new(&mut self.data)
    }

    /// Create a mutable accessor for `isbits` data with layout `L`.
    ///
    /// Thanks to the restrictions on `T` and `L` the elements are guaranteed to be stored inline
    /// as an array of `L`s.
    ///
    /// Safety:
    ///
    /// Mutating Julia data is generally unsafe. You must guarantee that you're allowed to mutate
    /// its content, and that no running Julia code is accessing this data.
    pub unsafe fn bits_data_mut_with_layout<'borrow, L>(
        &'borrow mut self,
    ) -> BitsAccessorMut<'borrow, 'scope, 'data, T, L, N>
    where
        T: HasLayout<'static, 'static, Layout = L>,
        L: IsBits + ValidField,
    {
        // No need for checks, guaranteed to have isbits layout and L is the layout of T
        BitsAccessorMut::new(&mut self.data)
    }

    /// Try to create a mutable accessor for `isbits` data with layout `L`.
    ///
    /// If the array doesn't have an isbits layout `ArrayLayoutError::NotBits` is returned. If `L`
    /// is not a valid field layout for the element type `TypeError::InvalidLayout` is returned.
    ///
    /// Safety:
    ///
    /// Mutating Julia data is generally unsafe. You must guarantee that you're allowed to mutate
    /// its content, and that no running Julia code is accessing this data.
    pub unsafe fn try_bits_data_mut<'borrow, L>(
        &'borrow mut self,
    ) -> JlrsResult<BitsAccessorMut<'borrow, 'scope, 'data, T, L, N>>
    where
        L: IsBits + ValidField,
    {
        if !self.has_bits_layout() {
            Err(ArrayLayoutError::NotBits {
                element_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        let ty = self.element_type();
        if !L::valid_field(ty) {
            Err(TypeError::InvalidLayout {
                value_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        Ok(BitsAccessorMut::new(&mut self.data))
    }

    /// Create a mutable accessor for `isbits` data with layout `L` without checking any
    /// invariants.
    ///
    /// Safety:
    ///
    /// Mutating Julia data is generally unsafe. You must guarantee that you're allowed to mutate
    /// its content, and that no running Julia code is accessing this data. The element type must
    /// be an isbits type, and `L` must be a valid field layout of the element type.
    pub unsafe fn bits_data_mut_unchecked<'borrow, L>(
        &'borrow mut self,
    ) -> BitsAccessorMut<'borrow, 'scope, 'data, T, L, N>
    where
        L: IsBits + ValidField,
    {
        BitsAccessorMut::new(&mut self.data)
    }

    /// Create a mutable accessor for inline data.
    ///
    /// Thanks to the restrictions on `T` the data is guaranteed to be stored inline as an array
    /// of `T`s.
    ///
    /// Safety:
    ///
    /// Mutating Julia data is generally unsafe. You must guarantee that you're allowed to mutate
    /// its content, and that no running Julia code is accessing this data.
    pub unsafe fn inline_data_mut<'borrow>(
        &'borrow mut self,
    ) -> InlineAccessorMut<'borrow, 'scope, 'data, T, T, N>
    where
        T: ConstructType + ValidField,
    {
        // No need for checks, guaranteed to have inline layout
        InlineAccessorMut::new(&mut self.data)
    }

    /// Create a mutable accessor for inline data with layout `L`.
    ///
    /// Thanks to the restrictions on `T` and `L` the elements are guaranteed to be stored inline
    /// as an array of `L`s.
    ///
    /// Safety:
    ///
    /// Mutating Julia data is generally unsafe. You must guarantee that you're allowed to mutate
    /// its content, and that no running Julia code is accessing this data.
    pub unsafe fn inline_data_mut_with_layout<'borrow, L>(
        &'borrow mut self,
    ) -> InlineAccessorMut<'borrow, 'scope, 'data, T, L, N>
    where
        T: ConstructType + HasLayout<'scope, 'data, Layout = L>,
        L: ValidField,
    {
        // No need for checks, guaranteed to have inline layout and L is the layout of T
        InlineAccessorMut::new(&mut self.data)
    }

    /// Try to create a mutable accessor for inline data with layout `L`.
    ///
    /// If the array doesn't have an inline layout `ArrayLayoutError::NotInline` is returned. If
    /// `L` is not a valid field layout for the element type `TypeError::InvalidLayout` is
    /// returned.
    ///
    /// Safety:
    ///
    /// Mutating Julia data is generally unsafe. You must guarantee that you're allowed to mutate
    /// its content, and that no running Julia code is accessing this data.
    pub unsafe fn try_inline_data_mut<'borrow, L>(
        &'borrow mut self,
    ) -> JlrsResult<InlineAccessorMut<'borrow, 'scope, 'data, T, L, N>>
    where
        L: ValidField,
    {
        if !self.has_inline_layout() {
            Err(ArrayLayoutError::NotInline {
                element_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        let ty = self.element_type();
        if !L::valid_field(ty) {
            Err(TypeError::InvalidLayout {
                value_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        Ok(InlineAccessorMut::new(&mut self.data))
    }

    /// Create a mutable  accessor for inline data with layout `L` without checking any
    /// invariants.
    ///
    /// Safety:
    ///
    /// Mutating Julia data is generally unsafe. You must guarantee that you're allowed to mutate
    /// its content, and that no running Julia code is accessing this data. The elements must be
    /// stored inline, and `L` must be a valid field layout of the element type.
    pub unsafe fn inline_data_mut_unchecked<'borrow, L>(
        &'borrow mut self,
    ) -> InlineAccessorMut<'borrow, 'scope, 'data, T, L, N>
    where
        L: ValidField,
    {
        InlineAccessorMut::new(&mut self.data)
    }

    /// Create a mutable accessor for unions of isbits types.
    ///
    /// This function panics if the array doesn't have a union layout.
    ///
    /// Safety:
    ///
    /// Mutating Julia data is generally unsafe. You must guarantee that you're allowed to mutate
    /// its content, and that no running Julia code is accessing this data.
    pub unsafe fn union_data_mut<'borrow>(
        &'borrow mut self,
    ) -> BitsUnionAccessorMut<'borrow, 'scope, 'data, T, N>
    where
        T: BitsUnionCtor,
    {
        assert!(
            self.has_union_layout(),
            "Array does not have a union layout"
        );
        BitsUnionAccessorMut::new(&mut self.data)
    }

    /// Try to create a mutable accessor for unions of isbits types.
    ///
    /// If the element type is not a union of isbits types `ArrayLayoutError::NotUnion` is
    /// returned.
    ///
    /// Safety:
    ///
    /// Mutating Julia data is generally unsafe. You must guarantee that you're allowed to mutate
    /// its content, and that no running Julia code is accessing this data.
    pub unsafe fn try_union_data_mut<'borrow>(
        &'borrow mut self,
    ) -> JlrsResult<BitsUnionAccessorMut<'borrow, 'scope, 'data, T, N>> {
        if !self.has_union_layout() {
            Err(ArrayLayoutError::NotUnion {
                element_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        Ok(BitsUnionAccessorMut::new(&mut self.data))
    }

    /// Create a mutable accessor for unions of isbits types without checking any invariants.
    ///
    /// Safety:
    ///
    /// Mutating Julia data is generally unsafe. You must guarantee that you're allowed to mutate
    /// its content, and that no running Julia code is accessing this data. The element type must
    /// be a union of isbits types.
    pub unsafe fn union_data_mut_unchecked<'borrow>(
        &'borrow mut self,
    ) -> BitsUnionAccessorMut<'borrow, 'scope, 'data, T, N> {
        BitsUnionAccessorMut::new(&mut self.data)
    }

    /// Create a mutable accessor for managed data.
    ///
    /// Thanks to the restrictions on `T` the data is guaranteed to be as an array of
    /// `Option<Ref<T>>`s.
    ///
    /// Safety:
    ///
    /// Mutating Julia data is generally unsafe. You must guarantee that you're allowed to mutate
    /// its content, and that no running Julia code is accessing this data.
    pub unsafe fn managed_data_mut<'borrow>(
        &'borrow mut self,
    ) -> ManagedAccessorMut<'borrow, 'scope, 'data, T, T, N>
    where
        T: Managed<'scope, 'data> + ConstructType,
    {
        // No need for checks, guaranteed to have correct layout
        ManagedAccessorMut::new(&mut self.data)
    }

    /// Try to create a mutable accessor for managed data of type `L`.
    ///
    /// If the element type is incompatible with `L` `ArrayLayoutError::NotManaged` is returned.
    ///
    /// Safety:
    ///
    /// Mutating Julia data is generally unsafe. You must guarantee that you're allowed to mutate
    /// its content, and that no running Julia code is accessing this data.
    pub unsafe fn try_managed_data_mut<'borrow, L>(
        &'borrow mut self,
    ) -> JlrsResult<ManagedAccessorMut<'borrow, 'scope, 'data, T, L, N>>
    where
        L: Managed<'scope, 'data> + Typecheck,
    {
        if !self.has_managed_layout::<L>() {
            Err(ArrayLayoutError::NotManaged {
                element_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
                name: L::NAME.into(),
            })?;
        }

        Ok(ManagedAccessorMut::new(&mut self.data))
    }

    /// Create a mutable accessor for managed data of type `L` without checking any invariants.
    ///
    /// Safety:
    ///
    /// Mutating Julia data is generally unsafe. You must guarantee that you're allowed to mutate
    /// its content, and that no running Julia code is accessing this data. The element type must
    /// be compatible with `L`.
    pub unsafe fn managed_data_mut_unchecked<'borrow, L>(
        &'borrow mut self,
    ) -> ManagedAccessorMut<'borrow, 'scope, 'data, T, L, N>
    where
        L: Managed<'scope, 'data>,
    {
        ManagedAccessorMut::new(&mut self.data)
    }

    /// Create a mutable accessor for value data.
    ///
    /// Thanks to the restrictions on `T` the data is guaranteed to be as an array of
    /// `Option<Ref<Value>>`s.
    ///
    /// Safety:
    ///
    /// Mutating Julia data is generally unsafe. You must guarantee that you're allowed to mutate
    /// its content, and that no running Julia code is accessing this data.
    pub unsafe fn value_data_mut<'borrow>(
        &'borrow mut self,
    ) -> ValueAccessorMut<'borrow, 'scope, 'data, T, N>
    where
        T: Managed<'scope, 'data> + ConstructType,
    {
        // No need for checks, guaranteed to have inline layout
        ValueAccessorMut::new(&mut self.data)
    }

    /// Try to create a mutable accessor for value data.
    ///
    /// If the elements are stored inline `ArrayLayoutError::NotPointer` is returned.
    ///
    /// Safety:
    ///
    /// Mutating Julia data is generally unsafe. You must guarantee that you're allowed to mutate
    /// its content, and that no running Julia code is accessing this data.
    pub unsafe fn try_value_data_mut<'borrow>(
        &'borrow mut self,
    ) -> JlrsResult<ValueAccessorMut<'borrow, 'scope, 'data, T, N>> {
        if !self.has_value_layout() {
            Err(ArrayLayoutError::NotPointer {
                element_type: self.element_type().error_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        Ok(ValueAccessorMut::new(&mut self.data))
    }

    /// Create a mutable accessor for managed data of type `L` without checking any invariants.
    ///
    /// Safety:
    ///
    /// Mutating Julia data is generally unsafe. You must guarantee that you're allowed to mutate
    /// its content, and that no running Julia code is accessing this data. The elements must not
    /// be stored inline.
    pub unsafe fn value_data_mut_unchecked<'borrow>(
        &'borrow mut self,
    ) -> ValueAccessorMut<'borrow, 'scope, 'data, T, N> {
        ValueAccessorMut::new(&mut self.data)
    }

    /// Create a mutable accessor for indeterminate data.
    ///
    /// Safety:
    ///
    /// Mutating Julia data is generally unsafe. You must guarantee that you're allowed to mutate
    /// its content, and that no running Julia code is accessing this data.
    pub unsafe fn indeterminate_data_mut<'borrow>(
        &'borrow mut self,
    ) -> IndeterminateAccessorMut<'borrow, 'scope, 'data, T, N> {
        IndeterminateAccessorMut::new(&mut self.data)
    }
}

impl<'scope, 'data, T, const N: isize> TrackedArrayBaseMut<'scope, 'data, T, N> {
    pub(crate) fn track_exclusive(array: ArrayBase<'scope, 'data, T, N>) -> JlrsResult<Self> {
        unsafe {
            let mut array_v = array.as_value();
            if array.how() == How::PointerToOwner {
                let owner = jlrs_array_data_owner(array.unwrap(Private));
                array_v = Value::wrap_non_null(NonNull::new_unchecked(owner), Private);
            }

            let success = Ledger::try_borrow_exclusive(array_v)?;
            assert!(success);

            Ok(TrackedArrayBaseMut { data: array })
        }
    }
}

impl<'scope, 'data, T, const N: isize> Deref for TrackedArrayBaseMut<'scope, 'data, T, N> {
    type Target = TrackedArrayBase<'scope, 'data, T, N>;

    fn deref(&self) -> &Self::Target {
        unsafe { std::mem::transmute(self) }
    }
}

impl<T, const N: isize> Drop for TrackedArrayBaseMut<'_, '_, T, N> {
    fn drop(&mut self) {
        unsafe {
            let array = self.data;
            let mut array_v = array.as_value();
            if array.how() == How::PointerToOwner {
                let owner = jlrs_array_data_owner(array.unwrap(Private));
                array_v = Value::wrap_non_null(NonNull::new_unchecked(owner), Private);
            }

            let success = Ledger::unborrow_exclusive(array_v).expect("Failed to untrack shared");
            assert!(success);
        }
    }
}

pub type TrackedArray<'scope, 'data> = TrackedArrayBase<'scope, 'data, Unknown, -1>;
pub type TrackedTypedArray<'scope, 'data, T> = TrackedArrayBase<'scope, 'data, T, -1>;
pub type TrackedRankedArray<'scope, 'data, const N: isize> =
    TrackedArrayBase<'scope, 'data, Unknown, N>;
pub type TrackedTypedRankedArray<'scope, 'data, T, const N: isize> =
    TrackedArrayBase<'scope, 'data, T, N>;

pub type TrackedVector<'scope, 'data> = TrackedArrayBase<'scope, 'data, Unknown, 1>;
pub type TrackedTypedVector<'scope, 'data, T> = TrackedArrayBase<'scope, 'data, T, 1>;

pub type TrackedMatrix<'scope, 'data> = TrackedArrayBase<'scope, 'data, Unknown, 2>;
pub type TrackedTypedMatrix<'scope, 'data, T> = TrackedArrayBase<'scope, 'data, T, 2>;

pub type TrackedArrayMut<'scope, 'data> = TrackedArrayBaseMut<'scope, 'data, Unknown, -1>;
pub type TrackedTypedArrayMut<'scope, 'data, T> = TrackedArrayBaseMut<'scope, 'data, T, -1>;
pub type TrackedRankedArrayMut<'scope, 'data, const N: isize> =
    TrackedArrayBaseMut<'scope, 'data, Unknown, N>;
pub type TrackedTypedRankedArrayMut<'scope, 'data, T, const N: isize> =
    TrackedArrayBaseMut<'scope, 'data, T, N>;

pub type TrackedVectorMut<'scope, 'data> = TrackedArrayBaseMut<'scope, 'data, Unknown, 1>;
pub type TrackedTypedVectorMut<'scope, 'data, T> = TrackedArrayBaseMut<'scope, 'data, T, 1>;

pub type TrackedMatrixMut<'scope, 'data> = TrackedArrayBaseMut<'scope, 'data, Unknown, 2>;
pub type TrackedTypedMatrixMut<'scope, 'data, T> = TrackedArrayBaseMut<'scope, 'data, T, 2>;
