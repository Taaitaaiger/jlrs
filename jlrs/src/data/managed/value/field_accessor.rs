use std::{mem::MaybeUninit, sync::atomic::Ordering, usize};
#[julia_version(since = "1.7")]
use std::{
    ptr::null_mut,
    ptr::NonNull,
    sync::atomic::{AtomicPtr, AtomicU16, AtomicU32, AtomicU64, AtomicU8},
};

use jl_sys::jlrs_array_typetagdata;
#[julia_version(since = "1.7")]
use jl_sys::{jl_value_t, jlrs_lock_value, jlrs_unlock_value};
use jlrs_macros::julia_version;

use super::{Value, ValueRef};
use crate::{
    data::{
        layout::valid_layout::ValidLayout,
        managed::{
            array::Array,
            datatype::{DataType, DataTypeRef},
            private::ManagedPriv,
            union::{nth_union_component, Union},
            Managed,
        },
    },
    error::{AccessError, JlrsResult, CANNOT_DISPLAY_TYPE},
    private::Private,
};

#[julia_version(since = "1.7")]
#[repr(C, align(16))]
#[derive(Clone, Copy)]
union AtomicBuffer {
    bytes: [MaybeUninit<u8>; 16],
    ptr: *mut jl_value_t,
}

#[julia_version(since = "1.7")]
impl AtomicBuffer {
    fn new() -> Self {
        AtomicBuffer { ptr: null_mut() }
    }
}

#[derive(Copy, Clone, PartialEq)]
enum ViewState {
    #[cfg(not(feature = "julia-1-6"))]
    Locked,
    Unlocked,
    #[cfg(not(feature = "julia-1-6"))]
    AtomicBuffer,
    Array,
}

// TODO: track

/// Access the raw contents of a Julia value.
///
/// A `FieldAccessor` for a value can be created with [`Value::field_accessor`]. By chaining calls
/// to the `field` and `atomic_field` methods you can access deeply nested fields without
/// allocating temporary Julia data. These two methods support three kinds of field identifiers:
/// field names, numerical field indices, and n-dimensional array indices. The first two can be
/// used with types that have named fields, the second must be used with tuples, and the last one
/// with arrays.
pub struct FieldAccessor<'scope, 'data> {
    value: Option<ValueRef<'scope, 'data>>,
    current_field_type: Option<DataTypeRef<'scope>>,
    #[cfg(not(feature = "julia-1-6"))]
    buffer: AtomicBuffer,
    offset: u32,
    state: ViewState,
}

impl<'scope, 'data> FieldAccessor<'scope, 'data> {
    #[inline]
    pub(crate) fn new(value: Value<'scope, 'data>) -> Self {
        FieldAccessor {
            value: Some(value.as_ref()),
            current_field_type: Some(value.datatype().as_ref()),
            offset: 0,
            #[cfg(not(feature = "julia-1-6"))]
            buffer: AtomicBuffer::new(),
            state: ViewState::Unlocked,
        }
    }
    /// Access the field the accessor is currenty pointing to as a value of type `T`.
    ///
    /// This method accesses the field using its concrete type. If the concrete type of the field
    /// has a matching managed type it can be accessed as a `ValueRef` or a `Ref` of that managed
    /// type. For example, a field that contains a `Module` can be accessed as a `ModuleRef`. In
    /// all other cases a layout type must be used. For example, an untyped field that currently
    /// holds a `Float64` must be accessed as `f64`.
    pub fn access<T: ValidLayout>(self) -> JlrsResult<T> {
        if self.current_field_type.is_none() {
            Err(AccessError::UndefRef)?;
        }

        if self.value.is_none() {
            Err(AccessError::UndefRef)?;
        }

        // Safety: in this block, the first check ensures that T is correct
        // for the data that is accessed. If the data is in the atomic buffer
        // it's read from there. If T is as Ref, the pointer is converted. If
        // it's an array, the element at the desired position is read.
        // Otherwise, the field is read at the offset where it has been determined
        // to be stored.
        unsafe {
            let ty = self.current_field_type.unwrap().as_value();
            if !T::valid_layout(ty) {
                let value_type = ty.display_string_or(CANNOT_DISPLAY_TYPE).into();
                Err(AccessError::InvalidLayout { value_type })?;
            }

            #[cfg(not(feature = "julia-1-6"))]
            if self.state == ViewState::AtomicBuffer {
                debug_assert!(!T::IS_REF);
                debug_assert!(std::mem::size_of::<T>() <= 8);
                return Ok(std::ptr::read(
                    self.buffer.bytes[self.offset as usize..].as_ptr() as *const T,
                ));
            }

            if T::IS_REF {
                Ok(std::mem::transmute_copy(&self.value))
            } else if self.state == ViewState::Array {
                Ok(self
                    .value
                    .unwrap()
                    .as_value()
                    .cast_unchecked::<Array>()
                    .data_ptr()
                    .cast::<u8>()
                    .add(self.offset as usize)
                    .cast::<T>()
                    .read())
            } else {
                Ok(self
                    .value
                    .unwrap()
                    .ptr()
                    .cast::<u8>()
                    .as_ptr()
                    .add(self.offset as usize)
                    .cast::<T>()
                    .read())
            }
        }
    }

    /// Returns `true` if `self.access::<T>()` will succeed, `false` if it will fail.
    #[inline]
    pub fn can_access_as<T: ValidLayout>(&self) -> bool {
        if self.current_field_type.is_none() {
            return false;
        }

        // Safety: the current_field_type field is not undefined.
        let ty = unsafe { self.current_field_type.unwrap().as_value() };
        if !T::valid_layout(ty) {
            return false;
        }

        true
    }

    /// Update the accessor to point to `field`.
    ///
    /// Three kinds of field indices exist: field names, numerical field indices, and
    /// n-dimensional array indices. The first two can be used with types that have named fields,
    /// the second must be used with tuples, and the last one with arrays.
    ///
    /// If `field` is an invalid identifier an error is returned. Calls to `field` can be chained
    /// to access nested fields.
    ///
    /// If the field is an atomic field the same ordering is used as Julia uses by default:
    /// `Relaxed` for pointer fields, `SeqCst` for small inline fields, and a lock for large
    /// inline fields.
    pub fn field<F: FieldIndex>(mut self, field: F) -> JlrsResult<Self> {
        if self.value.is_none() {
            Err(AccessError::UndefRef)?
        }

        if self.current_field_type.is_none() {
            Err(AccessError::UndefRef)?
        }

        // Safety: how to access the next field depends on the current view. If an array
        // is accessed the view is updated to the requested element. Otherwise, the offset
        // is adjusted to target the requested field. Because the starting point is assumed
        // to be rooted, all pointer fields are either reachablle or undefined. If a field is
        // atomic, atomic accesses (or locks for large atomic fields) are used.
        unsafe {
            let current_field_type = self.current_field_type.unwrap().as_managed();
            if self.state == ViewState::Array && current_field_type.is::<Array>() {
                let arr = self.value.unwrap().as_value().cast_unchecked::<Array>();
                // accessing an array, find the offset of the requested element
                let index = field.array_index(arr, Private)?;
                self.get_array_field(arr, index);
                return Ok(self);
            }

            let index = field.field_index(current_field_type, Private)?;

            let next_field_type = match current_field_type.field_type(index) {
                Some(ty) => ty,
                _ => Err(AccessError::UndefRef)?,
            };

            let is_pointer_field = current_field_type.is_pointer_field_unchecked(index);
            let field_offset = current_field_type.field_offset_unchecked(index);
            self.offset += field_offset;

            match self.state {
                ViewState::Array => {
                    self.get_inline_array_field(is_pointer_field, next_field_type)?
                }
                ViewState::Unlocked => self.get_unlocked_inline_field(
                    is_pointer_field,
                    current_field_type,
                    next_field_type,
                    index,
                    Ordering::Relaxed,
                    Ordering::SeqCst,
                ),
                #[cfg(not(feature = "julia-1-6"))]
                ViewState::Locked => {
                    self.get_locked_inline_field(is_pointer_field, next_field_type)
                }
                #[cfg(not(feature = "julia-1-6"))]
                ViewState::AtomicBuffer => {
                    self.get_atomic_buffer_field(is_pointer_field, next_field_type)
                }
            }
        }

        Ok(self)
    }

    #[julia_version(since = "1.7")]
    /// Update the accessor to point to `field`.
    ///
    /// If the field is a small atomic field `ordering` is used to read it. The ordering is
    /// ignored for non-atomic fields and fields that require a lock to access. See
    /// [`FieldAccessor::field`] for more information.
    pub fn atomic_field<F: FieldIndex>(mut self, field: F, ordering: Ordering) -> JlrsResult<Self> {
        if self.value.is_none() {
            Err(AccessError::UndefRef)?
        }

        if self.current_field_type.is_none() {
            Err(AccessError::UndefRef)?
        }

        // Safety: how to access the next field depends on the current view. If an array
        // is accessed the view is updated to the requested element. Otherwise, the offset
        // is adjusted to target the requested field. Because the starting point is assumed
        // to be rooted, all pointer fields are either reachablle or undefined. If a field is
        // atomic, atomic accesses (or locks for large atomic fields) are used.
        unsafe {
            let current_field_type = self.current_field_type.unwrap().as_managed();
            if self.state == ViewState::Array && current_field_type.is::<Array>() {
                let arr = self.value.unwrap().as_value().cast_unchecked::<Array>();
                // accessing an array, find the offset of the requested element
                let index = field.array_index(arr, Private)?;
                self.get_array_field(arr, index);
                return Ok(self);
            }

            let index = field.field_index(current_field_type, Private)?;

            let next_field_type = match current_field_type.field_type(index) {
                Some(ty) => ty,
                _ => Err(AccessError::UndefRef)?,
            };

            let is_pointer_field = current_field_type.is_pointer_field_unchecked(index);
            let field_offset = current_field_type.field_offset_unchecked(index);
            self.offset += field_offset;

            match self.state {
                ViewState::Array => {
                    self.get_inline_array_field(is_pointer_field, next_field_type)?
                }
                ViewState::Unlocked => self.get_unlocked_inline_field(
                    is_pointer_field,
                    current_field_type,
                    next_field_type,
                    index,
                    ordering,
                    ordering,
                ),
                ViewState::Locked => {
                    self.get_locked_inline_field(is_pointer_field, next_field_type)
                }
                ViewState::AtomicBuffer => {
                    self.get_atomic_buffer_field(is_pointer_field, next_field_type)
                }
            }
        }

        Ok(self)
    }

    /// Try to clone this accessor and its state.
    ///
    /// If the current value this accessor is accessing is locked an error is returned.
    #[inline]
    pub fn try_clone(&self) -> JlrsResult<Self> {
        #[cfg(not(feature = "julia-1-6"))]
        if self.state == ViewState::Locked {
            Err(AccessError::Locked)?;
        }

        Ok(FieldAccessor {
            value: self.value,
            current_field_type: self.current_field_type,
            offset: self.offset,
            #[cfg(not(feature = "julia-1-6"))]
            buffer: self.buffer.clone(),
            state: self.state,
        })
    }

    #[julia_version(since = "1.7")]
    #[inline]
    /// Returns `true` if the current value the accessor is accessing is locked.
    pub fn is_locked(&self) -> bool {
        self.state == ViewState::Locked
    }

    #[julia_version(until = "1.6")]
    #[inline]
    /// Returns `true` if the current value the accessor is accessing is locked.
    pub fn is_locked(&self) -> bool {
        false
    }

    /// Returns the type of the field the accessor is currently pointing at.
    #[inline]
    pub fn current_field_type(&self) -> Option<DataTypeRef<'scope>> {
        self.current_field_type
    }

    /// Returns the value the accessor is currently inspecting.
    #[inline]
    pub fn value(&self) -> Option<ValueRef<'scope, 'data>> {
        self.value
    }

    #[julia_version(since = "1.7")]
    // Safety: the view state must be ViewState::AtomicBuffer
    unsafe fn get_atomic_buffer_field(
        &mut self,
        is_pointer_field: bool,
        next_field_type: Value<'scope, 'data>,
    ) {
        if is_pointer_field {
            debug_assert_eq!(self.offset, 0);
            let ptr = self.buffer.ptr;
            if ptr.is_null() {
                self.value = None;
            } else {
                self.value = Some(ValueRef::wrap(NonNull::new_unchecked(ptr)));
            }

            self.state = ViewState::Unlocked;
            if self.value.is_none() {
                if let Ok(ty) = next_field_type.cast::<DataType>() {
                    if ty.is_concrete_type() {
                        self.current_field_type = Some(ty.as_ref());
                    } else {
                        self.current_field_type = None;
                    }
                } else {
                    self.current_field_type = None;
                }
            } else {
                self.current_field_type =
                    Some(self.value.unwrap().as_managed().datatype().as_ref());
            }
        } else {
            debug_assert!(next_field_type.is::<DataType>());
            self.current_field_type = Some(next_field_type.cast_unchecked::<DataType>().as_ref());
        }
    }

    #[julia_version(since = "1.7")]
    // Safety: the view state must be ViewState::Unlocked
    unsafe fn get_unlocked_inline_field(
        &mut self,
        is_pointer_field: bool,
        current_field_type: DataType<'scope>,
        next_field_type: Value<'scope, 'data>,
        index: usize,
        pointer_ordering: Ordering,
        inline_ordering: Ordering,
    ) {
        let is_atomic_field = current_field_type.is_atomic_field_unchecked(index);
        if is_pointer_field {
            if is_atomic_field {
                self.get_atomic_pointer_field(next_field_type, pointer_ordering);
            } else {
                self.get_pointer_field(false, next_field_type);
            }
        } else if let Ok(un) = next_field_type.cast::<Union>() {
            self.get_bits_union_field(un);
        } else {
            debug_assert!(next_field_type.is::<DataType>());
            self.current_field_type = Some(next_field_type.cast_unchecked::<DataType>().as_ref());

            if is_atomic_field {
                self.lock_or_copy_atomic(inline_ordering);
            }
        }
    }

    #[julia_version(until = "1.6")]
    // Safety: the view state must be ViewState::Unlocked
    unsafe fn get_unlocked_inline_field(
        &mut self,
        is_pointer_field: bool,
        _current_field_type: DataType<'scope>,
        next_field_type: Value<'scope, 'data>,
        _index: usize,
        _pointer_ordering: Ordering,
        _inline_ordering: Ordering,
    ) {
        if is_pointer_field {
            self.get_pointer_field(false, next_field_type);
        } else if let Ok(un) = next_field_type.cast::<Union>() {
            self.get_bits_union_field(un);
        } else {
            debug_assert!(next_field_type.is::<DataType>());
            self.current_field_type = Some(next_field_type.cast_unchecked::<DataType>().as_ref());
        }
    }

    #[julia_version(since = "1.7")]
    // Safety: the view state must be ViewState::Locked
    unsafe fn get_locked_inline_field(
        &mut self,
        is_pointer_field: bool,
        next_field_type: Value<'scope, 'data>,
    ) {
        if is_pointer_field {
            self.get_pointer_field(true, next_field_type);
        } else if let Ok(un) = next_field_type.cast::<Union>() {
            self.get_bits_union_field(un);
        } else {
            debug_assert!(next_field_type.is::<DataType>());
            self.current_field_type = Some(next_field_type.cast_unchecked::<DataType>().as_ref());
        }
    }

    // Safety: the view state must be ViewState::Array
    unsafe fn get_inline_array_field(
        &mut self,
        is_pointer_field: bool,
        next_field_type: Value<'scope, 'data>,
    ) -> JlrsResult<()> {
        // Inline field of the current array
        if is_pointer_field {
            self.value = self
                .value
                .unwrap()
                .as_value()
                .cast::<Array>()?
                .data_ptr()
                .cast::<MaybeUninit<u8>>()
                .add(self.offset as usize)
                .cast::<Option<ValueRef>>()
                .read();

            self.offset = 0;
            self.state = ViewState::Unlocked;

            if self.value.is_none() {
                if let Ok(ty) = next_field_type.cast::<DataType>() {
                    if ty.is_concrete_type() {
                        self.current_field_type = Some(ty.as_ref());
                    } else {
                        self.current_field_type = None;
                    }
                } else {
                    self.current_field_type = None;
                }
            } else {
                self.current_field_type = Some(self.value.unwrap().as_value().datatype().as_ref());
            }
        } else {
            self.current_field_type = Some(next_field_type.cast::<DataType>()?.as_ref());
        }

        Ok(())
    }

    #[julia_version(since = "1.7")]
    // Safety: must only be used to read an atomic field
    unsafe fn lock_or_copy_atomic(&mut self, ordering: Ordering) {
        let ptr = self
            .value
            .unwrap()
            .ptr()
            .cast::<MaybeUninit<u8>>()
            .as_ptr()
            .add(self.offset as usize);

        match self
            .current_field_type
            .unwrap()
            .as_managed()
            .size()
            .unwrap_or(std::mem::size_of::<usize>() as _)
        {
            0 => (),
            1 => {
                let atomic = &*ptr.cast::<AtomicU8>();
                let v = atomic.load(ordering);
                let dst_ptr = self.buffer.bytes.as_mut_ptr();
                std::ptr::copy_nonoverlapping(&v as *const _ as *const u8, dst_ptr as _, 1);
                self.state = ViewState::AtomicBuffer;
                self.offset = 0;
            }
            2 => {
                let atomic = &*ptr.cast::<AtomicU16>();
                let v = atomic.load(ordering);
                let dst_ptr = self.buffer.bytes.as_mut_ptr();
                std::ptr::copy_nonoverlapping(&v as *const _ as *const u8, dst_ptr as _, 2);
                self.state = ViewState::AtomicBuffer;
                self.offset = 0;
            }
            sz if sz <= 4 => {
                let atomic = &*ptr.cast::<AtomicU32>();
                let v = atomic.load(ordering);
                let dst_ptr = self.buffer.bytes.as_mut_ptr();
                std::ptr::copy_nonoverlapping(
                    &v as *const _ as *const u8,
                    dst_ptr as _,
                    sz as usize,
                );
                self.state = ViewState::AtomicBuffer;
                self.offset = 0;
            }
            sz if sz <= 8 => {
                let atomic = &*ptr.cast::<AtomicU64>();
                let v = atomic.load(ordering);
                let dst_ptr = self.buffer.bytes.as_mut_ptr();
                std::ptr::copy_nonoverlapping(
                    &v as *const _ as *const u8,
                    dst_ptr as _,
                    sz as usize,
                );
                self.state = ViewState::AtomicBuffer;
                self.offset = 0;
            }
            #[cfg(not(any(
                feature = "julia-1-6",
                feature = "julia-1-7",
                feature = "julia-1-8",
                feature = "julia-1-9",
                feature = "julia-1-10",
                feature = "julia-1-11"
            )))]
            sz if sz <= 16 => {
                let atomic = &*ptr.cast::<atomic::Atomic<u128>>();
                let v = atomic.load(ordering);
                let dst_ptr = self.buffer.bytes.as_mut_ptr();
                std::ptr::copy_nonoverlapping(
                    &v as *const _ as *const u8,
                    dst_ptr as _,
                    sz as usize,
                );
                self.state = ViewState::AtomicBuffer;
                self.offset = 0;
            }
            _ => {
                jlrs_lock_value(self.value.unwrap().ptr().as_ptr());
                self.state = ViewState::Locked;
            }
        }
    }

    // Safety: must only be used to read an array element
    unsafe fn get_array_field(&mut self, arr: Array<'scope, 'data>, index: usize) {
        debug_assert!(self.state == ViewState::Array);
        let el_size = arr.element_size() as usize;
        self.offset = (index * el_size) as u32;

        if arr.has_value_layout() {
            self.value = arr.data_ptr().cast::<Option<ValueRef>>().add(index).read();
            self.offset = 0;
            if self.value.is_none() {
                if let Ok(ty) = arr.element_type().cast::<DataType>() {
                    if ty.is_concrete_type() {
                        self.current_field_type = Some(ty.as_ref());
                    } else {
                        self.current_field_type = None;
                    }

                    if !ty.is::<Array>() {
                        self.state = ViewState::Unlocked;
                    }
                } else {
                    self.current_field_type = None;
                    self.state = ViewState::Unlocked;
                }
            } else {
                let ty = self.value.unwrap().as_value().datatype();
                self.current_field_type = Some(ty.as_ref());
                if !ty.is::<Array>() {
                    self.state = ViewState::Unlocked;
                }
            }
        } else if arr.has_union_layout() {
            let mut tag = *jlrs_array_typetagdata(arr.unwrap(Private)).add(index) as i32;
            let component = nth_union_component(arr.element_type(), &mut tag);
            debug_assert!(component.is_some());
            let ty = component.unwrap_unchecked();
            debug_assert!(ty.is::<DataType>());
            let ty = ty.cast_unchecked::<DataType>();
            debug_assert!(ty.is_concrete_type());
            self.current_field_type = Some(ty.as_ref());
        } else {
            let ty = arr.element_type();
            debug_assert!(ty.is::<DataType>());
            self.current_field_type = Some(ty.cast_unchecked::<DataType>().as_ref());
        }
    }

    #[julia_version(since = "1.7")]
    // Safety: must only be used to read an pointer field
    unsafe fn get_pointer_field(&mut self, locked: bool, next_field_type: Value<'scope, 'data>) {
        let value = self
            .value
            .unwrap()
            .ptr()
            .cast::<u8>()
            .as_ptr()
            .add(self.offset as usize)
            .cast::<Option<ValueRef>>()
            .read();

        if locked {
            jlrs_unlock_value(self.value.unwrap().ptr().as_ptr());
            self.state = ViewState::Unlocked;
        }

        self.value = value;
        self.offset = 0;

        if self.value.is_none() {
            if let Ok(ty) = next_field_type.cast::<DataType>() {
                if ty.is_concrete_type() {
                    self.current_field_type = Some(ty.as_ref());
                } else {
                    self.current_field_type = None;
                }
            } else {
                self.current_field_type = None;
            }
        } else {
            let value = self.value.unwrap().as_value();
            self.current_field_type = Some(value.datatype().as_ref());
            if value.is::<Array>() {
                self.state = ViewState::Array;
            }
        }
    }

    #[julia_version(until = "1.6")]
    // Safety: must only be used to read an pointer field
    unsafe fn get_pointer_field(&mut self, _locked: bool, next_field_type: Value<'scope, 'data>) {
        self.value = self
            .value
            .unwrap()
            .ptr()
            .cast::<u8>()
            .as_ptr()
            .add(self.offset as usize)
            .cast::<Option<ValueRef>>()
            .read();

        self.offset = 0;

        if self.value.is_none() {
            if let Ok(ty) = next_field_type.cast::<DataType>() {
                if ty.is_concrete_type() {
                    self.current_field_type = Some(ty.as_ref());
                } else {
                    self.current_field_type = None;
                }
            } else {
                self.current_field_type = None;
            }
        } else {
            let value = self.value.unwrap().as_value();
            self.current_field_type = Some(value.datatype().as_ref());
            if value.is::<Array>() {
                self.state = ViewState::Array;
            }
        }
    }

    #[julia_version(since = "1.7")]
    // Safety: must only be used to read an atomic pointer field
    unsafe fn get_atomic_pointer_field(
        &mut self,
        next_field_type: Value<'scope, 'data>,
        ordering: Ordering,
    ) {
        let v = &*self
            .value
            .unwrap()
            .ptr()
            .cast::<u8>()
            .as_ptr()
            .add(self.offset as usize)
            .cast::<AtomicPtr<jl_value_t>>();

        let ptr = v.load(ordering);
        if ptr.is_null() {
            self.value = None;
        } else {
            self.value = Some(ValueRef::wrap(NonNull::new_unchecked(ptr)));
        }

        self.offset = 0;

        if self.value.is_none() {
            if let Ok(ty) = next_field_type.cast::<DataType>() {
                if ty.is_concrete_type() {
                    self.current_field_type = Some(ty.as_ref());
                } else {
                    self.current_field_type = None;
                }
            } else {
                self.current_field_type = None;
            }
        } else {
            let value = self.value.unwrap().as_value();
            self.current_field_type = Some(value.datatype().as_ref());
            if value.is::<Array>() {
                self.state = ViewState::Array;
            }
        }
    }

    // Safety: must only be used to read a bits union field
    unsafe fn get_bits_union_field(&mut self, union: Union<'scope>) {
        let mut size = 0;
        let isbits = union.isbits_size_align(&mut size, &mut 0);
        debug_assert!(isbits);
        let flag_offset = self.offset as usize + size;
        let mut flag = self
            .value
            .unwrap()
            .ptr()
            .cast::<u8>()
            .as_ptr()
            .add(flag_offset)
            .read() as i32;

        let active_ty = nth_union_component(union.as_value(), &mut flag);
        debug_assert!(active_ty.is_some());
        let active_ty = active_ty.unwrap_unchecked();
        debug_assert!(active_ty.is::<DataType>());

        let ty = active_ty.cast_unchecked::<DataType>();
        debug_assert!(ty.is_concrete_type());
        self.current_field_type = Some(ty.as_ref());
    }
}

impl Drop for FieldAccessor<'_, '_> {
    fn drop(&mut self) {
        #[cfg(not(feature = "julia-1-6"))]
        if self.state == ViewState::Locked {
            debug_assert!(!self.value.is_none());
            // Safety: the value is currently locked.
            unsafe { jlrs_unlock_value(self.value.unwrap().ptr().as_ptr()) }
        }
    }
}
/// Trait implemented by types that can be used in combination with a
/// [`FieldAccessor`] as the index for a field.
///
/// [`FieldAccessor`]: crate::data::managed::value::field_accessor::FieldAccessor
pub trait FieldIndex: private::FieldIndexPriv {}
impl<I: private::FieldIndexPriv> FieldIndex for I {}

mod private {
    use crate::{
        convert::to_symbol::private::ToSymbolPriv,
        data::managed::{
            array::{dimensions::Dims, Array},
            datatype::DataType,
            string::JuliaString,
            symbol::Symbol,
            Managed,
        },
        error::{AccessError, JlrsResult, CANNOT_DISPLAY_TYPE, CANNOT_DISPLAY_VALUE},
        private::Private,
    };

    pub trait FieldIndexPriv: std::fmt::Debug {
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
            let sym = unsafe { self.to_symbol_priv(Private) };

            let Some(idx) = ty.field_index(sym) else {
                Err(AccessError::NoSuchField {
                    type_name: ty.display_string_or(CANNOT_DISPLAY_TYPE),
                    field_name: self.to_string(),
                })?
            };

            Ok(idx as usize)
        }
    }

    impl FieldIndexPriv for Symbol<'_> {
        #[inline]
        fn field_index(&self, ty: DataType, _: Private) -> JlrsResult<usize> {
            let Some(idx) = ty.field_index(*self) else {
                Err(AccessError::NoSuchField {
                    type_name: ty.display_string_or(CANNOT_DISPLAY_TYPE),
                    field_name: self.display_string_or(CANNOT_DISPLAY_VALUE),
                })?
            };

            Ok(idx as usize)
        }
    }

    impl FieldIndexPriv for JuliaString<'_> {
        #[inline]
        fn field_index(&self, ty: DataType, _: Private) -> JlrsResult<usize> {
            let sym = unsafe { self.to_symbol_priv(Private) };

            let Some(idx) = ty.field_index(sym) else {
                Err(AccessError::NoSuchField {
                    type_name: ty.display_string_or(CANNOT_DISPLAY_TYPE),
                    field_name: self.as_str().unwrap_or(CANNOT_DISPLAY_VALUE).to_string(),
                })?
            };

            Ok(idx as usize)
        }
    }

    impl<D: Dims> FieldIndexPriv for D {
        #[inline]
        fn field_index(&self, ty: DataType, _: Private) -> JlrsResult<usize> {
            debug_assert!(!ty.is::<Array>());

            if self.rank() != 1 {
                Err(AccessError::FieldNeedsSimpleIndex)?
            }

            let n = self.size();
            let n_fields = ty.n_fields().ok_or_else(|| AccessError::NoFields {
                value_type: ty.display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
            if n_fields as usize <= n {
                Err(AccessError::OutOfBoundsField {
                    idx: n,
                    n_fields: n_fields as usize,
                    value_type: ty.display_string_or(CANNOT_DISPLAY_TYPE),
                })?;
            }

            Ok(n)
        }

        #[inline]
        fn array_index(&self, data: Array, _: Private) -> JlrsResult<usize> {
            let res = data
                .dimensions()
                .index_of(self)
                .ok_or(AccessError::InvalidIndex {
                    idx: self.to_dimensions(),
                    sz: data.dimensions().to_dimensions(),
                })?;

            Ok(res)
        }
    }
}
