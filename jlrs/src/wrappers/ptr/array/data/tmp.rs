

impl<'scope, 'data> Array<'scope, 'data> {
    /// Returns the array's dimensions.
    pub fn dimensions<'borrow, 'frame: 'borrow, F>(self, frame: &F) -> JlrsResult<ArrayDimensionsRef<'borrow, 'scope, 'data>> 
    where 
        F: Frame<'frame> 
    {

        ArrayDimensions::new(self)
    }
    /// Returns the array's dimensions.
    pub fn dimensions_untracked<'frame, F>(self) -> ArrayDimensions<'scope>
    where 
        F: Frame<'frame> 
    {

        ArrayDimensions::new(self)
    }

    /// Returns the type of this array's elements.
    pub fn element_type(self) -> Value<'scope, 'static> {
        // Safety: C API function is called valid arguments.
        unsafe { Value::wrap(jl_array_eltype(self.unwrap(Private).cast()).cast(), Private) }
    }

    /// Returns the size of this array's elements.
    pub fn element_size(self) -> usize {
        // Safety: the pointer points to valid data.
        unsafe { self.unwrap_non_null(Private).as_ref().elsize as usize }
    }

    /// Returns `true` if the layout of the elements is compatible with `T`.
    pub fn contains<T: ValidLayout>(self) -> bool {
        // Safety: C API function is called valid arguments.
        unsafe {
            T::valid_layout(Value::wrap(
                jl_array_eltype(self.unwrap(Private).cast()).cast(),
                Private,
            ))
        }
    }

    /// Returns `true` if the layout of the elements is compatible with `T` and these elements are
    /// stored inline.
    pub fn contains_inline<T: ValidLayout>(self) -> bool {
        self.contains::<T>() && self.is_inline_array()
    }

    /// Returns `true` if the elements of the array are stored inline.
    pub fn is_inline_array(self) -> bool {
        // Safety: the pointer points to valid data.
        unsafe { self.unwrap_non_null(Private).as_ref().flags.ptrarray() == 0 }
    }

    /// Returns `true` if the elements of the array are stored inline and the element type is a
    /// union type.
    pub fn is_union_array(self) -> bool {
        self.is_inline_array() && self.element_type().is::<Union>()
    }

    /// Returns true if the elements of the array are stored inline and at least one of the fields
    /// of the inlined type is a pointer.
    pub fn has_inlined_pointers(self) -> bool {
        // Safety: the pointer points to valid data.
        unsafe {
            let flags = self.unwrap_non_null(Private).as_ref().flags;
            self.is_inline_array() && flags.hasptr() != 0
        }
    }

    /// Returns `true` if elements of this array are zero-initialized.
    pub fn zero_init(self) -> bool {
        // Safety: the pointer points to valid data.
        unsafe {
            let flags = self.unwrap_non_null(Private).as_ref().flags;
            if flags.ptrarray() == 1 || flags.hasptr() == 1 {
                return true;
            }

            let elty = self.element_type();
            if let Ok(dt) = elty.cast::<DataType>() {
                return dt.zero_init();
            } else {
                false
            }
        }
    }

    /// Returns true if the elements of the array are stored as [`Value`]s.
    pub fn is_value_array(self) -> bool {
        !self.is_inline_array()
    }

    /// Convert this untyped array to a [`TypedArray`].
    pub fn try_as_typed<T>(self) -> JlrsResult<TypedArray<'scope, 'data, T>>
    where
        T: Clone + ValidLayout + Debug,
    {
        if self.contains::<T>() {
            let ptr = self.unwrap_non_null(Private);
            // Safety: the type is correct
            unsafe { Ok(TypedArray::wrap_non_null(ptr, Private)) }
        } else {
            let value_type_str = self.element_type().display_string_or(CANNOT_DISPLAY_TYPE);
            Err(AccessError::InvalidLayout { value_type_str })?
        }
    }

    /// Convert this untyped array to a [`TypedArray`] without checking if this conversion is
    /// valid.
    ///
    /// Safety: `T` must be a valid representation of the data stored in the array.
    pub unsafe fn as_typed_unchecked<T>(self) -> TypedArray<'scope, 'data, T>
    where
        T: Clone + ValidLayout,
    {
        TypedArray::wrap_non_null(self.unwrap_non_null(Private), Private)
    }

    /// Copy the data of an inline array to Rust.
    ///
    /// Returns `ArrayLayoutError::NotInline` if the data is not stored inline or `AccessError::InvalidLayout`
    /// if the type of the elements is incorrect.
    pub fn copy_inline_data<'frame, T, F>(self, _: &F) -> JlrsResult<CopiedArray<T>>
    where
        T: 'static + ValidLayout,
        F: Frame<'frame>,
    {
        if !self.contains::<T>() {
            Err(AccessError::InvalidLayout {
                value_type_str: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        if !self.is_inline_array() {
            Err(ArrayLayoutError::NotInline {
                element_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        let dimensions = self.dimensions().into_dimensions();
        let sz = dimensions.size();
        let mut data = Vec::with_capacity(sz);

        // Safety: layouts are compatible and is guaranteed to be a bits type due to the
        // 'static constraint on T.
        unsafe {
            let jl_data = jl_array_data(self.unwrap(Private).cast()).cast();
            let ptr = data.as_mut_ptr();
            std::ptr::copy_nonoverlapping(jl_data, ptr, sz);
            data.set_len(sz);

            Ok(CopiedArray::new(data.into_boxed_slice(), dimensions))
        }
    }























    /// Immutably access the contents of this array. 
    /// 
    /// The elements must have an `isbits` type.
    /// 
    /// Errors:
    ///  - `ArrayLayoutError::NotInline` if the data is not stored inline, 
    ///  - `ArrayLayoutError::NotBits` if `T` is not an `isbits` type,
    ///  - `AccessError::InvalidLayout` if `T` is not a valid layout for the array elements. 
    ///  - `AccessError::BorrowError` if the array is already mutably borrowed.
    pub fn bits_data<'borrow, 'frame: 'borrow, T, F>(
        &'borrow self,
        frame: &F,
    ) -> JlrsResult<BitsArrayAccessor<'borrow, 'scope, 'data, T>>
    where
        T: ValidLayout,
        F: Frame<'frame>,
    {
        self.ensure_bits_containing::<T>()?;

        // Safety: layouts are compatible, access is immutable.
        let accessor = unsafe { ArrayAccessor::new2(self) };
        Ledger::try_borrow(frame.ledger(), accessor)
    }

    /// Immutably access the contents of this array without tracking the borrow. 
    /// 
    /// The elements must have an `isbits` type.
    /// 
    /// Errors:
    ///  - `ArrayLayoutError::NotInline` if the data is not stored inline, 
    ///  - `ArrayLayoutError::NotBits` if `T` is not an `isbits` type,
    ///  - `AccessError::InvalidLayout` if `T` is not a valid layout for the array elements. 
    /// 
    /// Safety: This method doesn't track the borrow, mutable aliasing is not prevented. 
    pub unsafe fn bits_data_untracked<'borrow, T>(
        &'borrow self,
    ) -> JlrsResult<BitsArrayAccessor<'borrow, 'scope, 'data, T>>
    where
        T: ValidLayout,
    {
        self.ensure_bits_containing::<T>()?;

        // Safety: layouts are compatible, access is immutable.
        let accessor = ArrayAccessor::new2(self);
        Ok(BitsArrayAccessorI::new(None, accessor))
    }

    /// Mutably access the contents of this array. 
    /// 
    /// The elements must have an `isbits` type.
    /// 
    /// Errors:
    ///  - `ArrayLayoutError::NotInline` if the data is not stored inline, 
    ///  - `ArrayLayoutError::NotBits` if `T` is not an `isbits` type,
    ///  - `AccessError::InvalidLayout` if `T` is not a valid layout for the array elements. 
    ///  - `AccessError::BorrowError` if the array is already mutably borrowed.
    pub fn bits_data_mut<'borrow, 'frame: 'borrow, T, F>(
        &'borrow mut self,
        frame: &F,
    ) -> JlrsResult<BitsArrayAccessorMut<'borrow, 'scope, 'data, T>>
    where
        T: ValidLayout,
        F: Frame<'frame>,
    {
        self.ensure_bits_containing::<T>()?;

        let accessor = unsafe { ArrayAccessor::new2(self) };
        Ledger::try_borrow_mut(frame.ledger(), accessor)
    }

    /// Mutably access the contents of this array without tracking the borrow. 
    /// 
    /// The elements must have an `isbits` type.
    /// 
    /// Errors:
    ///  - `ArrayLayoutError::NotInline` if the data is not stored inline, 
    ///  - `ArrayLayoutError::NotBits` if `T` is not an `isbits` type,
    ///  - `AccessError::InvalidLayout` if `T` is not a valid layout for the array elements. 
    /// 
    /// Safety: This method doesn't track the borrow, mutable aliasing is not prevented. 
    pub unsafe fn bits_data_mut_untracked<'borrow, T>(
        &'borrow mut self,
    ) -> JlrsResult<BitsArrayAccessorMut<'borrow, 'scope, 'data, T>>
    where
        T: ValidLayout,
    {
        self.ensure_bits_containing::<T>()?;

        let accessor = ArrayAccessor::new2(self);
        Ok(BitsArrayAccessorMut::new(None, accessor))
    }

    /// Immutably access the contents of this array. 
    /// 
    /// The elements must be stored inline.
    /// 
    /// Errors:
    ///  - `ArrayLayoutError::NotInline` if the data is not stored inline, 
    ///  - `AccessError::InvalidLayout` if `T` is not a valid layout for the array elements. 
    ///  - `AccessError::BorrowError` if the array is already mutably borrowed.
    pub fn inline_data<'borrow, 'frame: 'borrow, T, F>(
        &'borrow self,
        frame: &F,
    ) -> JlrsResult<InlineArrayAccessor<'borrow, 'scope, 'data, T>>
    where
        T: ValidLayout,
        F: Frame<'frame>,
    {
        self.ensure_inline_containing::<T>()?;

        // Safety: layouts are compatible, access is immutable.
        let accessor = unsafe { ArrayAccessor::new2(self) };
        Ledger::try_borrow(frame.ledger(), accessor)
    }

    /// Immutably access the contents of this array without tracking the borrow. 
    /// 
    /// The elements must be stored inline.
    /// 
    /// Errors:
    ///  - `ArrayLayoutError::NotInline` if the data is not stored inline, 
    ///  - `AccessError::InvalidLayout` if `T` is not a valid layout for the array elements. 
    /// 
    /// Safety: This method doesn't track the borrow, mutable aliasing is not prevented. 
    pub unsafe fn inline_data_untracked<'borrow, T>(
        &'borrow self,
    ) -> JlrsResult<InlineArrayAccessor<'borrow, 'scope, 'data, T>>
    where
        T: ValidLayout,
    {
        self.ensure_inline_containing::<T>()?;

        // Safety: layouts are compatible, access is immutable.
        let accessor = ArrayAccessor::new2(self);
        Ok(InlineArrayAccessorI::new(None, accessor))
    }

    /// Mutably access the contents of this array. 
    /// 
    /// The elements must be stored inline.
    /// 
    /// Errors:
    ///  - `ArrayLayoutError::NotInline` if the data is not stored inline.
    ///  - `AccessError::InvalidLayout` if `T` is not a valid layout for the array elements. 
    ///  - `AccessError::BorrowError` if the array is already mutably borrowed.
    pub fn inline_data_mut<'borrow, 'frame: 'borrow, T, F>(
        &'borrow mut self,
        frame: &F,
    ) -> JlrsResult<InlineArrayAccessorMut<'borrow, 'scope, 'data, T>>
    where
        T: ValidLayout,
        F: Frame<'frame>,
    {
        self.ensure_inline_containing::<T>()?;

        let accessor = unsafe { ArrayAccessor::new2(self) };
        Ledger::try_borrow_mut(frame.ledger(), accessor)
    }

    /// Mutably access the contents of this array without tracking the borrow. 
    /// 
    /// The elements must be stored inline.
    /// 
    /// Errors:
    ///  - `ArrayLayoutError::NotInline` if the data is not stored inline.
    ///  - `AccessError::InvalidLayout` if `T` is not a valid layout for the array elements. 
    /// 
    /// Safety: This method doesn't track the borrow, mutable aliasing is not prevented.
    pub unsafe fn inline_data_mut_untracked<'borrow, T>(
        &'borrow mut self,
    ) -> JlrsResult<InlineArrayAccessorMut<'borrow, 'scope, 'data, T>>
    where
        T: ValidLayout,
    {
        self.ensure_inline_containing::<T>()?;

        let accessor = ArrayAccessor::new2(self);
        Ok(InlineArrayAccessorMut::new(None, accessor))
    }

    /// Immutably access the contents of this array. 
    /// 
    /// The elements must not be stored inline.
    /// 
    /// Errors:
    ///  - `ArrayLayoutError::NotPointer` if the data is stored inline.
    ///  - `AccessError::InvalidLayout` if `T` is not a valid layout for the array elements.
    ///  - `AccessError::BorrowError` if the array is already mutably borrowed.
    pub fn wrapper_data<'borrow, 'frame: 'borrow, T, F>(
        &'borrow self,
        frame: &F,
    ) -> JlrsResult<PtrArrayAccessor<'borrow, 'scope, 'data, T>>
    where
        T: WrapperRef<'scope, 'data>,
        F: Frame<'frame>,
    {
        self.ensure_ptr_containing::<T>()?;

        // Safety: layouts are compatible, access is immutable.
        let accessor = unsafe { ArrayAccessor::new2(self) };
        Ledger::try_borrow(frame.ledger(), accessor)
    }

    /// Immutably access the contents of this array without tracking the borrow. 
    /// 
    /// The elements must not be stored inline.
    /// 
    /// Errors:
    ///  - `ArrayLayoutError::NotPointer` if the data is stored inline,
    ///  - `AccessError::InvalidLayout` if `T` is not a valid layout for the array elements. 
    /// 
    /// Safety: This method doesn't track the borrow, mutable aliasing is not prevented. 
    pub unsafe fn wrapper_data_untracked<'borrow, T>(
        &'borrow self,
    ) -> JlrsResult<PtrArrayAccessor<'borrow, 'scope, 'data, T>>
    where
        T: WrapperRef<'scope, 'data>,
    {
        self.ensure_ptr_containing::<T>()?;

        // Safety: layouts are compatible, access is immutable.
        let accessor = ArrayAccessor::new2(self);
        Ok(PtrArrayAccessorI::new(None, accessor))
    }

    /// Mutably access the contents of this array. 
    /// 
    /// The elements must not be stored inline.
    /// 
    /// Errors:
    ///  - `ArrayLayoutError::NotPointer` if the data is stored inline.
    ///  - `AccessError::InvalidLayout` if `T` is not a valid layout for the array elements.
    ///  - `AccessError::BorrowError` if the array is already mutably borrowed.
    pub fn wrapper_data_mut<'borrow, 'frame: 'borrow, T, F>(
        &'borrow mut self,
        frame: &F,
    ) -> JlrsResult<PtrArrayAccessorMut<'borrow, 'scope, 'data, T>>
    where
        T: WrapperRef<'scope, 'data>,
        F: Frame<'frame>,
    {
        self.ensure_ptr_containing::<T>()?;

        let accessor = unsafe { ArrayAccessor::new2(self) };
        Ledger::try_borrow_mut(frame.ledger(), accessor)
    }

    /// Mutably access the contents of this array without tracking the borrow. 
    /// 
    /// The elements must not be stored inline.
    /// 
    /// Errors:
    ///  - `ArrayLayoutError::NotPointer` if the data is stored inline,
    ///  - `AccessError::InvalidLayout` if `T` is not a valid layout for the array elements. 
    /// 
    /// Safety: This method doesn't track the borrow, mutable aliasing is not prevented. 
    pub unsafe fn wrapper_data_mut_untracked<'borrow, T>(
        &'borrow mut self,
    ) -> JlrsResult<PtrArrayAccessorMut<'borrow, 'scope, 'data, T>>
    where
        T: WrapperRef<'scope, 'data>,
    {
        self.ensure_ptr_containing::<T>()?;

        let accessor = ArrayAccessor::new2(self);
        Ok(PtrArrayAccessorMut::new(None, accessor))
    }
    
    /// Immutably access the contents of this array. 
    /// 
    /// The elements must not be stored inline.
    /// 
    /// Errors:
    ///  - `ArrayLayoutError::NotPointer` if the data is stored inline.
    ///  - `AccessError::BorrowError` if the array is already mutably borrowed.
    pub fn value_data<'borrow, 'frame: 'borrow, F>(
        &'borrow self,
        frame: &F,
    ) -> JlrsResult<PtrArrayAccessor<'borrow, 'scope, 'data, ValueRef<'scope, 'data>>>
    where
        F: Frame<'frame>,
    {
        self.ensure_ptr_containing::<ValueRef>()?;

        // Safety: layouts are compatible, access is immutable.
        let accessor = unsafe { ArrayAccessor::new2(self) };
        Ledger::try_borrow(frame.ledger(), accessor)
    }

    /// Immutably access the contents of this array without tracking the borrow. 
    /// 
    /// The elements must not be stored inline.
    /// 
    /// Errors:
    ///  - `ArrayLayoutError::NotPointer` if the data is stored inline,
    /// 
    /// Safety: This method doesn't track the borrow, mutable aliasing is not prevented. 
    pub unsafe fn value_data_untracked<'borrow>(
        &'borrow self,
    ) -> JlrsResult<PtrArrayAccessor<'borrow, 'scope, 'data, ValueRef<'scope, 'data>>>
    {
        self.ensure_ptr_containing::<ValueRef>()?;

        // Safety: layouts are compatible, access is immutable.
        let accessor = ArrayAccessor::new2(self);
        Ok(PtrArrayAccessorI::new(None, accessor))
    }

    /// Mutably access the contents of this array. 
    /// 
    /// The elements must not be stored inline.
    /// 
    /// Errors:
    ///  - `ArrayLayoutError::NotPointer` if the data is stored inline.
    ///  - `AccessError::BorrowError` if the array is already mutably borrowed.
    pub fn value_data_mut<'borrow, 'frame: 'borrow, F>(
        &'borrow mut self,
        frame: &F,
    ) -> JlrsResult<PtrArrayAccessorMut<'borrow, 'scope, 'data, ValueRef<'scope, 'data>>>
    where
        F: Frame<'frame>,
    {
        self.ensure_ptr_containing::<ValueRef>()?;

        let accessor = unsafe { ArrayAccessor::new2(self) };
        Ledger::try_borrow_mut(frame.ledger(), accessor)
    }

    /// Mutably access the contents of this array without tracking the borrow. 
    /// 
    /// The elements must not be stored inline.
    /// 
    /// Errors:
    ///  - `ArrayLayoutError::NotPointer` if the data is stored inline.
    /// 
    /// Safety: This method doesn't track the borrow, mutable aliasing is not prevented. 
    pub unsafe fn value_data_mut_untracked<'borrow>(
        &'borrow mut self,
    ) -> JlrsResult<PtrArrayAccessorMut<'borrow, 'scope, 'data, ValueRef<'scope, 'data>>>
    {
        self.ensure_ptr_containing::<ValueRef>()?;

        let accessor = ArrayAccessor::new2(self);
        Ok(PtrArrayAccessorMut::new(None, accessor))
    }
    
    /// Immutably access the contents of this array. 
    /// 
    /// The element type must be a bits union.
    /// 
    /// Errors:
    ///  - `ArrayLayoutError::NotUnion` if the data isn't a bits union.
    ///  - `AccessError::BorrowError` if the array is already mutably borrowed.
    pub fn union_data<'borrow, 'frame: 'borrow, F>(
        &'borrow self,
        frame: &F,
    ) -> JlrsResult<UnionArrayAccessor<'borrow, 'scope, 'data>>
    where
        F: Frame<'frame>,
    {
        self.ensure_union()?;

        // Safety: layouts are compatible, access is immutable.
        let accessor = unsafe { ArrayAccessor::new2(self) };
        Ledger::try_borrow(frame.ledger(), accessor)
    }
    
    /// Immutably access the contents of this array. 
    /// 
    /// The element type must be a bits union.
    /// 
    /// Errors:
    ///  - `ArrayLayoutError::NotUnion` if the data isn't a bits union.
    /// 
    /// Safety: This method doesn't track the borrow, mutable aliasing is not prevented. 
    pub unsafe fn union_data_untracked<'borrow>(
        &'borrow self,
    ) -> JlrsResult<UnionArrayAccessor<'borrow, 'scope, 'data>>
    {
        self.ensure_union()?;

        // Safety: layouts are compatible, access is immutable.
        let accessor = ArrayAccessor::new2(self);
        Ok(UnionArrayAccessor::new(None, accessor))
    }
    
    /// Mutably access the contents of this array. 
    /// 
    /// The element type must be a bits union.
    /// 
    /// Errors:
    ///  - `ArrayLayoutError::NotUnion` if the data isn't a bits union.
    ///  - `AccessError::BorrowError` if the array is already mutably borrowed.
    pub fn union_data_mut<'borrow, 'frame: 'borrow, F>(
        &'borrow mut self,
        frame: &F,
    ) -> JlrsResult<UnionArrayAccessorMut<'borrow, 'scope, 'data>>
    where
        F: Frame<'frame>,
    {
        self.ensure_union()?;

        let accessor = unsafe { ArrayAccessor::new2(self) };
        Ledger::try_borrow_mut(frame.ledger(), accessor)
    }
    
    /// Muutably access the contents of this array. 
    /// 
    /// The element type must be a bits union.
    /// 
    /// Errors:
    ///  - `ArrayLayoutError::NotUnion` if the data isn't a bits union.
    /// 
    /// Safety: This method doesn't track the borrow, mutable aliasing is not prevented. 
    pub unsafe fn union_data_mut_untracked<'borrow>(
        &'borrow mut self,
    ) -> JlrsResult<UnionArrayAccessorMut<'borrow, 'scope, 'data>>
    {
        self.ensure_union()?;

        let accessor = ArrayAccessor::new2(self);
        Ok(UnionArrayAccessorMut::new(None, accessor))
    }
    
    /// Immutably access the contents of this array. 
    /// 
    /// Errors:
    ///  - `AccessError::BorrowError` if the array is already mutably borrowed.
    pub fn indeterminate_data<'borrow, 'frame: 'borrow, F>(
        &'borrow self,
        frame: &F,
    ) -> JlrsResult<IndeterminateArrayAccessor<'borrow, 'scope, 'data>>
    where
        F: Frame<'frame>,
    {
        // Safety: layouts are compatible, access is immutable.
        let accessor = unsafe { ArrayAccessor::new2(self) };
        Ledger::try_borrow(frame.ledger(), accessor)
    }
    
    /// Immutably access the contents of this array. 
    /// 
    /// Safety: This method doesn't track the borrow, mutable aliasing is not prevented. 
    pub unsafe fn indeterminate_data_untracked<'borrow>(
        &'borrow self,
    ) -> IndeterminateArrayAccessor<'borrow, 'scope, 'data>
    {
        // Safety: layouts are compatible, access is immutable.
        let accessor = ArrayAccessor::new2(self);
        IndeterminateArrayAccessor::new(None, accessor)
    }
    
    /// Mutably access the contents of this array. 
    /// 
    /// Errors:
    ///  - `AccessError::BorrowError` if the array is already mutably borrowed.
    pub fn indeterminate_data_mut<'borrow, 'frame: 'borrow, F>(
        &'borrow mut self,
        frame: &F,
    ) -> JlrsResult<IndeterminateArrayAccessorMut<'borrow, 'scope, 'data>>
    where
        F: Frame<'frame>,
    {
        let accessor = unsafe { ArrayAccessor::new2(self) };
        Ledger::try_borrow_mut(frame.ledger(), accessor)
    }
    
    /// Mutably access the contents of this array. 
    /// 
    /// Safety: This method doesn't track the borrow, mutable aliasing is not prevented. 
    pub unsafe fn indeterminate_data_mut_untracked<'borrow>(
        &'borrow mut self,
    ) -> IndeterminateArrayAccessorMut<'borrow, 'scope, 'data>
    {
        let accessor = ArrayAccessor::new2(self);
        IndeterminateArrayAccessorMut::new(None, accessor)
    }
}