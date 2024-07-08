//! N-dimensional arrays
//!
//! Julia has a generic array type, `Array{T, N}`. These arrays are column-major, N-dimensional
//! arrays that can hold elements of type `T`.
//!
//! jlrs provides a flexible base type that wraps instances of this type, [`ArrayBase`]. This type
//! has two generics: a [type constructor] `T`, and a constant `isize` rank `N`. You shouldn't use
//! this type directly, but use the four available type aliases instead: [`Array`],
//! [`TypedArray`], [`RankedArray`], and [`TypedRankedArray`]. If `Typed` is missing from the
//! name, the element type `T` is set to [`Unknown`], if `Ranked` is missing the the rank `N` is
//! set to `-1`.
//!
//! There are several special aliases: [`Vector`], [`VectorAny`] and [`TypedVector`] (rank 1), and
//! [`Matrix`] and [`TypedMatrix`] (rank 2).
//!
//! ## Converting between array types
//!
//! The methods [`ArrayBase::set_rank`] and [`ArrayBase::set_type`] can be used to set the two
//! generic parameters, [`ArrayBase::forget_rank`] and [`ArrayBase::forget_type`] lets you set
//! them to `-1` and `Unknown` respectively.
//!
//! ## Constructing new arrays
//!
//! Many methods that construct new arrays exist, they can be divided into several groups:
//!
//! - `new`: Constructs a new array whose storage is managed by Julia.
//!
//! - `from_slice`: Constructs a new array whose storage is borrowed from Rust.
//!
//! - `from_vec`: Constructs a new array whose storage is moved from Rust.
//!
//! - `from_slice_cloned`: Constructs a new array whose storage is managed by Julia, the elements
//!   are initialized by cloning a slice.
//!
//! - `from_slice_copied`: Constructs a new array whose storage is managed by Julia, the elements
//!   are initialized by copying a slice.
//!
//! These methods exist as named for typed arrays, the element type is constructed from the
//! provided type parameter. Untyped arrays have `*_for` methods like `new_for` that take the
//! element type as an argument. All these methods have unsafe, unchecked variants like
//! `new_for_unchecked`.
//!
//! One limitation of arrays that are backed by Rust data is that Julia is not able to reallocate
//! this array. Functions that can reallocate, like `push!`, will throw an exception if they are
//! called with such an array.
//!
//! In addition to these generic constructors there are two specialized constructors:
//! [`TypedVector<u8>`] can be constructed with `from_bytes`, which behaves as `from_slice_copied`
//! does. [`VectorAny`] can be constructed with `new_any`, which behaves as `new` does.
//!
//! ## Array data
//!
//! In order to access the content of an array an accessor must be created first. There are
//! several kinds of accessors to account for the different ways the elements can be laid out in
//! memory.
//!
//! Elements are either stored inline in the backing storage or as references. They are stored
//! inline if they are immutable, concrete types. Unions of `isbits` types, i.e. immutable,
//! concrete types which contain no references to other Julia data, are also stored inline. In
//! this last case a type tag is stored for each element after the elements themselves. In all
//!  other cases, the elements are stored as references, i.e. as `Option<ValueRef>`.
//!
//! For several reasons, some more technical than others, it's useful to distinguish between
//! `isbits` and "non-bits" immutable types. Similarly, for elements that are stored as references
//! it can be useful to distinguish between arbitrary `Value`s and more specific managed types.
//! It's also perfectly valid to not make any assumptions about the layout and only work with
//! `Value`s, allocating new Julia data whenever necessary.
//!
//! Putting all of this together, we end up with the following accessors: [`BitsAccessor`],
//! [`InlineAccessor`], [`BitsUnionAccessor`], [`ValueAccessor`], [`ManagedAccessor`], and
//! [`IndeterminateAccessor`]. There are also mutable variants of all of these accessors.
//!
//! Depending on the element type parameter `T` and the traits it implements, it can be possible
//! to infer that a certain accessor must be used. If this is the case, a method to create
//! that accessor without performing any checks will be available. An example is
//! [`ArrayBase::bits_data`], which is only available if `T: IsBits + ConstructType`. If this
//! information can't be inferred from `T`, `try_*` and `*_unchecked` variants are available.
//!
//! ## Tracking
//!
//! It's very easy to accidentally create multiple mutable accessors to the same array. In order
//! to prevent this, you can track an array. You can either track an array exclusively or allow
//! multiple shared references with [`ArrayBase::track_exclusive`] and
//! [`ArrayBase::track_shared`] respectively. This dynamically enforces borrowing rules at
//! runtime, but is limited. While it is thread-safe and even works across multiple packages, the
//! tracking mechanism is unaware of how the data is used inside Julia. It won't protect you from
//! accessing an array that is currently being mutated by some Julia task running in the
//! background. Tracking is also relatively expensive; if you can guarantee you are the only user
//! of an array, e.g. you've just allocated it, you should avoid tracking the array.
//!
//! [type constructor]: crate::data::types::construct_type::ConstructType
//! [`Array::new_for`]: crate::data::managed::array::ArrayBase::new_for
//! [`Array::from_slice_for`]: crate::data::managed::array::ArrayBase::from_slice_for
//! [`Array::from_slice_cloned_for`]: crate::data::managed::array::ArrayBase::from_slice_cloned_for
//! [`Array::from_vec_for`]: crate::data::managed::array::ArrayBase::from_vec_for
//! [`TypedArray::new`]: crate::data::managed::array::ArrayBase::new
//! [`TypedArray:from_slice`]: crate::data::managed::array::ArrayBase::from_slice
//! [`TypedArray:from_slice_cloned`]: crate::data::managed::array::ArrayBase::from_slice_cloned
//! [`TypedArray:from_vec`]: crate::data::managed::array::ArrayBase::from_vec
//! [`RankedArray::new_for`]: crate::data::managed::array::ArrayBase::new_for
//! [`RankedArray::from_slice_for`]: crate::data::managed::array::ArrayBase::from_slice_for
//! [`RankedArray::from_slice_cloned_for`]: crate::data::managed::array::ArrayBase::from_slice_cloned_for
//! [`RankedArray::from_vec_for`]: crate::data::managed::array::ArrayBase::from_vec_for
//! [`TypedArrayRanked::new`]: crate::data::managed::array::ArrayBase::new
//! [`TypedArrayRanked::from_slice`]: crate::data::managed::array::ArrayBase::from_slice
//! [`TypedArrayRanked::from_slice_cloned`]: crate::data::managed::array::ArrayBase::from_slice_cloned
//! [`TypedArrayRanked::from_vec`]: crate::data::managed::array::ArrayBase::from_vec
//! [`TypedVector::from_bytes`]: crate::data::managed::array::ArrayBase::from_bytes
//! [isbits]: crate::data::layout::is_bits
//! [managed type]: crate::data::managed

pub mod data;
pub mod dimensions;
pub mod tracked;

#[julia_version(since = "1.11")]
use std::ptr::null_mut;
use std::{
    ffi::c_void,
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
    mem::MaybeUninit,
    ptr::NonNull,
};

use jl_sys::{
    inlined::{jlrs_array_dims_ptr, jlrs_array_ndims_fast},
    jl_alloc_vec_any, jl_apply_array_type, jl_array_eltype, jl_array_rank, jl_array_t,
    jl_array_to_string, jl_gc_add_ptr_finalizer, jl_new_struct_uninit, jl_pchar_to_array,
    jlrs_array_data, jlrs_array_data_owner, jlrs_array_has_pointers, jlrs_array_how,
    jlrs_array_is_pointer_array, jlrs_array_is_union_array, jlrs_array_len,
};
use jlrs_macros::julia_version;

#[julia_version(until = "1.10")]
use self::dimensions::Dims;
use self::{
    data::accessor::{
        BitsAccessor, BitsAccessorMut, BitsUnionAccessor, BitsUnionAccessorMut,
        IndeterminateAccessor, IndeterminateAccessorMut, InlineAccessor, InlineAccessorMut,
        ManagedAccessor, ManagedAccessorMut, ValueAccessor, ValueAccessorMut,
    },
    dimensions::{ArrayDimensions, DimsExt, DimsRankAssert, DimsRankCheck, RankedDims},
    tracked::{TrackedArrayBase, TrackedArrayBaseMut},
};
use super::{
    string::{JuliaString, StringData},
    symbol::static_symbol::{NSym, StaticSymbol, TSym},
    union::Union,
};
use crate::{
    catch::{catch_exceptions, unwrap_exc},
    convert::ccall_types::{CCallArg, CCallReturn},
    data::{
        layout::{
            is_bits::IsBits,
            typed_layout::HasLayout,
            valid_layout::{ValidField, ValidLayout},
        },
        managed::{
            private::ManagedPriv, type_name::TypeName, type_var::TypeVar, union_all::UnionAll, Ref,
        },
        types::{
            abstract_type::AnyType,
            construct_type::{BitsUnionCtor, ConstructType, IfConcreteElse},
            typecheck::Typecheck,
        },
    },
    error::{AccessError, ArrayLayoutError, InstantiationError, TypeError, CANNOT_DISPLAY_TYPE},
    memory::{
        get_tls,
        target::{unrooted::Unrooted, TargetResult},
    },
    prelude::{DataType, JlrsResult, LocalScope, Managed, Target, TargetType, Value, ValueData},
    private::Private,
};

// TODO: move to jl-sys
/// How an array has been allocated
#[repr(u8)]
#[derive(PartialEq, Debug)]
pub enum How {
    InlineOrForeign = 0,
    JuliaAllocated = 1,
    MallocAllocated = 2,
    PointerToOwner = 3,
}

/// Wrapper type for an array of rank `N` whose element type is `T`.
#[repr(transparent)]
pub struct ArrayBase<'scope, 'data, T, const N: isize>(
    NonNull<jl_array_t>,
    PhantomData<&'scope ()>,
    PhantomData<&'data mut ()>,
    PhantomData<T>,
);

impl<T, const N: isize> Clone for ArrayBase<'_, '_, T, N> {
    #[inline]
    fn clone(&self) -> Self {
        ArrayBase(self.0, PhantomData, PhantomData, PhantomData)
    }
}

impl<T, const N: isize> Copy for ArrayBase<'_, '_, T, N> {}

/// Constructor methods for typed arrays.
pub trait ConstructTypedArray<T: ConstructType, const N: isize> {
    /// Returns the array type for the element type `T` and the rank of the dimensions `dims`.
    fn array_type<'target, D, Tgt>(target: Tgt, dims: &D) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
        D: DimsExt;

    /// Allocate a new Julia array.
    ///
    /// The element type is `T`, the rank follows from the rank of `D`. If `N >= 0`, the rank of
    /// `D` must be equal to `N`. If this equality doesn't hold your program will fail to compile.
    /// If an exception is thrown when the array is allocated, it is caught and returned.
    ///
    /// Examples
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # fn main() {
    /// # let mut julia = Builder::new().start_local().unwrap();
    /// julia.local_scope::<_, 2>(|mut frame| {
    ///     // Allocate a 2x2 array of `u32`s with an implicit rank.
    ///     let array = TypedArray::<u32>::new(&mut frame, [2, 2]);
    ///     assert!(array.is_ok());
    ///
    ///     // Allocate a 4x2 array of `u32`s with an explicit rank.
    ///     let array = TypedRankedArray::<u32, 2>::new(&mut frame, [2, 2]);
    ///     assert!(array.is_ok());
    ///
    ///     // This fails to compile because the rank of the array doesn't match the rank of
    ///     // the dimensions.
    ///     // let array = TypedRankedArray::<u32, 3>::new(&mut frame, [2, 2]);
    /// });
    /// # }
    /// ```
    fn new<'target, D, Tgt>(target: Tgt, dims: D) -> ArrayBaseResult<'target, 'static, Tgt, T, N>
    where
        Tgt: Target<'target>,
        D: DimsExt,
    {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        if DimsRankAssert::<D, N>::NEEDS_RUNTIME_RANK_CHECK {
            assert_eq!(N as usize, dims.rank());
        }

        unsafe {
            let callback = || {
                let array_type = Self::array_type(&target, &dims).as_value();
                dims.alloc_array(&target, array_type)
            };

            let v = match catch_exceptions(callback, unwrap_exc) {
                Ok(arr) => Ok(arr.ptr()),
                Err(e) => Err(e),
            };

            target.result_from_ptr(v, Private)
        }
    }

    /// Allocate a new Julia array without checking any invariants.
    ///
    /// Safety:
    ///
    /// The element type is `T`, the rank follows from the rank of `D`. If `N >= 0`, the rank of
    /// `D` must be equal to `N`. If an exception is thrown when the array is allocated, it is not
    /// caught.
    unsafe fn new_unchecked<'target, D, Tgt>(
        target: Tgt,
        dims: D,
    ) -> ArrayBaseData<'target, 'static, Tgt, T, N>
    where
        Tgt: Target<'target>,
        D: DimsExt,
    {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        let array_type = Self::array_type(&target, &dims).as_value();
        let array = dims.alloc_array(&target, array_type);
        target.data_from_ptr(array.ptr(), Private)
    }

    /// Allocate a new Julia array that borrows its data from Rust.
    ///
    /// The element type is `T`, the rank follows from the rank of `D`. If `N >= 0`, the rank of
    /// `D` must be equal to `N`. If this equality doesn't hold `ArrayLayoutError::RankMismatch`
    /// is returned. If an exception is thrown when the array is allocated, it is caught and
    /// returned. The size of the dimensions must be equal to the length of `data`, otherwise
    /// `InstantiationError::ArraySizeMismatch` is returned.
    ///
    /// Note that the type of `data` is not `&mut [T]` but `&mut [U]`. The reason is that the
    /// type constructor can have more type parameters than its layout. `U` must implement
    /// `IsBits` and `ValidLayout`, and `T` must implement `HasLayout<Layout = U>` to guarantee
    /// that `U` is a valid representation of instances of `T`.
    ///
    /// NB: Because Julia didn't allocate the backing storage, there are some array functions in
    /// Julia that will throw an exception if you call them, e.g. `push!`. The reason is that the
    /// backing storage might need to be reallocated which is not possible.
    ///
    /// Examples
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # fn main() {
    /// # let mut julia = Builder::new().start_local().unwrap();
    /// julia.local_scope::<_, 3>(|mut frame| {
    ///     let mut data = vec![1u32, 2u32, 3u32, 4u32];
    ///
    ///     {
    ///         // Allocate a 2x2 array of `u32`s with an implicit rank.
    ///         let slice = data.as_mut_slice();
    ///         let array = TypedArray::<u32>::from_slice(&mut frame, slice, [2, 2]);
    ///         assert!(array.is_ok());
    ///         assert!(array.unwrap().is_ok());
    ///     }
    ///
    ///     {
    ///         // Allocate a 4x2 array of `u32`s with an explicit rank.
    ///         let slice = data.as_mut_slice();
    ///         let array = TypedRankedArray::<u32, 2>::from_slice(&mut frame, slice, [2, 2]);
    ///         assert!(array.is_ok());
    ///         assert!(array.unwrap().is_ok());
    ///     }
    ///
    ///     {
    ///         // This fails to compile because the rank of the array doesn't match the rank
    ///         // of the dimensions.
    ///         // let slice = data.as_mut_slice();
    ///         // let array = TypedRankedArray::<u32, 3>::from_slice(&mut frame, slice, [2, 2]);
    ///     }
    ///
    ///     {
    ///         // This fails because the size of the dimensions doesn't match the length of
    ///         // the data.
    ///         let slice = data.as_mut_slice();
    ///         let array = TypedArray::<u32>::from_slice(&mut frame, slice, [2, 1]);
    ///         assert!(array.is_err());
    ///     }
    /// });
    /// # }
    /// ```
    fn from_slice<'target, 'data, U, D, Tgt>(
        target: Tgt,
        data: &'data mut [U],
        dims: D,
    ) -> JlrsResult<ArrayBaseResult<'target, 'data, Tgt, T, N>>
    where
        Tgt: Target<'target>,
        D: DimsExt,
        T: HasLayout<'static, 'static, Layout = U>,
        U: ValidLayout + ValidField + IsBits,
    {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        if DimsRankAssert::<D, N>::NEEDS_RUNTIME_RANK_CHECK {
            let expected = N as usize;
            let found = dims.rank();
            if expected != found {
                Err(InstantiationError::ArrayRankMismatch { expected, found })?;
            }
        }

        if dims.size() != data.len() {
            Err(InstantiationError::ArraySizeMismatch {
                vec_size: data.len(),
                dim_size: dims.size(),
            })?;
        }

        unsafe {
            let callback = || {
                let array_type = Self::array_type(&target, &dims).as_value();
                dims.alloc_array_with_data(&target, array_type, data.as_ptr() as _)
            };

            let v = match catch_exceptions(callback, unwrap_exc) {
                Ok(arr) => Ok(arr.ptr()),
                Err(e) => Err(e),
            };

            Ok(target.result_from_ptr(v, Private))
        }
    }

    /// Allocate a new Julia array that borrows its data from Rust without checking any
    /// invariants.
    ///
    /// Safety:
    ///
    /// The element type is `T`, the rank follows from the rank of `D`. If `N >= 0`, the rank of
    /// `D` must be equal to `N`. If an exception is thrown when the array is allocated, it is not
    /// caught. The size of the dimensions must be equal to the length of `data`.
    unsafe fn from_slice_unchecked<'target, 'data, U, D, Tgt>(
        target: Tgt,
        data: &'data mut [U],
        dims: D,
    ) -> ArrayBaseData<'target, 'data, Tgt, T, N>
    where
        Tgt: Target<'target>,
        D: DimsExt,
        T: HasLayout<'static, 'static, Layout = U>,
        U: ValidLayout + ValidField + IsBits,
    {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        let array_type = Self::array_type(&target, &dims).as_value();
        let array = dims.alloc_array_with_data(&target, array_type, data.as_ptr() as _);
        target.data_from_ptr(array.ptr(), Private)
    }

    /// Allocate a new Julia array that takes owenership of a Rust `Vec`.
    ///
    /// The element type is `T`, the rank follows from the rank of `D`. If `N >= 0`, the rank of
    /// `D` must be equal to `N`. If this equality doesn't hold `ArrayLayoutError::RankMismatch`
    /// is returned. If an exception is thrown when the array is allocated, it is caught and
    /// returned. The size of the dimensions must be equal to the length of `data`, otherwise
    /// `InstantiationError::ArraySizeMismatch` is returned.
    ///
    /// Note that the type of `data` is not `Vec<T>` but `Vec<U>`. The reason is that the
    /// type constructor can have more type parameters than its layout. `U` must implement
    /// `IsBits` and `ValidLayout`, and `T` must implement `HasLayout<Layout = U>` to guarantee
    /// that `U` is a valid representation of instances of `T`.
    ///
    /// NB: Because Julia didn't allocate the backing storage, there are some array functions in
    /// Julia that will throw an exception if you call them, e.g. `push!`. The reason is that the
    /// backing storage might need to be reallocated which is not possible.
    ///
    /// Examples
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # fn main() {
    /// # let mut julia = Builder::new().start_local().unwrap();
    /// julia.local_scope::<_, 3>(|mut frame| {
    ///     {
    ///         // Allocate a 2x2 array of `u32`s with an implicit rank.
    ///         let data = vec![1u32, 2u32, 3u32, 4u32];
    ///         let array = TypedArray::<u32>::from_vec(&mut frame, data, [2, 2]);
    ///         assert!(array.is_ok());
    ///         assert!(array.unwrap().is_ok());
    ///     }
    ///
    ///     {
    ///         // Allocate a 4x2 array of `u32`s with an explicit rank.
    ///         let data = vec![1u32, 2u32, 3u32, 4u32];
    ///         let array = TypedRankedArray::<u32, 2>::from_vec(&mut frame, data, [2, 2]);
    ///         assert!(array.is_ok());
    ///         assert!(array.unwrap().is_ok());
    ///     }
    ///
    ///     {
    ///         // This fails to compile because the rank of the array doesn't match the rank
    ///         // of the dimensions.
    ///         // let data = vec![1u32, 2u32, 3u32, 4u32];
    ///         // let array = TypedRankedArray::<u32, 3>::from_vec(&mut frame, data, [2, 2]);
    ///     }
    ///
    ///     {
    ///         // This fails because the size of the dimensions doesn't match the length of
    ///         // the data.
    ///         let data = vec![1u32, 2u32, 3u32, 4u32];
    ///         let array = TypedArray::<u32>::from_vec(&mut frame, data, [2, 1]);
    ///         assert!(array.is_err());
    ///     }
    /// });
    /// # }
    /// ```
    fn from_vec<'target, U, D, Tgt>(
        target: Tgt,
        data: Vec<U>,
        dims: D,
    ) -> JlrsResult<ArrayBaseResult<'target, 'static, Tgt, T, N>>
    where
        Tgt: Target<'target>,
        D: DimsExt,
        T: HasLayout<'static, 'static, Layout = U>,
        U: ValidLayout + ValidField + IsBits,
    {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        if DimsRankAssert::<D, N>::NEEDS_RUNTIME_RANK_CHECK {
            let expected = N as usize;
            let found = dims.rank();
            if expected != found {
                Err(InstantiationError::ArrayRankMismatch { expected, found })?;
            }
        }

        if dims.size() != data.len() {
            Err(InstantiationError::ArraySizeMismatch {
                vec_size: data.len(),
                dim_size: dims.size(),
            })?;
        }

        let data = Box::leak(data.into_boxed_slice());

        unsafe {
            let callback = || {
                let array_type = Self::array_type(&target, &dims).as_value();
                let array = dims.alloc_array_with_data(&target, array_type, data.as_mut_ptr() as _);

                #[cfg(not(any(
                    feature = "julia-1-6",
                    feature = "julia-1-7",
                    feature = "julia-1-8",
                    feature = "julia-1-9",
                    feature = "julia-1-10",
                )))]
                let mem = jl_sys::inlined::jlrs_array_mem(array.ptr().as_ptr());
                #[cfg(any(
                    feature = "julia-1-6",
                    feature = "julia-1-7",
                    feature = "julia-1-8",
                    feature = "julia-1-9",
                    feature = "julia-1-10",
                ))]
                let mem = array.ptr().as_ptr().cast();

                jl_gc_add_ptr_finalizer(get_tls(), mem, droparray::<U> as *mut c_void);

                array
            };

            let v = match catch_exceptions(callback, unwrap_exc) {
                Ok(arr) => Ok(arr.ptr()),
                Err(e) => Err(e),
            };

            Ok(target.result_from_ptr(v, Private))
        }
    }

    /// Allocate a new Julia array that takes owenership of a Rust `Vec` without checking any
    /// invariants.
    ///
    /// Safety:
    ///
    /// The element type is `T`, the rank follows from the rank of `D`. If `N >= 0`, the rank of
    /// `D` must be equal to `N`. If an exception is thrown when the array is allocated, it is not
    /// caught. The size of the dimensions must be equal to the length of `data`.
    unsafe fn from_vec_unchecked<'target, U, D, Tgt>(
        target: Tgt,
        data: Vec<U>,
        dims: D,
    ) -> ArrayBaseData<'target, 'static, Tgt, T, N>
    where
        Tgt: Target<'target>,
        D: DimsExt,
        T: HasLayout<'static, 'static, Layout = U>,
        U: ValidLayout + ValidField + IsBits,
    {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        let data = Box::leak(data.into_boxed_slice());

        let array_type = Self::array_type(&target, &dims).as_value();
        let array = dims.alloc_array_with_data(&target, array_type, data.as_mut_ptr() as _);
        #[cfg(not(any(
            feature = "julia-1-6",
            feature = "julia-1-7",
            feature = "julia-1-8",
            feature = "julia-1-9",
            feature = "julia-1-10",
        )))]
        let mem = jl_sys::inlined::jlrs_array_mem(array.ptr().as_ptr());
        #[cfg(any(
            feature = "julia-1-6",
            feature = "julia-1-7",
            feature = "julia-1-8",
            feature = "julia-1-9",
            feature = "julia-1-10",
        ))]
        let mem = array.ptr().as_ptr().cast();

        jl_gc_add_ptr_finalizer(get_tls(), mem, droparray::<U> as *mut c_void);

        target.data_from_ptr(array.ptr(), Private)
    }

    /// Allocate a new Julia array that clones its data from Rust.
    ///
    /// The element type is `T`, the rank follows from the rank of `D`. If `N >= 0`, the rank of
    /// `D` must be equal to `N`. If this equality doesn't hold `ArrayLayoutError::RankMismatch`
    /// is returned. If an exception is thrown when the array is allocated, it is caught and
    /// returned. The size of the dimensions must be equal to the length of `data`, otherwise
    /// `InstantiationError::ArraySizeMismatch` is returned.
    ///
    /// Note that the type of `data` is not `AsRef<[T]>` but `AsRef<[U]>`. The reason is that the
    /// type constructor can have more type parameters than its layout. `U` must implement
    /// `IsBits` and `ValidLayout`, and `T` must implement `HasLamut_yout<Layout = U>` to guarantee
    /// that `U` is a valid representation of instances of `T`.
    ///
    /// Examples
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # fn main() {
    /// # let mut julia = Builder::new().start_local().unwrap();
    /// julia.local_scope::<_, 3>(|mut frame| {
    ///     let data = vec![1u32, 2u32, 3u32, 4u32];
    ///
    ///     {
    ///         // Allocate a 2x2 array of `u32`s with an implicit rank.
    ///         let slice = data.as_slice();
    ///         let array = TypedArray::<u32>::from_slice_cloned(&mut frame, slice, [2, 2]);
    ///         assert!(array.is_ok());
    ///         assert!(array.unwrap().is_ok());
    ///     }
    ///
    ///     {
    ///         // Allocate a 4x2 array of `u32`s with an explicit rank.
    ///         let slice = data.as_slice();
    ///         let array = TypedRankedArray::<u32, 2>::from_slice_cloned(&mut frame, slice, [2, 2]);
    ///         assert!(array.is_ok());
    ///         assert!(array.unwrap().is_ok());
    ///     }
    ///
    ///     {
    ///         // This fails to compile because the rank of the array doesn't match the rank
    ///         // of the dimensions.
    ///         // let slice = data.as_slice();
    ///         // let array = TypedRankedArray::<u32, 3>::from_slice_cloned(&mut frame, slice, [2, 2]);
    ///     }
    ///
    ///     {
    ///         // This fails because the size of the dimensions doesn't match the length of
    ///         // the data.
    ///         let slice = data.as_slice();
    ///         let array = TypedArray::<u32>::from_slice_cloned(&mut frame, slice, [2, 1]);
    ///         assert!(array.is_err());
    ///     }
    /// });
    /// # }
    /// ```
    fn from_slice_cloned<'target, V, U, D, Tgt>(
        target: Tgt,
        data: V,
        dims: D,
    ) -> JlrsResult<ArrayBaseResult<'target, 'static, Tgt, T, N>>
    where
        Tgt: Target<'target>,
        D: DimsExt,
        T: HasLayout<'static, 'static, Layout = U>,
        U: ValidLayout + ValidField + IsBits + Clone,
        V: AsRef<[U]>,
    {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        if DimsRankAssert::<D, N>::NEEDS_RUNTIME_RANK_CHECK {
            let expected = N as usize;
            let found = dims.rank();
            if expected != found {
                Err(InstantiationError::ArrayRankMismatch { expected, found })?;
            }
        }

        let data = data.as_ref();
        let len = data.len();
        let dim_size = dims.size();
        if len != dim_size {
            Err(InstantiationError::ArraySizeMismatch {
                vec_size: len,
                dim_size: dim_size,
            })?;
        }

        unsafe {
            let arr = match Self::new(&target, dims) {
                Ok(arr) => arr,
                Err(e) => return Ok(Err(e.as_value().root(target))),
            };

            let array_data = jlrs_array_data(arr.as_managed().unwrap(Private));
            let array_data_slice = std::slice::from_raw_parts_mut(array_data as _, len);
            array_data_slice.clone_from_slice(data);

            Ok(Ok(arr.root(target)))
        }
    }

    /// Allocate a new Julia array that clones its data from Rust without checking any invariants.
    ///
    /// The element type is `T`, the rank follows from the rank of `D`. If `N >= 0`, the rank of
    /// `D` must be equal to `N`. If an exception is thrown when the array is allocated, it is not
    /// caught. The size of the dimensions must be equal to the length of `data`.
    ///
    /// Note that the type of `data` is not `AsRef<[T]>` but `AsRef<[U]>`. The reason is that the
    /// type constructor can have more type parameters than its layout. `U` must implement
    /// `IsBits` and `ValidLayout`, and `T` must implement `HasLamut_yout<Layout = U>` to guarantee
    /// that `U` is a valid representation of instances of `T`.
    unsafe fn from_slice_cloned_unchecked<'target, V, U, D, Tgt>(
        target: Tgt,
        data: V,
        dims: D,
    ) -> ArrayBaseData<'target, 'static, Tgt, T, N>
    where
        Tgt: Target<'target>,
        D: DimsExt,
        T: HasLayout<'static, 'static, Layout = U>,
        U: ValidLayout + ValidField + IsBits + Clone,
        V: AsRef<[U]>,
    {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        let data = data.as_ref();
        let len = data.len();

        let arr = Self::new_unchecked(&target, dims);
        let array_data = jlrs_array_data(arr.as_managed().unwrap(Private));
        let array_data_slice = std::slice::from_raw_parts_mut(array_data as _, len);
        array_data_slice.clone_from_slice(data);

        arr.root(target)
    }

    /// Allocate a new Julia array that copies its data from Rust.
    ///
    /// The element type is `T`, the rank follows from the rank of `D`. If `N >= 0`, the rank of
    /// `D` must be equal to `N`. If this equality doesn't hold `ArrayLayoutError::RankMismatch`
    /// is returned. If an exception is thrown when the array is allocated, it is caught and
    /// returned. The size of the dimensions must be equal to the length of `data`, otherwise
    /// `InstantiationError::ArraySizeMismatch` is returned.
    ///
    /// Note that the type of `data` is not `AsRef<[T]>` but `AsRef<[U]>`. The reason is that the
    /// type constructor can have more type parameters than its layout. `U` must implement
    /// `IsBits` and `ValidLayout`, and `T` must implement `HasLamut_yout<Layout = U>` to guarantee
    /// that `U` is a valid representation of instances of `T`.
    ///
    /// Examples
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # fn main() {
    /// # let mut julia = Builder::new().start_local().unwrap();
    /// julia.local_scope::<_, 3>(|mut frame| {
    ///     let data = vec![1u32, 2u32, 3u32, 4u32];
    ///
    ///     {
    ///         // Allocate a 2x2 array of `u32`s with an implicit rank.
    ///         let slice = data.as_slice();
    ///         let array = TypedArray::<u32>::from_slice_copied(&mut frame, slice, [2, 2]);
    ///         assert!(array.is_ok());
    ///         assert!(array.unwrap().is_ok());
    ///     }
    ///
    ///     {
    ///         // Allocate a 4x2 array of `u32`s with an explicit rank.
    ///         let slice = data.as_slice();
    ///         let array = TypedRankedArray::<u32, 2>::from_slice_copied(&mut frame, slice, [2, 2]);
    ///         assert!(array.is_ok());
    ///         assert!(array.unwrap().is_ok());
    ///     }
    ///
    ///     {
    ///         // This fails to compile because the rank of the array doesn't match the rank
    ///         // of the dimensions.
    ///         // let slice = data.as_slice();
    ///         // let array = TypedRankedArray::<u32, 3>::from_slice_copied(&mut frame, slice, [2, 2]);
    ///     }
    ///
    ///     {
    ///         // This fails because the size of the dimensions doesn't match the length of
    ///         // the data.
    ///         let slice = data.as_slice();
    ///         let array = TypedArray::<u32>::from_slice_copied(&mut frame, slice, [2, 1]);
    ///         assert!(array.is_err());
    ///     }
    /// });
    /// # }
    /// ```
    fn from_slice_copied<'target, V, U, D, Tgt>(
        target: Tgt,
        data: V,
        dims: D,
    ) -> JlrsResult<ArrayBaseResult<'target, 'static, Tgt, T, N>>
    where
        Tgt: Target<'target>,
        D: DimsExt,
        T: HasLayout<'static, 'static, Layout = U>,
        U: ValidLayout + ValidField + IsBits + Copy,
        V: AsRef<[U]>,
    {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        if DimsRankAssert::<D, N>::NEEDS_RUNTIME_RANK_CHECK {
            let expected = N as usize;
            let found = dims.rank();
            if expected != found {
                Err(InstantiationError::ArrayRankMismatch { expected, found })?;
            }
        }

        let data = data.as_ref();
        let len = data.len();
        let dim_size = dims.size();
        if len != dim_size {
            Err(InstantiationError::ArraySizeMismatch {
                vec_size: len,
                dim_size: dim_size,
            })?;
        }

        unsafe {
            let arr = match Self::new(&target, dims) {
                Ok(arr) => arr,
                Err(e) => return Ok(Err(e.as_value().root(target))),
            };

            let array_data = jlrs_array_data(arr.as_managed().unwrap(Private));
            let array_data_slice = std::slice::from_raw_parts_mut(array_data as _, len);
            array_data_slice.copy_from_slice(data);

            Ok(Ok(arr.root(target)))
        }
    }

    /// Allocate a new Julia array that clones its data from Rust without checking any invariants.
    ///
    /// The element type is `T`, the rank follows from the rank of `D`. If `N >= 0`, the rank of
    /// `D` must be equal to `N`. If an exception is thrown when the array is allocated, it is not
    /// caught. The size of the dimensions must be equal to the length of `data`.
    ///
    /// Note that the type of `data` is not `AsRef<[T]>` but `AsRef<[U]>`. The reason is that the
    /// type constructor can have more type parameters than its layout. `U` must implement
    /// `IsBits` and `ValidLayout`, and `T` must implement `HasLamut_yout<Layout = U>` to guarantee
    /// that `U` is a valid representation of instances of `T`.
    unsafe fn from_slice_copied_unchecked<'target, V, U, D, Tgt>(
        target: Tgt,
        data: V,
        dims: D,
    ) -> ArrayBaseData<'target, 'static, Tgt, T, N>
    where
        Tgt: Target<'target>,
        D: DimsExt,
        T: HasLayout<'static, 'static, Layout = U>,
        U: ValidLayout + ValidField + IsBits + Copy,
        V: AsRef<[U]>,
    {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        let data = data.as_ref();
        let len = data.len();

        let arr = Self::new_unchecked(&target, dims);
        let array_data = jlrs_array_data(arr.as_managed().unwrap(Private));
        let array_data_slice = std::slice::from_raw_parts_mut(array_data as _, len);
        array_data_slice.copy_from_slice(data);

        arr.root(target)
    }
}

impl<T: ConstructType, const N: isize> ConstructTypedArray<T, N> for ArrayBase<'_, '_, T, N> {
    fn array_type<'target, D, Tgt>(target: Tgt, dims: &D) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
        D: DimsExt,
    {
        dims.array_type::<T, _>(target)
    }
}

impl<const N: isize> ArrayBase<'_, '_, Unknown, N> {
    /// Allocate a new Julia array for elements of some provided type.
    ///
    /// The element type is `ty`, the rank follows from the rank of `D`. If `N >= 0`, the rank of
    /// `D` must be equal to `N`. If this equality doesn't hold `ArrayLayoutError::RankMismatch`
    /// is returned. If an exception is thrown when the array is allocated, it is caught and
    /// returned.
    ///
    /// Examples
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # fn main() {
    /// # let mut julia = Builder::new().start_local().unwrap();
    /// julia.local_scope::<_, 2>(|mut frame| {
    ///     let ty = DataType::uint32_type(&frame).as_value();
    ///
    ///     // Allocate a 2x2 array of `u32`s with an implicit rank.
    ///     let array = Array::new_for(&mut frame, ty, [2, 2]);
    ///     assert!(array.is_ok());
    ///
    ///     // Allocate a 4x2 array of `u32`s with an explicit rank.
    ///     let array = RankedArray::<2>::new_for(&mut frame, ty, [2, 2]);
    ///     assert!(array.is_ok());
    ///
    ///     // This fails to compile because the rank of the array doesn't match the rank of
    ///     // the dimensions.
    ///     // let array = RankedArray::<3>::new_for(&mut frame, ty, [2, 2]);
    /// });
    /// # }
    /// ```
    pub fn new_for<'target, D, Tgt>(
        target: Tgt,
        ty: Value,
        dims: D,
    ) -> ArrayBaseResult<'target, 'static, Tgt, Unknown, N>
    where
        Tgt: Target<'target>,
        D: DimsExt,
    {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        if DimsRankAssert::<D, N>::NEEDS_RUNTIME_RANK_CHECK {
            assert_eq!(N as usize, dims.rank());
        }

        unsafe {
            let callback = || {
                // array_type should be a concrete type.
                let array_type = jl_apply_array_type(ty.unwrap(Private), D::RANK as _);
                let array_type = Value::wrap_non_null(NonNull::new_unchecked(array_type), Private);
                let array = dims.alloc_array(&target, array_type);
                array
            };

            let v = match catch_exceptions(callback, unwrap_exc) {
                Ok(arr) => Ok(arr.ptr()),
                Err(e) => Err(e),
            };

            target.result_from_ptr(v, Private)
        }
    }

    /// Allocate a new Julia array for elements of some provided type without checking any
    /// invariants.
    ///
    /// Safety:
    ///
    /// The element type is `ty`, the rank follows from the rank of `D`. If `N >= 0`, the rank of
    /// `D` must be equal to `N`. If an exception is thrown when the array is allocated, it is not
    /// caught.
    pub unsafe fn new_for_unchecked<'target, D, Tgt>(
        target: Tgt,
        ty: Value,
        dims: D,
    ) -> ArrayBaseData<'target, 'static, Tgt, Unknown, N>
    where
        Tgt: Target<'target>,
        D: DimsExt,
    {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        // array_type should be a concrete type.
        let array_type = jl_apply_array_type(ty.unwrap(Private), D::RANK as _);
        let array_type = Value::wrap_non_null(NonNull::new_unchecked(array_type), Private);
        let array = dims.alloc_array(&target, array_type);

        target.data_from_ptr(array.ptr(), Private)
    }

    /// Allocate a new Julia array that borrows its data from Rust with some provided element
    /// type.
    ///
    /// The element type is `ty`, the rank follows from the rank of `D`. If `N >= 0`, the rank of
    /// `D` must be equal to `N`. If this equality doesn't hold `ArrayLayoutError::RankMismatch`
    /// is returned. If an exception is thrown when the array is allocated, it is caught and
    /// returned. The size of the dimensions must be equal to the length of `data`, otherwise
    /// `InstantiationError::ArraySizeMismatch` is returned.
    ///
    /// The layout of `U` must be a valid layout for `ty`, if this is not true
    /// `AccessError::InvalidLayout` is returned.
    ///
    /// NB: Because Julia didn't allocate the backing storage, there are some array functions in
    /// Julia that will throw an exception if you call them, e.g. `push!`. The reason is that the
    /// backing storage might need to be reallocated which is not possible.
    ///
    /// Examples
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # fn main() {
    /// # let mut julia = Builder::new().start_local().unwrap();
    /// julia.local_scope::<_, 3>(|mut frame| {
    ///     let mut data = vec![1u32, 2u32, 3u32, 4u32];
    ///     let ty = DataType::uint32_type(&frame).as_value();
    ///
    ///     {
    ///         // Allocate a 2x2 array of `u32`s with an implicit rank.
    ///         let slice = data.as_mut_slice();
    ///         let array = Array::from_slice_for(&mut frame, ty, slice, [2, 2]);
    ///         assert!(array.is_ok());
    ///         assert!(array.unwrap().is_ok());
    ///     }
    ///
    ///     {
    ///         // Allocate a 4x2 array of `u32`s with an explicit rank.
    ///         let slice = data.as_mut_slice();
    ///         let array = RankedArray::<2>::from_slice_for(&mut frame, ty, slice, [2, 2]);
    ///         assert!(array.is_ok());
    ///         assert!(array.unwrap().is_ok());
    ///     }
    ///
    ///     {
    ///         // This fails to compile because the rank of the array doesn't match the rank
    ///         // of the dimensions.
    ///         // let slice = data.as_mut_slice();
    ///         // let array = RankedArray::<3>::from_slice_for(&mut frame, ty, slice, [2, 2]);
    ///     }
    ///
    ///     {
    ///         // This fails because the size of the dimensions doesn't match the length of
    ///         // the data.
    ///         let slice = data.as_mut_slice();
    ///         let array = Array::from_slice_for(&mut frame, ty, slice, [2, 1]);
    ///         assert!(array.is_err());
    ///     }
    ///
    ///     {
    ///         // This fails because the layout of the data is incompatible with `ty`.
    ///         let ty = DataType::uint64_type(&frame).as_value();
    ///         let slice = data.as_mut_slice();
    ///         let array = Array::from_slice_for(&mut frame, ty, slice, [2, 2]);
    ///         assert!(array.is_err());
    ///     }
    /// });
    /// # }
    /// ```
    pub fn from_slice_for<'target, 'data, U, D, Tgt>(
        target: Tgt,
        ty: Value,
        data: &'data mut [U],
        dims: D,
    ) -> JlrsResult<ArrayBaseResult<'target, 'data, Tgt, Unknown, N>>
    where
        Tgt: Target<'target>,
        D: DimsExt,
        U: ValidLayout + ValidField + IsBits,
    {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        if DimsRankAssert::<D, N>::NEEDS_RUNTIME_RANK_CHECK {
            let expected = N as usize;
            let found = dims.rank();
            if expected != found {
                Err(InstantiationError::ArrayRankMismatch { expected, found })?;
            }
        }

        if dims.size() != data.len() {
            Err(InstantiationError::ArraySizeMismatch {
                vec_size: data.len(),
                dim_size: dims.size(),
            })?;
        }

        if !U::valid_layout(ty) {
            let value_type = ty.display_string_or(CANNOT_DISPLAY_TYPE).into();
            Err(AccessError::InvalidLayout { value_type })?;
        }

        unsafe {
            let callback = || {
                // array_type should be a concrete type.
                let array_type = jl_apply_array_type(ty.unwrap(Private), D::RANK as _);
                let array_type = Value::wrap_non_null(NonNull::new_unchecked(array_type), Private);
                let array = dims.alloc_array_with_data(&target, array_type, data.as_ptr() as _);
                array
            };

            let v = match catch_exceptions(callback, unwrap_exc) {
                Ok(arr) => Ok(arr.ptr()),
                Err(e) => Err(e),
            };

            Ok(target.result_from_ptr(v, Private))
        }
    }

    /// Allocate a new Julia array that borrows its data from Rust with some provided element
    /// type without checking any invariants.
    ///
    /// Safety:
    ///
    /// The element type is `ty`, the rank follows from the rank of `D`. If `N >= 0`, the rank of
    /// `D` must be equal to `N`.If an exception is thrown when the array is allocated, it is not
    /// caught. The size of the dimensions must be equal to the length of `data`. The layout of
    /// `U` must be a valid layout for `ty`.
    pub unsafe fn from_slice_for_unchecked<'target, 'data, U, D, Tgt>(
        target: Tgt,
        ty: Value,
        data: &'data mut [U],
        dims: D,
    ) -> ArrayBaseData<'target, 'data, Tgt, Unknown, N>
    where
        Tgt: Target<'target>,
        D: DimsExt,
        U: ValidLayout + ValidField + IsBits,
    {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;

        // array_type should be a concrete type.
        let array_type = jl_apply_array_type(ty.unwrap(Private), D::RANK as _);
        let array_type = Value::wrap_non_null(NonNull::new_unchecked(array_type), Private);
        let array = dims.alloc_array_with_data(&target, array_type, data.as_ptr() as _);
        target.data_from_ptr(array.ptr(), Private)
    }

    /// Allocate a new Julia array that takes ownership of a Rust `Vec` with some provided element
    /// type.
    ///
    /// The element type is `ty`, the rank follows from the rank of `D`. If `N >= 0`, the rank of
    /// `D` must be equal to `N`. If this equality doesn't hold `ArrayLayoutError::RankMismatch`
    /// is returned. If an exception is thrown when the array is allocated, it is caught and
    /// returned. The size of the dimensions must be equal to the length of `data`, otherwise
    /// `InstantiationError::ArraySizeMismatch` is returned.
    ///
    /// The layout of `U` must be a valid layout for `ty`, if this is not true
    /// `AccessError::InvalidLayout` is returned.
    ///
    /// NB: Because Julia didn't allocate the backing storage, there are some array functions in
    /// Julia that will throw an exception if you call them, e.g. `push!`. The reason is that the
    /// backing storage might need to be reallocated which is not possible.
    ///
    /// Examples
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # fn main() {
    /// # let mut julia = Builder::new().start_local().unwrap();
    /// julia.local_scope::<_, 4>(|mut frame| {
    ///     let ty = DataType::uint32_type(&frame).as_value();
    ///
    ///     {
    ///         // Allocate a 2x2 array of `u32`s with an implicit rank.
    ///         let data = vec![1u32, 2u32, 3u32, 4u32];
    ///         let array = Array::from_vec_for(&mut frame, ty, data, [2, 2]);
    ///         assert!(array.is_ok());
    ///         assert!(array.unwrap().is_ok());
    ///     }
    ///
    ///     {
    ///         // Allocate a 4x2 array of `u32`s with an explicit rank.
    ///         let data = vec![1u32, 2u32, 3u32, 4u32];
    ///         let array = RankedArray::<2>::from_vec_for(&mut frame, ty, data, [2, 2]);
    ///         assert!(array.is_ok());
    ///         assert!(array.unwrap().is_ok());
    ///     }
    ///
    ///     {
    ///         // This fails to compile because the rank of the array doesn't match the rank
    ///         // of the dimensions.
    ///         // let data = vec![1u32, 2u32, 3u32, 4u32];
    ///         // let array = RankedArray::<3>::from_vec_for(&mut frame, ty, data, [2, 2]);
    ///     }
    ///
    ///     {
    ///         // This fails because the size of the dimensions doesn't match the length of
    ///         // the data.
    ///         let mut data = vec![1u32, 2u32, 3u32, 4u32];
    ///         let array = Array::from_vec_for(&mut frame, ty, data, [2, 1]);
    ///         assert!(array.is_err());
    ///     }
    ///
    ///     {
    ///         // This fails because the layout of the data is incompatible with `ty`.
    ///         let data = vec![1u32, 2u32, 3u32, 4u32];
    ///         let ty = DataType::uint64_type(&frame).as_value();
    ///         let array = Array::from_vec_for(&mut frame, ty, data, [2, 2]);
    ///         assert!(array.is_err());
    ///     }
    /// });
    /// # }
    /// ```
    pub fn from_vec_for<'target, U, D, Tgt>(
        target: Tgt,
        ty: Value,
        data: Vec<U>,
        dims: D,
    ) -> JlrsResult<ArrayBaseResult<'target, 'static, Tgt, Unknown, N>>
    where
        Tgt: Target<'target>,
        D: DimsExt,
        U: ValidLayout + ValidField + IsBits,
    {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        if DimsRankAssert::<D, N>::NEEDS_RUNTIME_RANK_CHECK {
            let expected = N as usize;
            let found = dims.rank();
            if expected != found {
                Err(InstantiationError::ArrayRankMismatch { expected, found })?;
            }
        }

        if dims.size() != data.len() {
            Err(InstantiationError::ArraySizeMismatch {
                vec_size: data.len(),
                dim_size: dims.size(),
            })?;
        }

        if !U::valid_layout(ty) {
            let value_type = ty.display_string_or(CANNOT_DISPLAY_TYPE).into();
            Err(AccessError::InvalidLayout { value_type })?;
        }

        let data = Box::leak(data.into_boxed_slice());

        unsafe {
            let callback = || {
                // array_type should be a concrete type.
                let array_type = jl_apply_array_type(ty.unwrap(Private), D::RANK as _);
                let array_type = Value::wrap_non_null(NonNull::new_unchecked(array_type), Private);
                let array = dims.alloc_array_with_data(&target, array_type, data.as_mut_ptr() as _);
                #[cfg(not(any(
                    feature = "julia-1-6",
                    feature = "julia-1-7",
                    feature = "julia-1-8",
                    feature = "julia-1-9",
                    feature = "julia-1-10",
                )))]
                let mem = jl_sys::inlined::jlrs_array_mem(array.ptr().as_ptr());
                #[cfg(any(
                    feature = "julia-1-6",
                    feature = "julia-1-7",
                    feature = "julia-1-8",
                    feature = "julia-1-9",
                    feature = "julia-1-10",
                ))]
                let mem = array.ptr().as_ptr().cast();

                jl_gc_add_ptr_finalizer(get_tls(), mem, droparray::<U> as *mut c_void);

                array
            };

            let v = match catch_exceptions(callback, unwrap_exc) {
                Ok(arr) => Ok(arr.ptr()),
                Err(e) => Err(e),
            };

            Ok(target.result_from_ptr(v, Private))
        }
    }

    /// Allocate a new Julia array that takes ownership of a Rust `Vec` with some provided element
    /// type without checking any invariants.
    ///
    /// Safety:
    ///
    /// The element type is `ty`, the rank follows from the rank of `D`. If `N >= 0`, the rank of
    /// `D` must be equal to `N`. If an exception is thrown when the array is allocated, it is not
    /// caught. The size of the dimensions must be equal to the length of `data`. The layout of
    /// `U` must be a valid layout for `ty`.
    pub unsafe fn from_vec_for_unchecked<'target, U, D, Tgt>(
        target: Tgt,
        ty: Value,
        data: Vec<U>,
        dims: D,
    ) -> ArrayBaseData<'target, 'static, Tgt, Unknown, N>
    where
        Tgt: Target<'target>,
        D: DimsExt,
        U: ValidLayout + ValidField + IsBits,
    {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        let data = Box::leak(data.into_boxed_slice());

        // array_type should be a concrete type.
        let array_type = jl_apply_array_type(ty.unwrap(Private), D::RANK as _);
        let array_type = Value::wrap_non_null(NonNull::new_unchecked(array_type), Private);
        let array = dims.alloc_array_with_data(&target, array_type, data.as_mut_ptr() as _);
        #[cfg(not(any(
            feature = "julia-1-6",
            feature = "julia-1-7",
            feature = "julia-1-8",
            feature = "julia-1-9",
            feature = "julia-1-10",
        )))]
        let mem = jl_sys::inlined::jlrs_array_mem(array.ptr().as_ptr());
        #[cfg(any(
            feature = "julia-1-6",
            feature = "julia-1-7",
            feature = "julia-1-8",
            feature = "julia-1-9",
            feature = "julia-1-10",
        ))]
        let mem = array.ptr().as_ptr().cast();

        jl_gc_add_ptr_finalizer(get_tls(), mem, droparray::<U> as *mut c_void);

        target.data_from_ptr(array.ptr(), Private)
    }

    /// Allocate a new Julia array with some provided element type that clones its data from Rust.
    ///
    /// The element type is `ty`, the rank follows from the rank of `D`. If `N >= 0`, the rank of
    /// `D` must be equal to `N`. If this equality doesn't hold `ArrayLayoutError::RankMismatch`
    /// is returned. If an exception is thrown when the array is allocated, it is caught and
    /// returned. The size of the dimensions must be equal to the length of `data`, otherwise
    /// `InstantiationError::ArraySizeMismatch` is returned.
    ///
    /// Note that the type of `data` is not `AsRef<[T]>` but `AsRef<[U]>`. The reason is that the
    /// type constructor can have more type parameters than its layout. `U` must implement
    /// `IsBits` and `ValidLayout`, and `T` must implement `HasLamut_yout<Layout = U>` to guarantee
    /// that `U` is a valid representation of instances of `T`.
    ///
    /// Examples
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # fn main() {
    /// # let mut julia = Builder::new().start_local().unwrap();
    /// julia.local_scope::<_, 4>(|mut frame| {
    ///     let data = vec![1u32, 2u32, 3u32, 4u32];
    ///     let ty = DataType::uint32_type(&frame).as_value();
    ///
    ///     {
    ///         // Allocate a 2x2 array of `u32`s with an implicit rank.
    ///         let slice = data.as_slice();
    ///         let array = Array::from_slice_cloned_for(&mut frame, ty, slice, [2, 2]);
    ///         assert!(array.is_ok());
    ///         assert!(array.unwrap().is_ok());
    ///     }
    ///
    ///     {
    ///         // Allocate a 4x2 array of `u32`s with an explicit rank.
    ///         let slice = data.as_slice();
    ///         let array = RankedArray::<2>::from_slice_cloned_for(&mut frame, ty, slice, [2, 2]);
    ///         assert!(array.is_ok());
    ///         assert!(array.unwrap().is_ok());
    ///     }
    ///
    ///     {
    ///         // This fails to compile because the rank of the array doesn't match the rank
    ///         // of the dimensions.
    ///         // let slice = data.as_slice();
    ///         // let array = RankedArray::<3>::from_slice_cloned_for(&mut frame, ty, slice, [2, 2]);
    ///     }
    ///
    ///     {
    ///         // This fails because the size of the dimensions doesn't match the length of
    ///         // the data.
    ///         let slice = data.as_slice();
    ///         let array = Array::from_slice_cloned_for(&mut frame, ty, slice, [2, 1]);
    ///         assert!(array.is_err());
    ///     }
    ///
    ///     {
    ///         // This fails because the layout of the data is incompatible with `ty`.
    ///         let slice = data.as_slice();
    ///         let ty = DataType::uint64_type(&frame).as_value();
    ///         let array = Array::from_slice_cloned_for(&mut frame, ty, slice, [2, 1]);
    ///         assert!(array.is_err());
    ///     }
    /// });
    /// # }
    /// ```
    pub fn from_slice_cloned_for<'target, V, U, D, Tgt>(
        target: Tgt,
        ty: Value,
        data: V,
        dims: D,
    ) -> JlrsResult<ArrayBaseResult<'target, 'static, Tgt, Unknown, N>>
    where
        Tgt: Target<'target>,
        D: DimsExt,
        U: ValidLayout + ValidField + IsBits + Clone,
        V: AsRef<[U]>,
    {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        if DimsRankAssert::<D, N>::NEEDS_RUNTIME_RANK_CHECK {
            let expected = N as usize;
            let found = dims.rank();
            if expected != found {
                Err(InstantiationError::ArrayRankMismatch { expected, found })?;
            }
        }

        let data = data.as_ref();
        let len = data.len();
        let dim_size = dims.size();
        if len != dim_size {
            Err(InstantiationError::ArraySizeMismatch {
                vec_size: len,
                dim_size: dim_size,
            })?;
        }

        if !U::valid_layout(ty) {
            let value_type = ty.display_string_or(CANNOT_DISPLAY_TYPE).into();
            Err(AccessError::InvalidLayout { value_type })?;
        }

        unsafe {
            match Self::new_for(&target, ty, dims) {
                Ok(arr) => {
                    let array_data = jlrs_array_data(arr.as_managed().unwrap(Private));
                    let array_data_slice = std::slice::from_raw_parts_mut(array_data as _, len);
                    array_data_slice.clone_from_slice(data);

                    Ok(Ok(arr.root(target)))
                }
                Err(err) => Ok(Err(err.as_value().root(target))),
            }
        }
    }

    /// Allocate a new Julia array that clones its data from Rust without checking any invariants.
    ///
    /// The element type is `T`, the rank follows from the rank of `D`. If `N >= 0`, the rank of
    /// `D` must be equal to `N`. If an exception is thrown when the array is allocated, it is not
    /// caught. The size of the dimensions must be equal to the length of `data`.
    ///
    /// Note that the type of `data` is not `AsRef<[T]>` but `AsRef<[U]>`. The reason is that the
    /// type constructor can have more type parameters than its layout. `U` must implement
    /// `IsBits` and `ValidLayout`, and `T` must implement `HasLamut_yout<Layout = U>` to guarantee
    /// that `U` is a valid representation of instances of `T`.
    pub unsafe fn from_slice_cloned_for_unchecked<'target, V, U, D, Tgt>(
        target: Tgt,
        ty: Value,
        data: V,
        dims: D,
    ) -> ArrayBaseData<'target, 'static, Tgt, Unknown, N>
    where
        Tgt: Target<'target>,
        D: DimsExt,
        U: ValidLayout + ValidField + IsBits + Clone,
        V: AsRef<[U]>,
    {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;

        let data = data.as_ref();
        let len = data.len();

        let arr = Self::new_for_unchecked(&target, ty, dims);
        let array_data = jlrs_array_data(arr.as_managed().unwrap(Private));
        let array_data_slice = std::slice::from_raw_parts_mut(array_data as _, len);
        array_data_slice.clone_from_slice(data);

        arr.root(target)
    }

    /// Allocate a new Julia array with some provided element type that copies its data from Rust.
    ///
    /// The element type is `ty`, the rank follows from the rank of `D`. If `N >= 0`, the rank of
    /// `D` must be equal to `N`. If this equality doesn't hold `ArrayLayoutError::RankMismatch`
    /// is returned. If an exception is thrown when the array is allocated, it is caught and
    /// returned. The size of the dimensions must be equal to the length of `data`, otherwise
    /// `InstantiationError::ArraySizeMismatch` is returned.
    ///
    /// Note that the type of `data` is not `AsRef<[T]>` but `AsRef<[U]>`. The reason is that the
    /// type constructor can have more type parameters than its layout. `U` must implement
    /// `IsBits` and `ValidLayout`, and `T` must implement `HasLamut_yout<Layout = U>` to guarantee
    /// that `U` is a valid representation of instances of `T`.
    ///
    /// Examples
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # fn main() {
    /// # let mut julia = Builder::new().start_local().unwrap();
    /// julia.local_scope::<_, 4>(|mut frame| {
    ///     let data = vec![1u32, 2u32, 3u32, 4u32];
    ///     let ty = DataType::uint32_type(&frame).as_value();
    ///
    ///     {
    ///         // Allocate a 2x2 array of `u32`s with an implicit rank.
    ///         let slice = data.as_slice();
    ///         let array = Array::from_slice_copied_for(&mut frame, ty, slice, [2, 2]);
    ///         assert!(array.is_ok());
    ///         assert!(array.unwrap().is_ok());
    ///     }
    ///
    ///     {
    ///         // Allocate a 4x2 array of `u32`s with an explicit rank.
    ///         let slice = data.as_slice();
    ///         let array = RankedArray::<2>::from_slice_copied_for(&mut frame, ty, slice, [2, 2]);
    ///         assert!(array.is_ok());
    ///         assert!(array.unwrap().is_ok());
    ///     }
    ///
    ///     {
    ///         // This fails to compile because the rank of the array doesn't match the rank
    ///         // of the dimensions.
    ///         // let slice = data.as_slice();
    ///         // let array = RankedArray::<3>::from_slice_copied_for(&mut frame, ty, slice, [2, 2]);
    ///     }
    ///
    ///     {
    ///         // This fails because the size of the dimensions doesn't match the length of
    ///         // the data.
    ///         let slice = data.as_slice();
    ///         let array = Array::from_slice_copied_for(&mut frame, ty, slice, [2, 1]);
    ///         assert!(array.is_err());
    ///     }
    ///
    ///     {
    ///         // This fails because the layout of the data is incompatible with `ty`.
    ///         let slice = data.as_slice();
    ///         let ty = DataType::uint64_type(&frame).as_value();
    ///         let array = Array::from_slice_copied_for(&mut frame, ty, slice, [2, 1]);
    ///         assert!(array.is_err());
    ///     }
    /// });
    /// # }
    /// ```
    pub fn from_slice_copied_for<'target, V, U, D, Tgt>(
        target: Tgt,
        ty: Value,
        data: V,
        dims: D,
    ) -> JlrsResult<ArrayBaseResult<'target, 'static, Tgt, Unknown, N>>
    where
        Tgt: Target<'target>,
        D: DimsExt,
        U: ValidLayout + ValidField + IsBits + Copy,
        V: AsRef<[U]>,
    {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;
        if DimsRankAssert::<D, N>::NEEDS_RUNTIME_RANK_CHECK {
            let expected = N as usize;
            let found = dims.rank();
            if expected != found {
                Err(InstantiationError::ArrayRankMismatch { expected, found })?;
            }
        }

        let data = data.as_ref();
        let len = data.len();
        let dim_size = dims.size();
        if len != dim_size {
            Err(InstantiationError::ArraySizeMismatch {
                vec_size: len,
                dim_size: dim_size,
            })?;
        }

        if !U::valid_layout(ty) {
            let value_type = ty.display_string_or(CANNOT_DISPLAY_TYPE).into();
            Err(AccessError::InvalidLayout { value_type })?;
        }

        unsafe {
            match Self::new_for(&target, ty, dims) {
                Ok(arr) => {
                    let array_data = jlrs_array_data(arr.as_managed().unwrap(Private));
                    let array_data_slice = std::slice::from_raw_parts_mut(array_data as _, len);
                    array_data_slice.copy_from_slice(data);

                    Ok(Ok(arr.root(target)))
                }
                Err(err) => Ok(Err(err.as_value().root(target))),
            }
        }
    }

    /// Allocate a new Julia array that clones its data from Rust without checking any invariants.
    ///
    /// The element type is `T`, the rank follows from the rank of `D`. If `N >= 0`, the rank of
    /// `D` must be equal to `N`. If an exception is thrown when the array is allocated, it is not
    /// caught. The size of the dimensions must be equal to the length of `data`.
    ///
    /// Note that the type of `data` is not `AsRef<[T]>` but `AsRef<[U]>`. The reason is that the
    /// type constructor can have more type parameters than its layout. `U` must implement
    /// `IsBits` and `ValidLayout`, and `T` must implement `HasLamut_yout<Layout = U>` to guarantee
    /// that `U` is a valid representation of instances of `T`.
    pub unsafe fn from_slice_copied_for_unchecked<'target, V, U, D, Tgt>(
        target: Tgt,
        ty: Value,
        data: V,
        dims: D,
    ) -> ArrayBaseData<'target, 'static, Tgt, Unknown, N>
    where
        Tgt: Target<'target>,
        D: DimsExt,
        U: ValidLayout + ValidField + IsBits + Copy,
        V: AsRef<[U]>,
    {
        let _ = DimsRankAssert::<D, N>::ASSERT_VALID_RANK;

        let data = data.as_ref();
        let len = data.len();

        let arr = Self::new_for_unchecked(&target, ty, dims);
        let array_data = jlrs_array_data(arr.as_managed().unwrap(Private));
        let array_data_slice = std::slice::from_raw_parts_mut(array_data as _, len);
        array_data_slice.copy_from_slice(data);

        arr.root(target)
    }
}

impl TypedVector<'_, '_, u8> {
    /// Convert a slice of bytes to a `TypedVector<u8>`.
    ///
    /// The bytes are copied from Rust to Julia. If an exception is thrown, it is caught and
    /// returned.
    pub fn from_bytes<'target, B, Tgt>(
        target: Tgt,
        bytes: B,
    ) -> ArrayBaseResult<'target, 'static, Tgt, u8, 1>
    where
        Tgt: Target<'target>,
        B: AsRef<[u8]>,
    {
        unsafe {
            let callback = || {
                let bytes = bytes.as_ref();
                jl_pchar_to_array(bytes.as_ptr() as *const _ as _, bytes.len())
            };

            let v = match catch_exceptions(callback, unwrap_exc) {
                Ok(arr) => Ok(NonNull::new_unchecked(arr)),
                Err(e) => Err(e),
            };

            target.result_from_ptr(v, Private)
        }
    }

    /// Convert a slice of bytes to a `TypedVector<u8>` without catching exceptions.
    ///
    /// The bytes are copied from Rust to Julia.
    ///
    /// Safety:
    ///
    /// If an exception is thrown, it is not caught.
    pub unsafe fn from_bytes_unchecked<'target, B, Tgt>(
        target: Tgt,
        bytes: B,
    ) -> ArrayBaseData<'target, 'static, Tgt, u8, 1>
    where
        Tgt: Target<'target>,
        B: AsRef<[u8]>,
    {
        let bytes = bytes.as_ref();
        let array = jl_pchar_to_array(bytes.as_ptr() as *const _ as _, bytes.len());
        target.data_from_ptr(NonNull::new_unchecked(array), Private)
    }

    /// Convert this array to a [`JuliaString`].
    pub fn to_jl_string<'target, Tgt>(self, target: Tgt) -> StringData<'target, Tgt>
    where
        Tgt: Target<'target>,
    {
        unsafe {
            let s = jl_array_to_string(self.unwrap(Private));
            let s = JuliaString::wrap_non_null(NonNull::new_unchecked(s.cast()), Private);
            s.root(target)
        }
    }
}

impl<'scope, 'data> VectorAny<'_, '_> {
    /// Allocate a new Julia array, the element type is the `Any` type and rank is 1.
    ///
    /// Examples
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # fn main() {
    /// # let mut julia = Builder::new().start_local().unwrap();
    /// julia.local_scope::<_, 2>(|mut frame| {
    ///     let array = VectorAny::new_any(&mut frame, 2);
    ///     assert!(array.is_ok());
    ///
    ///     let array = VectorAny::new_any(&mut frame, usize::MAX);
    ///     assert!(array.is_err());
    /// });
    /// # }
    /// ```
    pub fn new_any<'target, Tgt>(target: Tgt, size: usize) -> VectorAnyResult<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        unsafe {
            let callback = || jl_alloc_vec_any(size);

            let v = match catch_exceptions(callback, unwrap_exc) {
                Ok(arr) => Ok(NonNull::new_unchecked(arr)),
                Err(e) => Err(e),
            };
            target.result_from_ptr(v, Private)
        }
    }

    /// Allocate a new Julia array, the element type is the `Any` type and rank is 1 without
    /// checking any invariants.
    ///
    /// Safety: if an exception is thrown, it's not caught.
    pub unsafe fn new_any_unchecked<'target, Tgt>(
        target: Tgt,
        size: usize,
    ) -> VectorAnyData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        let arr = jl_alloc_vec_any(size);
        target.data_from_ptr(NonNull::new_unchecked(arr), Private)
    }
}

impl<T, const N: isize> ArrayBase<'_, '_, T, N> {
    /// Returns the rank of this array.
    pub fn rank(self) -> i32 {
        unsafe { jl_array_rank(self.unwrap(Private).cast()) }
    }

    // Returns `true` if `N != -1`.
    pub const fn has_rank(self) -> bool {
        N != -1
    }

    // Returns `true` if `N != -1`.
    pub const fn has_rank_s() -> bool {
        N != -1
    }
}

impl<const N: isize> ArrayBase<'_, '_, Unknown, N> {
    // Returns `false` because the the element type is `Unknown`.
    pub const fn has_constrained_type(self) -> bool {
        false
    }

    // Returns `false` because the the element type is `Unknown`.
    pub const fn has_constrained_type_s() -> bool {
        false
    }
}

impl<T: ConstructType, const N: isize> ArrayBase<'_, '_, T, N> {
    // Returns `true` because the the element type implements `ConstructType`.
    pub const fn has_constrained_type(self) -> bool {
        true
    }

    // Returns `true` because the the element type implements `ConstructType`.
    pub const fn has_constrained_type_s() -> bool {
        true
    }
}

// Fields and flags
impl<'scope, 'data, T, const N: isize> ArrayBase<'scope, 'data, T, N> {
    /// Returns the element size in bytes.
    pub fn element_size(self) -> usize {
        unsafe {
            let t = self.as_value().datatype().parameter_unchecked(0);

            if t.is::<DataType>() {
                t.cast_unchecked::<DataType>()
                    .size()
                    .map(|sz| sz as usize)
                    .unwrap_or(std::mem::size_of::<Value>())
            } else if t.is::<Union>() {
                let u = t.cast_unchecked::<Union>();

                let mut sz = 0;
                let mut align = 0;
                if u.isbits_size_align(&mut sz, &mut align) {
                    return sz;
                }

                std::mem::size_of::<Value>()
            } else {
                std::mem::size_of::<Value>()
            }
        }
    }

    /// Returns the element type.
    pub fn element_type(self) -> Value<'scope, 'static> {
        unsafe {
            Value::wrap_non_null(
                NonNull::new_unchecked(jl_array_eltype(self.unwrap(Private).cast()).cast()),
                Private,
            )
        }
    }

    /// Returns `true` if `L` is a valid layout for the element type.
    pub fn contains<L: ValidField>(self) -> bool {
        L::valid_field(self.element_type())
    }

    /// Returns the length of this array.
    pub fn length(self) -> usize {
        unsafe { jlrs_array_len(self.unwrap(Private)) }
    }

    /// Returns how the array has been allocated.
    pub fn how(self) -> How {
        let how = unsafe { jlrs_array_how(self.unwrap(Private)) };
        match how {
            0 => How::InlineOrForeign,
            1 => How::JuliaAllocated,
            2 => How::MallocAllocated,
            3 => How::PointerToOwner,
            _ => unreachable!(),
        }
    }

    /// Returns the number of dimensions (i.e. the rank) of this array.
    #[inline]
    pub fn n_dims(self) -> usize {
        if N != -1 {
            N as usize
        } else {
            unsafe { jlrs_array_ndims_fast(self.unwrap(Private)) }
        }
    }

    /// Returns the const parameter, `N`.
    #[inline]
    pub const fn generic_rank(self) -> isize {
        N
    }

    /// Returns `true` if the elements are stored as pointers, i.e. `Option<ValueRef>`.
    #[inline]
    pub fn ptr_array(self) -> bool {
        unsafe { jlrs_array_is_pointer_array(self.unwrap(Private)) != 0 }
    }

    /// Returns `true` if the elements are stored inline and contain references to managed data.
    #[inline]
    pub fn has_ptr(self) -> bool {
        unsafe { jlrs_array_has_pointers(self.unwrap(Private)) != 0 }
    }

    #[inline]
    pub fn union_array(self) -> bool {
        unsafe { jlrs_array_is_union_array(self.unwrap(Private)) != 0 }
    }

    /// Returns the dimensions of this array.
    #[inline]
    pub fn dimensions<'borrow>(&'borrow self) -> ArrayDimensions<'borrow, N> {
        unsafe {
            let ptr = jlrs_array_dims_ptr(self.unwrap(Private));
            let n = self.n_dims() as usize;
            let dims = std::slice::from_raw_parts(ptr.cast(), n);
            ArrayDimensions::new(dims)
        }
    }

    /// Returns a pointer to this array's data.
    #[inline]
    pub unsafe fn data_ptr(self) -> *mut c_void {
        jlrs_array_data(self.unwrap(Private))
    }

    /// Returns the owner of the array data.
    pub fn owner(self) -> Option<Value<'scope, 'data>> {
        if self.how() == How::PointerToOwner {
            unsafe {
                return Some(Value::wrap_non_null(
                    NonNull::new_unchecked(jlrs_array_data_owner(self.unwrap(Private))),
                    Private,
                ));
            }
        }
        None
    }

    /// Returns true if the elements are zero-initialized.
    pub fn zero_init(&self) -> bool {
        let ty = self.element_type();
        if ty.is::<DataType>() {
            unsafe {
                let ty = ty.cast_unchecked::<DataType>();
                ty.zero_init()
            }
        } else {
            true
        }
    }
}

// Tracking
impl<'scope, 'data, T, const N: isize> ArrayBase<'scope, 'data, T, N> {
    /// Track this array, allowing shared access.
    pub fn track_shared(self) -> JlrsResult<TrackedArrayBase<'scope, 'data, T, N>> {
        TrackedArrayBase::track_shared(self)
    }

    /// Track this array, enforcing exclusive access.
    pub fn track_exclusive(self) -> JlrsResult<TrackedArrayBaseMut<'scope, 'data, T, N>> {
        TrackedArrayBaseMut::track_exclusive(self)
    }
}

// Layout checks
impl<'scope, 'data, T, const N: isize> ArrayBase<'scope, 'data, T, N> {
    /// Returns `true` if the elements are stored inline and the element type is an isbits type.
    pub fn has_bits_layout(self) -> bool {
        self.has_inline_layout() && !self.has_ptr()
    }

    /// Returns `true` if the elements are stored inline.
    pub fn has_inline_layout(self) -> bool {
        !self.ptr_array() && !self.union_array()
    }

    /// Returns `true` if the elements are stored inline and the elements contain references to
    /// other Julia data.
    pub fn has_inline_with_refs_layout(self) -> bool {
        !self.ptr_array() && !self.has_union_layout() && self.has_ptr()
    }

    /// Returns `true` if the elements are stored inline and the element type is a union.
    pub fn has_union_layout(self) -> bool {
        self.union_array()
    }

    /// Returns `true` if the elements are stored as references to Julia data.
    pub fn has_value_layout(self) -> bool {
        self.ptr_array()
    }

    /// Returns `true` if the elements are stored as references to managed data.
    pub fn has_managed_layout<M: Managed<'scope, 'data> + Typecheck>(self) -> bool {
        if self.ptr_array() {
            let elty = self.element_type();
            if elty.is::<DataType>() {
                unsafe { elty.cast_unchecked::<DataType>().is::<M>() }
            } else {
                elty.is::<M>()
            }
        } else {
            false
        }
    }
}

// Accessors
impl<'scope, 'data, T, const N: isize> ArrayBase<'scope, 'data, T, N> {
    /// Create an accessor for `isbits` data.
    ///
    /// Thanks to the restrictions on `T` the data is guaranteed to be stored inline as an array
    /// of `T`s.
    ///
    /// Safety:
    ///
    /// No mutable accessors to this data must exist.
    pub unsafe fn bits_data<'borrow>(&'borrow self) -> BitsAccessor<'borrow, 'scope, 'data, T, T, N>
    where
        T: ConstructType + ValidField + IsBits,
    {
        // No need for checks, guaranteed to have isbits layout
        BitsAccessor::new(self)
    }

    /// Create an accessor for `isbits` data with layout `L`.
    ///
    /// Thanks to the restrictions on `T` and `L` the elements are guaranteed to be stored inline
    /// as an array of `L`s.
    ///
    /// Safety:
    ///
    /// No mutable accessors to this data must exist.
    pub unsafe fn bits_data_with_layout<'borrow, L>(
        &'borrow self,
    ) -> BitsAccessor<'borrow, 'scope, 'data, T, L, N>
    where
        T: ConstructType + HasLayout<'static, 'static, Layout = L>,
        L: IsBits + ValidField,
    {
        // No need for checks, guaranteed to have isbits layout and L is the layout of T
        BitsAccessor::new(self)
    }

    /// Try to create an accessor for `isbits` data with layout `L`.
    ///
    /// If the array doesn't have an isbits layout `ArrayLayoutError::NotBits` is returned. If `L`
    /// is not a valid field layout for the element type `TypeError::InvalidLayout` is returned.
    ///
    /// Safety:
    ///
    /// No mutable accessors to this data must exist.
    pub unsafe fn try_bits_data<'borrow, L>(
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

        Ok(BitsAccessor::new(self))
    }

    /// Create an accessor for `isbits` data with layout `L` without checking any invariants.
    ///
    /// Safety:
    ///
    /// No mutable accessors to this data must exist. The element type must be an isbits type, and
    /// `L` must be a valid field layout of the element type.
    #[inline]
    pub unsafe fn bits_data_unchecked<'borrow, L>(
        &'borrow self,
    ) -> BitsAccessor<'borrow, 'scope, 'data, T, L, N>
    where
        L: IsBits + ValidField,
    {
        BitsAccessor::new(self)
    }

    /// Create a mutable accessor for `isbits` data.
    ///
    /// Thanks to the restrictions on `T` the data is guaranteed to be stored inline as an array
    /// of `T`s.
    ///
    /// Safety:
    ///
    /// No other accessors to this data must exist.
    #[inline]
    pub unsafe fn bits_data_mut<'borrow>(
        &'borrow mut self,
    ) -> BitsAccessorMut<'borrow, 'scope, 'data, T, T, N>
    where
        T: ConstructType + ValidField + IsBits,
    {
        // No need for checks, guaranteed to have isbits layout
        BitsAccessorMut::new(self)
    }

    /// Create a mutable accessor for `isbits` data with layout `L`.
    ///
    /// Thanks to the restrictions on `T` and `L` the elements are guaranteed to be stored inline
    /// as an array of `L`s.
    ///
    /// Safety:
    ///
    /// No other accessors to this data must exist.
    pub unsafe fn bits_data_mut_with_layout<'borrow, L>(
        &'borrow mut self,
    ) -> BitsAccessorMut<'borrow, 'scope, 'data, T, L, N>
    where
        T: ConstructType + HasLayout<'static, 'static, Layout = L>,
        L: IsBits + ValidField,
    {
        // No need for checks, guaranteed to have isbits layout and L is the layout of T
        BitsAccessorMut::new(self)
    }

    /// Try to create a mutable accessor for `isbits` data with layout `L`.
    ///
    /// If the array doesn't have an isbits layout `ArrayLayoutError::NotBits` is returned. If `L`
    /// is not a valid field layout for the element type `TypeError::InvalidLayout` is returned.
    ///
    /// Safety:
    ///
    /// No other accessors to this data must exist.
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

        Ok(BitsAccessorMut::new(self))
    }

    /// Create a mutable accessor for `isbits` data with layout `L` without checking any
    /// invariants.
    ///
    /// Safety:
    ///
    /// No other accessors to this data must exist. The element type must be an isbits type, and
    /// `L` must be a valid field layout of the element type.
    #[inline]
    pub unsafe fn bits_data_mut_unchecked<'borrow, L>(
        &'borrow mut self,
    ) -> BitsAccessorMut<'borrow, 'scope, 'data, T, L, N>
    where
        L: IsBits + ValidField,
    {
        BitsAccessorMut::new(self)
    }

    /// Create an accessor for inline data.
    ///
    /// Thanks to the restrictions on `T` the data is guaranteed to be stored inline as an array
    /// of `T`s.
    ///
    /// Safety:
    ///
    /// No mutable accessors to this data must exist.
    #[inline]
    pub unsafe fn inline_data<'borrow>(
        &'borrow self,
    ) -> InlineAccessor<'borrow, 'scope, 'data, T, T, N>
    where
        T: ConstructType + ValidField,
    {
        // No need for checks, guaranteed to have inline layout
        InlineAccessor::new(self)
    }

    /// Create an accessor for inline data with layout `L`.
    ///
    /// Thanks to the restrictions on `T` and `L` the elements are guaranteed to be stored inline
    /// as an array of `L`s.
    ///
    /// Safety:
    ///
    /// No mutable accessors to this data must exist.
    #[inline]
    pub unsafe fn inline_data_with_layout<'borrow, L>(
        &'borrow self,
    ) -> InlineAccessor<'borrow, 'scope, 'data, T, L, N>
    where
        T: ConstructType + HasLayout<'scope, 'data, Layout = L>,
        L: ValidField,
    {
        // No need for checks, guaranteed to have inline layout and L is the layout of T
        InlineAccessor::new(self)
    }

    /// Try to create an accessor for inline data with layout `L`.
    ///
    /// If the array doesn't have an inline layout `ArrayLayoutError::NotInline` is returned. If
    /// `L` is not a valid field layout for the element type `TypeError::InvalidLayout` is
    /// returned.
    ///
    /// Safety:
    ///
    /// No mutable accessors to this data must exist.
    pub unsafe fn try_inline_data<'borrow, L>(
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

        Ok(InlineAccessor::new(self))
    }

    /// Create an accessor for inline data with layout `L` without checking any invariants.
    ///
    /// Safety:
    ///
    /// No mutable accessors to this data must exist. The elements must be stored inline, and `L`
    /// must be a valid field layout of the element type.
    #[inline]
    pub unsafe fn inline_data_unchecked<'borrow, L>(
        &'borrow self,
    ) -> InlineAccessor<'borrow, 'scope, 'data, T, L, N>
    where
        L: ValidField,
    {
        InlineAccessor::new(self)
    }

    /// Create a mutable accessor for inline data.
    ///
    /// Thanks to the restrictions on `T` the data is guaranteed to be stored inline as an array
    /// of `T`s.
    ///
    /// Safety:
    ///
    /// No other accessors to this data must exist.
    #[inline]
    pub unsafe fn inline_data_mut<'borrow>(
        &'borrow mut self,
    ) -> InlineAccessorMut<'borrow, 'scope, 'data, T, T, N>
    where
        T: ConstructType + ValidField,
    {
        // No need for checks, guaranteed to have inline layout
        InlineAccessorMut::new(self)
    }

    /// Create a mutable accessor for inline data with layout `L`.
    ///
    /// Thanks to the restrictions on `T` and `L` the elements are guaranteed to be stored inline
    /// as an array of `L`s.
    ///
    /// Safety:
    ///
    /// No other accessors to this data must exist.
    #[inline]
    pub unsafe fn inline_data_mut_with_layout<'borrow, L>(
        &'borrow mut self,
    ) -> InlineAccessorMut<'borrow, 'scope, 'data, T, L, N>
    where
        T: ConstructType + HasLayout<'scope, 'data, Layout = L>,
        L: ValidField,
    {
        // No need for checks, guaranteed to have inline layout and L is the layout of T
        InlineAccessorMut::new(self)
    }

    /// Try to create a mutable accessor for inline data with layout `L`.
    ///
    /// If the array doesn't have an inline layout `ArrayLayoutError::NotInline` is returned. If
    /// `L` is not a valid field layout for the element type `TypeError::InvalidLayout` is
    /// returned.
    ///
    /// Safety:
    ///
    /// No other accessors to this data must exist.
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

        Ok(InlineAccessorMut::new(self))
    }

    /// Create a mutable  accessor for inline data with layout `L` without checking any
    /// invariants.
    ///
    /// Safety:
    ///
    /// No other accessors to this data must exist. The elements must be stored inline, and `L`
    /// must be a valid field layout of the element type.
    #[inline]
    pub unsafe fn inline_data_mut_unchecked<'borrow, L>(
        &'borrow mut self,
    ) -> InlineAccessorMut<'borrow, 'scope, 'data, T, L, N>
    where
        L: ValidField,
    {
        InlineAccessorMut::new(self)
    }

    /// Create an accessor for unions of isbits types.
    ///
    /// This function panics if the array doesn't have a union layout.
    ///
    /// Safety:
    ///
    /// No mutable accessors to this data must exist.
    #[inline]
    pub unsafe fn union_data<'borrow>(
        &'borrow self,
    ) -> BitsUnionAccessor<'borrow, 'scope, 'data, T, N>
    where
        T: BitsUnionCtor,
    {
        assert!(
            self.has_union_layout(),
            "Array does not have a union layout"
        );
        BitsUnionAccessor::new(self)
    }

    /// Try to create an accessor for unions of isbits types.
    ///
    /// If the element type is not a union of isbits types `ArrayLayoutError::NotUnion` is
    /// returned.
    ///
    /// Safety:
    ///
    /// No mutable accessors to this data must exist.
    pub unsafe fn try_union_data<'borrow>(
        &'borrow self,
    ) -> JlrsResult<BitsUnionAccessor<'borrow, 'scope, 'data, T, N>> {
        if !self.has_union_layout() {
            Err(ArrayLayoutError::NotUnion {
                element_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        Ok(BitsUnionAccessor::new(self))
    }

    /// Create an accessor for unions of isbits types without checking any invariants.
    ///
    /// Safety:
    ///
    /// No mutable accessors to this data must exist. The element type must be a union of isbits
    /// types.
    #[inline]
    pub unsafe fn union_data_unchecked<'borrow>(
        &'borrow self,
    ) -> BitsUnionAccessor<'borrow, 'scope, 'data, T, N> {
        BitsUnionAccessor::new(self)
    }

    /// Create a mutable accessor for unions of isbits types.
    ///
    /// This function panics if the array doesn't have a union layout.
    ///
    /// Safety:
    ///
    /// No other accessors to this data must exist.
    #[inline]
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
        BitsUnionAccessorMut::new(self)
    }

    /// Try to create a mutable accessor for unions of isbits types.
    ///
    /// If the element type is not a union of isbits types `ArrayLayoutError::NotUnion` is
    /// returned.
    ///
    /// Safety:
    ///
    /// No other accessors to this data must exist.
    pub unsafe fn try_union_data_mut<'borrow>(
        &'borrow mut self,
    ) -> JlrsResult<BitsUnionAccessorMut<'borrow, 'scope, 'data, T, N>> {
        if !self.has_union_layout() {
            Err(ArrayLayoutError::NotUnion {
                element_type: self.element_type().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        Ok(BitsUnionAccessorMut::new(self))
    }

    /// Create a mutable accessor for unions of isbits types without checking any invariants.
    ///
    /// Safety:
    ///
    /// No other accessors to this data must exist. The element type must be a union of isbits
    /// types.
    #[inline]
    pub unsafe fn union_data_mut_unchecked<'borrow>(
        &'borrow mut self,
    ) -> BitsUnionAccessorMut<'borrow, 'scope, 'data, T, N> {
        BitsUnionAccessorMut::new(self)
    }

    /// Create an accessor for managed data.
    ///
    /// Thanks to the restrictions on `T` the data is guaranteed to be as an array of
    /// `Option<Ref<T>>`s.
    ///
    /// Safety:
    ///
    /// No mutable accessors to this data must exist.
    #[inline]
    pub unsafe fn managed_data<'borrow>(
        &'borrow self,
    ) -> ManagedAccessor<'borrow, 'scope, 'data, T, T, N>
    where
        T: Managed<'scope, 'data> + ConstructType,
    {
        // No need for checks, guaranteed to have correct layout
        ManagedAccessor::new(self)
    }

    /// Try to create an accessor for managed data of type `L`.
    ///
    /// If the element type is incompatible with `L` `ArrayLayoutError::NotManaged` is returned.
    ///
    /// Safety:
    ///
    /// No mutable accessors to this data must exist.
    pub unsafe fn try_managed_data<'borrow, L>(
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

        Ok(ManagedAccessor::new(self))
    }

    /// Create an accessor for managed data of type `L` without checking any invariants.
    ///
    /// Safety:
    ///
    /// No mutable accessors to this data must exist. The element type must be compatible with
    /// `L`.
    #[inline]
    pub unsafe fn managed_data_unchecked<'borrow, L>(
        &'borrow self,
    ) -> ManagedAccessor<'borrow, 'scope, 'data, T, L, N>
    where
        L: Managed<'scope, 'data>,
    {
        ManagedAccessor::new(self)
    }

    /// Create a mutable accessor for managed data.
    ///
    /// Thanks to the restrictions on `T` the data is guaranteed to be as an array of
    /// `Option<Ref<T>>`s.
    ///
    /// Safety:
    ///
    /// No other accessors to this data must exist.
    #[inline]
    pub unsafe fn managed_data_mut<'borrow>(
        &'borrow mut self,
    ) -> ManagedAccessorMut<'borrow, 'scope, 'data, T, T, N>
    where
        T: Managed<'scope, 'data> + ConstructType,
    {
        // No need for checks, guaranteed to have correct layout
        ManagedAccessorMut::new(self)
    }

    /// Try to create a mutable accessor for managed data of type `L`.
    ///
    /// If the element type is incompatible with `L` `ArrayLayoutError::NotManaged` is returned.
    ///
    /// Safety:
    ///
    /// No other accessors to this data must exist.
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

        Ok(ManagedAccessorMut::new(self))
    }

    /// Create a mutable accessor for managed data of type `L` without checking any invariants.
    ///
    /// Safety:
    ///
    /// No other accessors to this data must exist. The element type must be compatible with
    /// `L`.
    #[inline]
    pub unsafe fn managed_data_mut_unchecked<'borrow, L>(
        &'borrow mut self,
    ) -> ManagedAccessorMut<'borrow, 'scope, 'data, T, L, N>
    where
        L: Managed<'scope, 'data>,
    {
        ManagedAccessorMut::new(self)
    }

    /// Create an accessor for value data.
    ///
    /// Thanks to the restrictions on `T` the data is guaranteed to be as an array of
    /// `Option<Ref<Value>>`s.
    ///
    /// Safety:
    ///
    /// No mutable accessors to this data must exist.
    #[inline]
    pub unsafe fn value_data<'borrow>(&'borrow self) -> ValueAccessor<'borrow, 'scope, 'data, T, N>
    where
        T: Managed<'scope, 'data> + ConstructType,
    {
        // No need for checks, guaranteed to have inline layout
        ValueAccessor::new(self)
    }

    /// Try to create an accessor for value data.
    ///
    /// If the elements are stored inline `ArrayLayoutError::NotPointer` is returned.
    ///
    /// Safety:
    ///
    /// No mutable accessors to this data must exist.
    pub unsafe fn try_value_data<'borrow>(
        &'borrow self,
    ) -> JlrsResult<ValueAccessor<'borrow, 'scope, 'data, T, N>> {
        if !self.has_value_layout() {
            Err(ArrayLayoutError::NotPointer {
                element_type: self.element_type().error_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        Ok(ValueAccessor::new(self))
    }

    /// Create an accessor for managed data of type `L` without checking any invariants.
    ///
    /// Safety:
    ///
    /// No mutable accessors to this data must exist. The elements must not be stored inline.
    #[inline]
    pub unsafe fn value_data_unchecked<'borrow>(
        &'borrow self,
    ) -> ValueAccessor<'borrow, 'scope, 'data, T, N> {
        ValueAccessor::new(self)
    }

    /// Create a mutable accessor for value data.
    ///
    /// Thanks to the restrictions on `T` the data is guaranteed to be as an array of
    /// `Option<Ref<Value>>`s.
    ///
    /// Safety:
    ///
    /// No other accessors to this data must exist.
    #[inline]
    pub unsafe fn value_data_mut<'borrow>(
        &'borrow mut self,
    ) -> ValueAccessorMut<'borrow, 'scope, 'data, T, N>
    where
        T: Managed<'scope, 'data> + ConstructType,
    {
        // No need for checks, guaranteed to have inline layout
        ValueAccessorMut::new(self)
    }

    /// Try to create a mutable accessor for value data.
    ///
    /// If the elements are stored inline `ArrayLayoutError::NotPointer` is returned.
    ///
    /// Safety:
    ///
    /// No other accessors to this data must exist.
    pub unsafe fn try_value_data_mut<'borrow>(
        &'borrow mut self,
    ) -> JlrsResult<ValueAccessorMut<'borrow, 'scope, 'data, T, N>> {
        if !self.has_value_layout() {
            Err(ArrayLayoutError::NotPointer {
                element_type: self.element_type().error_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        Ok(ValueAccessorMut::new(self))
    }

    /// Create a mutable accessor for managed data of type `L` without checking any invariants.
    ///
    /// Safety:
    ///
    /// No other accessors to this data must exist. The elements must not be stored inline.
    #[inline]
    pub unsafe fn value_data_mut_unchecked<'borrow>(
        &'borrow mut self,
    ) -> ValueAccessorMut<'borrow, 'scope, 'data, T, N> {
        ValueAccessorMut::new(self)
    }

    /// Create an accessor for indeterminate data.
    ///
    /// Safety:
    ///
    /// No mutable accessors to this data must exist.
    #[inline]
    pub unsafe fn indeterminate_data<'borrow>(
        &'borrow self,
    ) -> IndeterminateAccessor<'borrow, 'scope, 'data, T, N> {
        IndeterminateAccessor::new(self)
    }

    /// Create a mutable accessor for indeterminate data.
    ///
    /// Safety:
    ///
    /// No other accessors to this data must exist.
    #[inline]
    pub unsafe fn indeterminate_data_mut<'borrow>(
        &'borrow mut self,
    ) -> IndeterminateAccessorMut<'borrow, 'scope, 'data, T, N> {
        IndeterminateAccessorMut::new(self)
    }
}

// Conversions
impl<'scope, 'data, T> ArrayBase<'scope, 'data, T, -1> {
    /// Sets the rank of this array to `N` if `N` is equal to the rank of `self` at runtime.
    pub fn set_rank<const N: isize>(self) -> JlrsResult<ArrayBase<'scope, 'data, T, N>> {
        if self.n_dims() as isize != N {
            Err(ArrayLayoutError::RankMismatch {
                found: self.n_dims() as _,
                provided: N,
            })?;
        }

        unsafe { Ok(self.set_rank_unchecked()) }
    }

    /// Sets the rank of this array to `N`.
    ///
    /// Safety:
    ///
    /// The rank at runtime must be equal to `N`.
    #[inline]
    pub unsafe fn set_rank_unchecked<const N: isize>(self) -> ArrayBase<'scope, 'data, T, N> {
        ArrayBase(
            self.unwrap_non_null(Private),
            PhantomData,
            PhantomData,
            PhantomData,
        )
    }
}

impl<'scope, 'data, const N: isize> ArrayBase<'scope, 'data, Unknown, N> {
    /// Sets the element type of this array to `T` if the cosntructed type of `T` is equal to the
    /// element type of `self` at runtime.
    pub fn set_type<T: ConstructType>(self) -> JlrsResult<ArrayBase<'scope, 'data, T, N>> {
        unsafe {
            let unrooted = Unrooted::new();

            let constructed = T::construct_type(unrooted).as_value();
            let elem_ty = self.element_type();
            if constructed != elem_ty {
                Err(TypeError::IncompatibleType {
                    element_type: constructed.display_string_or(CANNOT_DISPLAY_TYPE),
                    value_type: elem_ty.display_string_or(CANNOT_DISPLAY_TYPE),
                })?;
            }

            Ok(self.set_type_unchecked())
        }
    }

    /// Sets the element type of this array to `T`.
    ///
    /// Safety:
    ///
    /// The element at runtime must be equal to the constructed type of `T`.
    #[inline]
    pub unsafe fn set_type_unchecked<T: ConstructType>(self) -> ArrayBase<'scope, 'data, T, N> {
        ArrayBase(
            self.unwrap_non_null(Private),
            PhantomData,
            PhantomData,
            PhantomData,
        )
    }
}

impl<'scope, 'data, T, const N: isize> ArrayBase<'scope, 'data, T, N> {
    /// Forget the rank of this array.
    #[inline]
    pub fn forget_rank(self) -> ArrayBase<'scope, 'data, T, -1> {
        ArrayBase(
            self.unwrap_non_null(Private),
            PhantomData,
            PhantomData,
            PhantomData,
        )
    }

    /// Forget the element type of this array.
    #[inline]
    pub fn forget_type(self) -> ArrayBase<'scope, 'data, Unknown, N> {
        ArrayBase(
            self.unwrap_non_null(Private),
            PhantomData,
            PhantomData,
            PhantomData,
        )
    }

    /// Asserts that the rank is correct.
    #[inline]
    pub fn assert_rank(self) {
        if N == -1 {
            return;
        }

        let rank = self.rank();
        assert!(rank as isize == N);
    }
}

impl<'scope, 'data, T: ConstructType, const N: isize> ArrayBase<'scope, 'data, T, N> {
    /// Asserts that the element type of `self` is equal to the type constructed by `T`.
    ///
    /// Panics if the element type of `self`is not equal to the type constructed by `T`.
    pub fn assert_type(self) {
        unsafe {
            let unrooted = Unrooted::new();
            unrooted.local_scope::<_, 1>(|mut frame| {
                let ty = T::construct_type(&mut frame);
                assert_eq!(ty, self.element_type());
            });
        }
    }
}

/// Marker type used to indicate the element type of an array is unknown.
pub enum Unknown {}

/// `Array` or `ArrayRef`, depending on the target type `T`.
pub type ArrayBaseData<'target, 'data, Tgt, T, const N: isize> =
    <Tgt as TargetType<'target>>::Data<'data, ArrayBase<'target, 'data, T, N>>;

/// `JuliaResult<Array>` or `JuliaResultRef<ArrayRef>`, depending on the target type `T`.
pub type ArrayBaseResult<'target, 'data, Tgt, T, const N: isize> =
    TargetResult<'target, 'data, ArrayBase<'target, 'data, T, N>, Tgt>;

/// An array with an unknown element type and unknown rank.
pub type Array<'scope, 'data> = ArrayBase<'scope, 'data, Unknown, -1>;
pub type ArrayRef<'scope, 'data> = Ref<'scope, 'data, Array<'scope, 'data>>;
pub type ArrayRet = ArrayRef<'static, 'static>;
pub type ArrayData<'target, 'data, Tgt> =
    <Tgt as TargetType<'target>>::Data<'data, Array<'target, 'data>>;
pub type ArrayResult<'target, 'data, Tgt> =
    TargetResult<'target, 'data, Array<'target, 'data>, Tgt>;

/// An array with an unknown element type of rank 1.
pub type Vector<'scope, 'data> = ArrayBase<'scope, 'data, Unknown, 1>;
pub type VectorRef<'scope, 'data> = Ref<'scope, 'data, Vector<'scope, 'data>>;
pub type VectorRet = VectorRef<'static, 'static>;
pub type VectorData<'target, 'data, Tgt> =
    <Tgt as TargetType<'target>>::Data<'data, Vector<'target, 'data>>;
pub type VectorResult<'target, 'data, Tgt> =
    TargetResult<'target, 'data, Vector<'target, 'data>, Tgt>;

/// An array with an unknown element type of rank 1.
pub type VectorAny<'scope, 'data> = ArrayBase<'scope, 'data, Value<'scope, 'data>, 1>;
pub type VectorAnyRef<'scope, 'data> = Ref<'scope, 'data, VectorAny<'scope, 'data>>;
pub type VectorAnyRet = VectorAnyRef<'static, 'static>;
pub type VectorAnyData<'target, 'data, Tgt> =
    <Tgt as TargetType<'target>>::Data<'data, VectorAny<'target, 'data>>;
pub type VectorAnyResult<'target, 'data, Tgt> =
    TargetResult<'target, 'data, VectorAny<'target, 'data>, Tgt>;

/// An array with an unknown element type of rank 2.
pub type Matrix<'scope, 'data> = ArrayBase<'scope, 'data, Unknown, 2>;
pub type MatrixRef<'scope, 'data> = Ref<'scope, 'data, Matrix<'scope, 'data>>;
pub type MatrixRet = MatrixRef<'static, 'static>;
pub type MatrixData<'target, 'data, Tgt> =
    <Tgt as TargetType<'target>>::Data<'data, Matrix<'target, 'data>>;
pub type MatrixResult<'target, 'data, Tgt> =
    TargetResult<'target, 'data, Matrix<'target, 'data>, Tgt>;

/// An array with a known element type and unknown rank.
pub type TypedArray<'scope, 'data, T> = ArrayBase<'scope, 'data, T, -1>;
pub type TypedArrayRef<'scope, 'data, T> = Ref<'scope, 'data, TypedArray<'scope, 'data, T>>;
pub type TypedArrayRet<T> = TypedArrayRef<'static, 'static, T>;
pub type TypedArrayData<'target, 'data, Tgt, T> =
    <Tgt as TargetType<'target>>::Data<'data, TypedArray<'target, 'data, T>>;
pub type TypedArrayResult<'target, 'data, Tgt, T> =
    TargetResult<'target, 'data, TypedArray<'target, 'data, T>, Tgt>;

/// An array with a known element type of rank 1.
pub type TypedVector<'scope, 'data, T> = ArrayBase<'scope, 'data, T, 1>;
pub type TypedVectorRef<'scope, 'data, T> = Ref<'scope, 'data, TypedVector<'scope, 'data, T>>;
pub type TypedVectorRet<T> = TypedVectorRef<'static, 'static, T>;
pub type TypedVectorData<'target, 'data, Tgt, T> =
    <Tgt as TargetType<'target>>::Data<'data, TypedVector<'target, 'data, T>>;
pub type TypedVectorResult<'target, 'data, Tgt, T> =
    TargetResult<'target, 'data, TypedVector<'target, 'data, T>, Tgt>;

/// An array with a known element type of rank 2.
pub type TypedMatrix<'scope, 'data, T> = ArrayBase<'scope, 'data, T, 2>;
pub type TypedMatrixRef<'scope, 'data, T> = Ref<'scope, 'data, TypedMatrix<'scope, 'data, T>>;
pub type TypedMatrixRet<T> = TypedMatrixRef<'static, 'static, T>;
pub type TypedMatrixData<'target, 'data, Tgt, T> =
    <Tgt as TargetType<'target>>::Data<'data, TypedMatrix<'target, 'data, T>>;
pub type TypedMatrixResult<'target, 'data, Tgt, T> =
    TargetResult<'target, 'data, TypedMatrix<'target, 'data, T>, Tgt>;

/// An array with an unknown element type and known rank.
pub type RankedArray<'scope, 'data, const N: isize> = ArrayBase<'scope, 'data, Unknown, N>;
pub type RankedArrayRef<'scope, 'data, const N: isize> =
    Ref<'scope, 'data, RankedArray<'scope, 'data, N>>;
pub type RankedArrayRet<const N: isize> = RankedArrayRef<'static, 'static, N>;
pub type RankedArrayData<'target, 'data, Tgt, const N: isize> =
    <Tgt as TargetType<'target>>::Data<'data, RankedArray<'target, 'data, N>>;
pub type RankedArrayResult<'target, 'data, Tgt, const N: isize> =
    TargetResult<'target, 'data, RankedArray<'target, 'data, N>, Tgt>;

/// An array with a known element type and known rank.
pub type TypedRankedArray<'scope, 'data, T, const N: isize> = ArrayBase<'scope, 'data, T, N>;
pub type TypedRankedArrayRef<'scope, 'data, T, const N: isize> =
    Ref<'scope, 'data, TypedRankedArray<'scope, 'data, T, N>>;
pub type TypedRankedArrayRet<T, const N: isize> = TypedRankedArrayRef<'static, 'static, T, N>;
pub type TypedRankedArrayData<'target, 'data, Tgt, T, const N: isize> =
    <Tgt as TargetType<'target>>::Data<'data, TypedRankedArray<'target, 'data, T, N>>;
pub type TypedRankedArrayResult<'target, 'data, Tgt, T, const N: isize> =
    TargetResult<'target, 'data, TypedRankedArray<'target, 'data, T, N>, Tgt>;

unsafe impl<'scope, 'data, const N: isize> Typecheck for ArrayBase<'scope, 'data, Unknown, N> {
    fn typecheck(ty: DataType) -> bool {
        let unrooted = ty.unrooted_target();

        // Datatype must be an array type
        if ty.type_name().unwrap(Private) != TypeName::of_array(&unrooted).unwrap(Private) {
            return false;
        }

        if N >= 0 {
            // Casting to RankedArray, check if the rank is correct
            unsafe {
                let param = ty.parameter_unchecked(1);

                if !param.is::<isize>() || param.unbox_unchecked::<isize>() != N {
                    return false;
                }
            }
        }

        true
    }
}

unsafe impl<'scope, 'data, T: ConstructType, const N: isize> Typecheck
    for ArrayBase<'scope, 'data, T, N>
{
    fn typecheck(ty: DataType) -> bool {
        let unrooted = ty.unrooted_target();

        // Datatype must be an array type
        if ty.type_name().unwrap(Private) != TypeName::of_array(&unrooted).unwrap(Private) {
            return false;
        }

        if N >= 0 {
            // Casting to RankedArray, check if the rank is correct
            unsafe {
                let param = ty.parameter_unchecked(1);

                if !param.is::<isize>() || param.unbox_unchecked::<isize>() != N {
                    return false;
                }
            }
        }

        unrooted.local_scope::<_, 1>(|mut frame| {
            // Safety: elem_ty is reachable from ty
            let elem_ty = unsafe { ty.parameter_unchecked(0) };
            let constructed_ty = T::construct_type(&mut frame);
            if elem_ty.is::<TypeVar>() && constructed_ty.is::<TypeVar>() {
                unsafe {
                    let et = elem_ty.cast_unchecked::<TypeVar>();
                    let ct = constructed_ty.cast_unchecked::<TypeVar>();
                    return et.name() == ct.name()
                        && et.lower_bound(&frame).as_value() == ct.lower_bound(&frame).as_value()
                        && et.upper_bound(&frame).as_value() == ct.upper_bound(&frame).as_value();
                }
            }
            elem_ty == constructed_ty
        })
    }
}

impl<T, const N: isize> Debug for ArrayBase<'_, '_, T, N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self.display_string() {
            Ok(s) => write!(f, "{}", s),
            Err(e) => write!(f, "<Cannot display value: {}>", e),
        }
    }
}

impl<'scope, 'data, T, const N: isize> ManagedPriv<'scope, 'data>
    for ArrayBase<'scope, 'data, T, N>
{
    type Wraps = jl_array_t;

    type WithLifetimes<'target, 'da> = ArrayBase<'target, 'da, T, N>;

    const NAME: &'static str = "Array";

    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: crate::private::Private) -> Self {
        ArrayBase(inner, PhantomData, PhantomData, PhantomData)
    }

    fn unwrap_non_null(self, _: crate::private::Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

unsafe impl<const N: isize> ValidField for Option<RankedArrayRef<'_, '_, N>> {
    fn valid_field(v: Value) -> bool {
        if v.is::<DataType>() {
            let dt = unsafe { v.cast_unchecked::<DataType>() };
            let is_array = dt.is::<Array>();

            if !is_array {
                return false;
            }

            let parameters = dt.parameters();
            let parameters = parameters.data();
            if N != -1 {
                unsafe {
                    let unrooted = Unrooted::new();
                    let rank_param = parameters.get(unrooted, 1).unwrap_unchecked().as_value();
                    if !rank_param.is::<isize>() || rank_param.unbox_unchecked::<isize>() != N {
                        return false;
                    }
                }
            }

            true
        } else if v.is::<UnionAll>() {
            let ua = unsafe { v.cast_unchecked::<UnionAll>() };
            let dt = ua.base_type();

            if !dt.is::<Array>() {
                return false;
            }
            let parameters = dt.parameters();
            let parameters = parameters.data();
            if N != -1 {
                unsafe {
                    let unrooted = Unrooted::new();
                    let rank_param = parameters.get(unrooted, 1).unwrap_unchecked().as_value();
                    if !rank_param.is::<isize>() || rank_param.unbox_unchecked::<isize>() != N {
                        return false;
                    }
                }
            }

            true
        } else {
            false
        }
    }
}

unsafe impl<T: ConstructType, const N: isize> ValidField
    for Option<TypedRankedArrayRef<'_, '_, T, N>>
{
    fn valid_field(v: Value) -> bool {
        if v.is::<DataType>() {
            let dt = unsafe { v.cast_unchecked::<DataType>() };
            if !dt.is::<Array>() {
                return false;
            }

            let parameters = dt.parameters();
            let parameters = parameters.data();
            if N != -1 {
                unsafe {
                    let unrooted = Unrooted::new();
                    let rank_param = parameters.get(unrooted, 1).unwrap_unchecked().as_value();
                    if !rank_param.is::<isize>() || rank_param.unbox_unchecked::<isize>() != N {
                        return false;
                    }
                }
            }

            unsafe {
                let unrooted = Unrooted::new();
                unrooted.local_scope::<_, 1>(|mut frame| {
                    let ty = T::construct_type(&mut frame);
                    let elem_ty = parameters.get(unrooted, 0).unwrap_unchecked().as_value();
                    ty == elem_ty
                })
            }
        } else {
            false
        }
    }
}

unsafe impl<'scope, 'data, T: ConstructType, const N: isize> ConstructType
    for TypedRankedArray<'scope, 'data, T, N>
{
    type Static = TypedRankedArray<'static, 'static, T::Static, N>;

    fn construct_type_uncached<'target, Tgt>(
        target: Tgt,
    ) -> crate::prelude::ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        let ty = UnionAll::array_type(&target);

        if N == -1 {
            target.with_local_scope::<_, _, 2>(|target, mut frame| unsafe {
                let elty = T::construct_type(&mut frame);
                let tn_n = ty.body().cast_unchecked::<UnionAll>().var();
                let applied = ty.apply_types_unchecked(&mut frame, [elty, tn_n.as_value()]);

                UnionAll::rewrap(target, applied.cast_unchecked::<DataType>())
            })
        } else {
            target.with_local_scope::<_, _, 3>(|target, mut frame| unsafe {
                let elty = T::construct_type(&mut frame);
                let n = Value::new(&mut frame, N);
                let applied = ty.apply_types_unchecked(&mut frame, [elty, n]);

                UnionAll::rewrap(target, applied.cast_unchecked::<DataType>())
            })
        }
    }

    fn base_type<'target, Tgt>(target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        Some(UnionAll::array_type(target).as_value())
    }

    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        env: &crate::data::types::construct_type::TypeVarEnv,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        let ty = UnionAll::array_type(&target);

        if N == -1 {
            let n_sym = NSym::get_symbol(&target);
            let n_param = match env.get(n_sym) {
                Some(n_param) => n_param.as_value(),
                _ => ty.base_type().parameter(1).unwrap(),
            };

            target.with_local_scope::<_, _, 2>(|target, mut frame| unsafe {
                let t = T::construct_type_with_env(&mut frame, env);
                let applied = ty.apply_types_unchecked(&mut frame, [t, n_param]);
                assert!(applied.is::<DataType>());
                applied
                    .cast_unchecked::<DataType>()
                    .wrap_with_env(target, env)
            })
        } else {
            target.with_local_scope::<_, _, 3>(|target, mut frame| unsafe {
                let t = T::construct_type_with_env(&mut frame, env);
                let n = Value::new(&mut frame, N);
                let applied = ty.apply_types_unchecked(&mut frame, [t, n]);
                assert!(applied.is::<DataType>());
                applied
                    .cast_unchecked::<DataType>()
                    .wrap_with_env(target, env)
            })
        }
    }
}

unsafe impl<'scope, 'data, const N: isize> ConstructType for RankedArray<'scope, 'data, N> {
    type Static = RankedArray<'static, 'static, N>;

    fn construct_type_uncached<'target, Tgt>(
        target: Tgt,
    ) -> crate::prelude::ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        let ty = UnionAll::array_type(&target);

        if N == -1 {
            ty.as_value().root(target)
        } else {
            target.with_local_scope::<_, _, 3>(|target, mut frame| unsafe {
                let tn_t = TypeVar::new_unchecked(&mut frame, "T", None, None);
                let n = Value::new(&mut frame, N);
                let applied = ty.apply_types_unchecked(&mut frame, [tn_t.as_value(), n]);

                UnionAll::rewrap(target, applied.cast_unchecked::<DataType>())
            })
        }
    }

    #[inline]
    fn base_type<'target, Tgt>(target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        Some(UnionAll::array_type(target).as_value())
    }

    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        env: &crate::data::types::construct_type::TypeVarEnv,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        let ty = UnionAll::array_type(&target);
        let t_sym = TSym::get_symbol(&target);
        let t_param = env.get(t_sym).expect("TypeVar T is not in env");

        if N == -1 {
            let n_sym = NSym::get_symbol(&target);
            let n_param = match env.get(n_sym) {
                Some(n_param) => n_param.as_value(),
                _ => unsafe {
                    ty.body()
                        .cast_unchecked::<UnionAll>()
                        .body()
                        .cast_unchecked::<DataType>()
                        .parameter(1)
                        .unwrap()
                },
            };

            unsafe { ty.apply_types_unchecked(target, [t_param.as_value(), n_param]) }
        } else {
            target.with_local_scope::<_, _, 1>(|target, mut frame| unsafe {
                let n = Value::new(&mut frame, N);
                let applied = ty.apply_types_unchecked(target, [t_param.as_value(), n]);
                applied
            })
        }
    }
}

unsafe impl<'scope, 'data, const N: isize> CCallArg for RankedArray<'scope, 'data, N> {
    type CCallArgType = AnyType;
    type FunctionArgType = Self;
}

unsafe impl<const N: isize> CCallReturn for RankedArrayRet<N> {
    type CCallReturnType = AnyType;
    type FunctionReturnType = RankedArray<'static, 'static, N>;
    type ReturnAs = Self;

    #[inline]
    unsafe fn return_or_throw(self) -> Self::ReturnAs {
        self
    }
}

unsafe impl<'scope, 'data, T: ConstructType, const N: isize> CCallArg
    for TypedRankedArray<'scope, 'data, T, N>
{
    type CCallArgType = IfConcreteElse<Self, AnyType>;
    type FunctionArgType = Self;
}

unsafe impl<T: ConstructType, const N: isize> CCallReturn for TypedRankedArrayRet<T, N> {
    type CCallReturnType = IfConcreteElse<TypedRankedArray<'static, 'static, T, N>, AnyType>;
    type FunctionReturnType = TypedRankedArray<'static, 'static, T, N>;
    type ReturnAs = Self;

    #[inline]
    unsafe fn return_or_throw(self) -> Self::ReturnAs {
        self
    }
}

#[inline]
pub(crate) fn sized_dim_tuple<'target, D, Tgt>(
    target: Tgt,
    dims: &D,
) -> ValueData<'target, 'static, Tgt>
where
    D: RankedDims,
    Tgt: Target<'target>,
{
    unsafe {
        let dims_type = dims.dimension_object(&target).as_managed();
        let tuple = jl_new_struct_uninit(dims_type.unwrap(Private));

        {
            let slice =
                std::slice::from_raw_parts_mut(tuple as *mut MaybeUninit<usize>, D::RANK as _);
            dims.fill_tuple(slice, Private);
        }

        Value::wrap_non_null(NonNull::new_unchecked(tuple), Private).root(target)
    }
}

#[inline]
pub(crate) fn unsized_dim_tuple<'target, D, Tgt>(
    target: Tgt,
    dims: &D,
) -> ValueData<'target, 'static, Tgt>
where
    D: DimsExt,
    Tgt: Target<'target>,
{
    unsafe {
        let dims_type = dims.dimension_object(&target).as_managed();
        let tuple = jl_new_struct_uninit(dims_type.unwrap(Private));

        {
            let slice =
                std::slice::from_raw_parts_mut(tuple as *mut MaybeUninit<usize>, dims.rank());
            dims.fill_tuple(slice, Private);
        }

        Value::wrap_non_null(NonNull::new_unchecked(tuple), Private).root(target)
    }
}

// Safety: must be used as a finalizer when moving array data from Rust to Julia
// to ensure it's freed correctly.
#[julia_version(until = "1.10")]
unsafe extern "C" fn droparray<T>(a: Array) {
    let sz = a.dimensions().size();
    let data_ptr = a.data_ptr().cast::<T>();

    let data = Vec::from_raw_parts(data_ptr, sz, sz);
    std::mem::drop(data);
}

#[julia_version(since = "1.11")]
unsafe extern "C" fn droparray<T>(a: *mut c_void) {
    #[repr(C)]
    struct GenericMemory<T> {
        length: usize,
        ptr: *mut T,
    }

    let a = NonNull::new_unchecked(a as *mut GenericMemory<T>).as_mut();
    let v = Vec::from_raw_parts(a.ptr as *mut T, a.length, a.length);
    a.ptr = null_mut();
    a.length = 0;
    std::mem::drop(v);
}
