//! Convert data to another type with the same layout.
//!
//! Crates that use jlrs can implement traits like `ValidLayout`, but they can only be implemented
//! for types defined in that crate. It's not possible to implement that trait for, say,
//! `num_complex::Complex` outside of jlrs and num_complex due to the orphan rule. This can be
//! problematic, because `num_complex::Complex` and `Complex` in Julia have exactly the same
//! layout and crates like `rustfft` provide functions that take `num_complex::Complex`.
//!
//! In order to work around this limitation without having to copy data, jlrs lets you declare
//! that two types are compatible by implementing the [`Compatible`] trait. This trait may only
//! be implemented if the types have exactly the same layout.
//!
//! If `T` implements `Compatible<U>`, you can convert references to and slices of `T` to
//! references to and slices of `U` by calling [`CompatibleCast::compatible_cast`] and
//! [`CompatibleCast::compatible_cast_mut`].

use self::private::CompatibleCastPriv;
use crate::data::layout::valid_layout::ValidLayout;

/// Marker trait that indicates `Self` and `U` have the same layout.
///
/// If a type implements `Compatible<U>`, it's possible to convert references to and slices of
/// `Self` to references to and slices of `U` using the methods of the [`CompatibleCast`] trait.
///
/// Safety: `Self` and `U` must have the same layout and must both be `#[repr(C)]`.
pub unsafe trait Compatible<U>: ValidLayout {}

// Types are compatible with themselves.
unsafe impl<T: ValidLayout> Compatible<T> for T {}

/// Cast data to a compatible type.
///
/// If `T` implements [`ValidLayout`] this trait is automatically implemented for `T`, `[T]`, and
/// `[T; N]`. The methods of this traits substitute `T` with `U` and can only be called if
/// `T: Compatible<U>` which guarantees this conversion is valid.
pub trait CompatibleCast: CompatibleCastPriv {
    type Inner;
    type Output<U: Sized>: ?Sized;

    /// Converts `&Self` to `&U`, `&[Self]` to `&[U]`, and `&[Self; N]` to `&[U; N]`.
    fn compatible_cast<U>(&self) -> &Self::Output<U>
    where
        Self::Inner: Compatible<U>;

    /// Converts `&mut Self` to `&mut U`, `&mut [Self]` to `&mut [U]`, and `&mut [Self; N]` to
    /// `&mut [U; N]`.
    fn compatible_cast_mut<U>(&mut self) -> &mut Self::Output<U>
    where
        Self::Inner: Compatible<U>;
}

impl<T: ValidLayout> CompatibleCast for T {
    type Inner = T;
    type Output<U: Sized> = U;

    #[inline]
    fn compatible_cast<U>(&self) -> &Self::Output<U>
    where
        T: Compatible<U>,
    {
        debug_assert_eq!(std::mem::size_of::<T>(), std::mem::size_of::<U>());
        debug_assert_eq!(std::mem::align_of::<T>(), std::mem::align_of::<U>());
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    fn compatible_cast_mut<U>(&mut self) -> &mut Self::Output<U>
    where
        T: Compatible<U>,
    {
        debug_assert_eq!(std::mem::size_of::<T>(), std::mem::size_of::<U>());
        debug_assert_eq!(std::mem::align_of::<T>(), std::mem::align_of::<U>());
        unsafe { std::mem::transmute(self) }
    }
}

impl<T: ValidLayout> CompatibleCast for [T] {
    type Inner = T;
    type Output<U: Sized> = [U];

    #[inline]
    fn compatible_cast<U>(&self) -> &Self::Output<U>
    where
        T: Compatible<U>,
    {
        debug_assert_eq!(std::mem::size_of::<T>(), std::mem::size_of::<U>());
        debug_assert_eq!(std::mem::align_of::<T>(), std::mem::align_of::<U>());
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    fn compatible_cast_mut<U>(&mut self) -> &mut Self::Output<U>
    where
        T: Compatible<U>,
    {
        debug_assert_eq!(std::mem::size_of::<T>(), std::mem::size_of::<U>());
        debug_assert_eq!(std::mem::align_of::<T>(), std::mem::align_of::<U>());
        unsafe { std::mem::transmute(self) }
    }
}

impl<T: ValidLayout, const N: usize> CompatibleCast for [T; N] {
    type Inner = T;
    type Output<U: Sized> = [U; N];

    #[inline]
    fn compatible_cast<U>(&self) -> &Self::Output<U>
    where
        T: Compatible<U>,
    {
        debug_assert_eq!(std::mem::size_of::<T>(), std::mem::size_of::<U>());
        debug_assert_eq!(std::mem::align_of::<T>(), std::mem::align_of::<U>());
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    fn compatible_cast_mut<U>(&mut self) -> &mut Self::Output<U>
    where
        T: Compatible<U>,
    {
        debug_assert_eq!(std::mem::size_of::<T>(), std::mem::size_of::<U>());
        debug_assert_eq!(std::mem::align_of::<T>(), std::mem::align_of::<U>());
        unsafe { std::mem::transmute(self) }
    }
}

mod private {
    use crate::data::layout::valid_layout::ValidLayout;

    pub trait CompatibleCastPriv {}

    impl<T: ValidLayout> CompatibleCastPriv for T {}

    impl<T: ValidLayout> CompatibleCastPriv for [T] {}

    impl<T: ValidLayout, const N: usize> CompatibleCastPriv for [T; N] {}
}

#[cfg(test)]
mod tests {
    use super::{Compatible, CompatibleCast};
    use crate::data::{layout::valid_layout::ValidLayout, managed::value::Value};

    #[repr(C)]
    struct A {
        a: f64,
        b: f32,
    }

    #[repr(C)]
    struct B {
        c: f64,
        d: f32,
    }

    unsafe impl ValidLayout for A {
        fn valid_layout(_: Value) -> bool {
            unimplemented!()
        }

        fn type_object<'target, Tgt: crate::prelude::Target<'target>>(
            _target: &Tgt,
        ) -> Value<'target, 'static> {
            unimplemented!()
        }
    }

    unsafe impl Compatible<B> for A {}

    #[test]
    fn compatible_cast_ref() {
        let a = &A { a: 1.0, b: 2.0 };

        {
            let b = a.compatible_cast::<B>();
            assert_eq!(b.c, a.a);
            assert_eq!(b.d, a.b);
        }
    }

    #[test]
    fn compatible_cast_mut() {
        let a = &mut A { a: 1.0, b: 2.0 };

        {
            let b = a.compatible_cast_mut::<B>();
            b.c = 2.0;
            b.d = 3.0
        }

        assert_eq!(a.a, 2.0);
        assert_eq!(a.b, 3.0);
    }

    #[test]
    fn compatible_cast_slice() {
        let a = &[A { a: 1.0, b: 2.0 }][..];

        {
            let b = a.compatible_cast::<B>();
            assert_eq!(b[0].c, a[0].a);
            assert_eq!(b[0].d, a[0].b);
        }
    }

    #[test]
    fn compatible_cast_slice_mut() {
        let a = &mut [A { a: 1.0, b: 2.0 }][..];

        {
            let b = a.compatible_cast_mut::<B>();
            b[0].c = 2.0;
            b[0].d = 3.0;
        }

        assert_eq!(a[0].a, 2.0);
        assert_eq!(a[0].b, 3.0);
    }

    #[test]
    fn compatible_cast_sized_slice() {
        let a = &[A { a: 1.0, b: 2.0 }];

        {
            let b = a.compatible_cast::<B>();
            assert_eq!(b[0].c, a[0].a);
            assert_eq!(b[0].d, a[0].b);
        }
    }

    #[test]
    fn compatible_cast_sized_slice_mut() {
        let a = &mut [A { a: 1.0, b: 2.0 }];

        {
            let b = a.compatible_cast_mut::<B>();
            b[0].c = 2.0;
            b[0].d = 3.0;
        }

        assert_eq!(a[0].a, 2.0);
        assert_eq!(a[0].b, 3.0);
    }
}
