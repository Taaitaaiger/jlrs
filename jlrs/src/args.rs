//! Julia function arguments
//!
//! Calling some functions requires adding one or more additional arguments before the provided
//! arguments. Functions that require this take the arguments as an implementation of [`Values`],
//! which can add these extra arguments without requiring any heap allocations as long as the
//! number of arguments is known at compile-time.

use self::private::ValuesPriv;
use crate::data::managed::value::Value;

const MAX_SIZE: usize = 8;
const UMAX: usize = usize::MAX;

/// A number of `Value`s.
///
/// This trait is implemented for sized and unsized arrays and array slices, if the number of
/// `Value`s is indeterminate `N` is `usize::MAX`. In this case allocating may be required to
/// add additional values so you should always prefer using constantly-sized array.
pub trait Values<'scope, 'data, const N: usize>: ValuesPriv<'scope, 'data, N> {}

impl<'scope, 'data, const N: usize> Values<'scope, 'data, N> for &[Value<'scope, 'data>; N] {}
impl<'scope, 'data, const N: usize> Values<'scope, 'data, N> for &mut [Value<'scope, 'data>; N] {}
impl<'scope, 'data, const N: usize> Values<'scope, 'data, N> for [Value<'scope, 'data>; N] {}

impl<'scope, 'data> Values<'scope, 'data, UMAX> for &[Value<'scope, 'data>] {}
impl<'scope, 'data> Values<'scope, 'data, UMAX> for &mut [Value<'scope, 'data>] {}
impl<'scope, 'data, 'borrow, const SIZE: usize> Values<'scope, 'data, UMAX>
    for WithSmallVecSize<'scope, 'data, 'borrow, SIZE>
{
}

/// Use a custom size for the internal `SmallVec` when extra items are added.
///
/// When a slice of `Value`s is used as `Values`, the internal `SmallVec` has capacity for 8
/// values. By using `WithSmallVecSize` you can use a custom size.
#[repr(transparent)]
pub struct WithSmallVecSize<'scope, 'data, 'borrow, const SIZE: usize>(
    &'borrow [Value<'scope, 'data>],
);

/// Convert a slice of `Value`s to `WithSmallVecSize`.
#[inline]
pub fn with_small_vec_size<'scope, 'data, 'borrow, const SIZE: usize>(
    values: &'borrow [Value<'scope, 'data>],
) -> WithSmallVecSize<'scope, 'data, 'borrow, SIZE> {
    WithSmallVecSize(values)
}

pub(crate) mod private {
    use std::slice;

    use jl_sys::jl_value_t;
    use smallvec::SmallVec;

    use super::{WithSmallVecSize, MAX_SIZE, UMAX};
    use crate::{
        data::managed::{private::ManagedPriv, value::Value},
        private::Private,
    };

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct ExtendedArray<T: Copy, const N: usize, const M: usize> {
        a: [T; N],
        b: [T; M],
    }

    impl<T: Copy, const N: usize, const M: usize> ExtendedArray<T, N, M> {
        #[inline]
        fn new(a: [T; N], b: [T; M]) -> Self {
            ExtendedArray { a, b }
        }
    }

    impl<T: Copy, const N: usize, const M: usize> AsRef<[T]> for ExtendedArray<T, N, M> {
        #[inline]
        fn as_ref(&self) -> &[T] {
            unsafe { slice::from_raw_parts(self as *const _ as *const _, N + M) }
        }
    }

    pub trait ValuesPriv<'scope, 'data, const N: usize> {
        type ExtendedPointers<const A: usize, const B: usize>: AsRef<[*mut jl_value_t]>;

        type Extended<const A: usize, const B: usize>: AsRef<[Value<'scope, 'data>]>;

        fn as_slice(&self, _: Private) -> &[Value<'scope, 'data>];

        fn into_extended_with_start<const A: usize>(
            self,
            start: [Value<'scope, 'data>; A],
            _: Private,
        ) -> Self::Extended<A, N>;

        fn as_pointers(&self, _: Private) -> &[*mut jl_value_t];

        fn into_extended_pointers_with_start<const A: usize>(
            self,
            start: [*mut jl_value_t; A],
            _: Private,
        ) -> Self::ExtendedPointers<A, N>;
    }

    impl<'scope, 'data> ValuesPriv<'scope, 'data, UMAX> for &[Value<'scope, 'data>] {
        type ExtendedPointers<const A: usize, const B: usize> =
            SmallVec<[*mut jl_value_t; MAX_SIZE]>;

        type Extended<const A: usize, const B: usize> = SmallVec<[Value<'scope, 'data>; MAX_SIZE]>;

        #[inline]
        fn as_slice(&self, _: Private) -> &[Value<'scope, 'data>] {
            self
        }

        #[inline]
        fn into_extended_with_start<const N2: usize>(
            self,
            start: [Value<'scope, 'data>; N2],
            _: Private,
        ) -> Self::Extended<N2, UMAX> {
            let mut sv: SmallVec<[Value<'scope, 'data>; MAX_SIZE]> = SmallVec::from_slice(&start);
            sv.extend(self.iter().copied());
            sv
        }

        #[inline]
        fn as_pointers(&self, _: Private) -> &[*mut jl_value_t] {
            let ptr = self.as_ptr();
            unsafe { slice::from_raw_parts(ptr.cast(), self.len()) }
        }

        #[inline]
        fn into_extended_pointers_with_start<const A: usize>(
            self,
            start: [*mut jl_value_t; A],
            _: Private,
        ) -> Self::ExtendedPointers<A, UMAX> {
            let mut sv: SmallVec<[*mut jl_value_t; MAX_SIZE]> = SmallVec::from_slice(&start);
            sv.extend(self.as_pointers(Private).into_iter().copied());
            sv
        }
    }

    impl<'scope, 'data, 'borrow, const SIZE: usize> ValuesPriv<'scope, 'data, UMAX>
        for WithSmallVecSize<'scope, 'data, 'borrow, SIZE>
    {
        type ExtendedPointers<const A: usize, const B: usize> = SmallVec<[*mut jl_value_t; SIZE]>;

        type Extended<const A: usize, const B: usize> = SmallVec<[Value<'scope, 'data>; SIZE]>;

        #[inline]
        fn as_slice(&self, _: Private) -> &[Value<'scope, 'data>] {
            self.0
        }

        #[inline]
        fn into_extended_with_start<const N2: usize>(
            self,
            start: [Value<'scope, 'data>; N2],
            _: Private,
        ) -> Self::Extended<N2, UMAX> {
            let mut sv: SmallVec<[Value<'scope, 'data>; SIZE]> = SmallVec::from_slice(&start);
            sv.extend(self.0.iter().copied());
            sv
        }

        #[inline]
        fn as_pointers(&self, _: Private) -> &[*mut jl_value_t] {
            let ptr = self.0.as_ptr();
            unsafe { slice::from_raw_parts(ptr.cast(), self.0.len()) }
        }

        #[inline]
        fn into_extended_pointers_with_start<const A: usize>(
            self,
            start: [*mut jl_value_t; A],
            _: Private,
        ) -> Self::ExtendedPointers<A, UMAX> {
            let mut sv: SmallVec<[*mut jl_value_t; SIZE]> = SmallVec::from_slice(&start);
            sv.extend(self.as_pointers(Private).into_iter().copied());
            sv
        }
    }

    impl<'scope, 'data> ValuesPriv<'scope, 'data, UMAX> for &mut [Value<'scope, 'data>] {
        type ExtendedPointers<const A: usize, const B: usize> =
            SmallVec<[*mut jl_value_t; MAX_SIZE]>;

        type Extended<const A: usize, const B: usize> = SmallVec<[Value<'scope, 'data>; MAX_SIZE]>;

        #[inline]
        fn as_slice(&self, _: Private) -> &[Value<'scope, 'data>] {
            self
        }

        #[inline]
        fn into_extended_with_start<const N2: usize>(
            self,
            start: [Value<'scope, 'data>; N2],
            _: Private,
        ) -> Self::Extended<N2, UMAX> {
            let mut sv: SmallVec<[Value<'scope, 'data>; MAX_SIZE]> = SmallVec::from_slice(&start);
            sv.extend(self.iter().copied());
            sv
        }

        #[inline]
        fn as_pointers(&self, _: Private) -> &[*mut jl_value_t] {
            let ptr = self.as_ptr();
            unsafe { slice::from_raw_parts(ptr.cast(), self.len()) }
        }

        #[inline]
        fn into_extended_pointers_with_start<const A: usize>(
            self,
            start: [*mut jl_value_t; A],
            _: Private,
        ) -> Self::ExtendedPointers<A, UMAX> {
            let mut sv: SmallVec<[*mut jl_value_t; MAX_SIZE]> = SmallVec::from_slice(&start);
            sv.extend(self.as_pointers(Private).into_iter().copied());
            sv
        }
    }

    impl<'scope, 'data, const N: usize> ValuesPriv<'scope, 'data, N> for [Value<'scope, 'data>; N] {
        type ExtendedPointers<const A: usize, const B: usize> =
            ExtendedArray<*mut jl_value_t, A, B>;

        type Extended<const A: usize, const B: usize> = ExtendedArray<Value<'scope, 'data>, A, B>;

        #[inline]
        fn as_slice(&self, _: Private) -> &[Value<'scope, 'data>] {
            &self[..]
        }

        #[inline]
        fn into_extended_with_start<const A: usize>(
            self,
            start: [Value<'scope, 'data>; A],
            _: Private,
        ) -> Self::Extended<A, N> {
            Self::Extended::<A, N>::new(start, self)
        }

        #[inline]
        fn as_pointers(&self, _: Private) -> &[*mut jl_value_t] {
            unsafe { std::mem::transmute(&self[..]) }
        }

        #[inline]
        fn into_extended_pointers_with_start<const A: usize>(
            self,
            start: [*mut jl_value_t; A],
            _: Private,
        ) -> Self::ExtendedPointers<A, N> {
            Self::ExtendedPointers::<A, N>::new(start, self.map(|x| x.unwrap(Private)))
        }
    }

    impl<'scope, 'data, const N: usize> ValuesPriv<'scope, 'data, N> for &[Value<'scope, 'data>; N] {
        type ExtendedPointers<const A: usize, const B: usize> =
            ExtendedArray<*mut jl_value_t, A, B>;

        type Extended<const A: usize, const B: usize> = ExtendedArray<Value<'scope, 'data>, A, B>;

        #[inline]
        fn as_slice(&self, _: Private) -> &[Value<'scope, 'data>] {
            &self[..]
        }

        #[inline]
        fn into_extended_with_start<const A: usize>(
            self,
            start: [Value<'scope, 'data>; A],
            _: Private,
        ) -> Self::Extended<A, N> {
            Self::Extended::<A, N>::new(start, *self)
        }

        #[inline]
        fn as_pointers(&self, _: Private) -> &[*mut jl_value_t] {
            unsafe { std::mem::transmute(&self[..]) }
        }

        #[inline]
        fn into_extended_pointers_with_start<const A: usize>(
            self,
            start: [*mut jl_value_t; A],
            _: Private,
        ) -> Self::ExtendedPointers<A, N> {
            Self::ExtendedPointers::<A, N>::new(start, self.map(|x| x.unwrap(Private)))
        }
    }

    impl<'scope, 'data, const N: usize> ValuesPriv<'scope, 'data, N>
        for &mut [Value<'scope, 'data>; N]
    {
        type ExtendedPointers<const A: usize, const B: usize> =
            ExtendedArray<*mut jl_value_t, A, B>;

        type Extended<const A: usize, const B: usize> = ExtendedArray<Value<'scope, 'data>, A, B>;

        #[inline]
        fn as_slice(&self, _: Private) -> &[Value<'scope, 'data>] {
            &self[..]
        }

        #[inline]
        fn into_extended_with_start<const A: usize>(
            self,
            start: [Value<'scope, 'data>; A],
            _: Private,
        ) -> Self::Extended<A, N> {
            Self::Extended::<A, N>::new(start, *self)
        }

        #[inline]
        fn as_pointers(&self, _: Private) -> &[*mut jl_value_t] {
            unsafe { std::mem::transmute(&self[..]) }
        }

        #[inline]
        fn into_extended_pointers_with_start<const A: usize>(
            self,
            start: [*mut jl_value_t; A],
            _: Private,
        ) -> Self::ExtendedPointers<A, N> {
            Self::ExtendedPointers::<A, N>::new(start, self.map(|x| x.unwrap(Private)))
        }
    }
}
