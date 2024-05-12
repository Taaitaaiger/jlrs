//! Convert data to and from a `SimpleVector`.

use crate::{
    data::managed::simple_vector::{SimpleVector, SimpleVectorData},
    prelude::{Managed, Target},
    weak_handle_unchecked,
};

/// Convert data to a `SimpleVector`.
///
/// Safety: Must be the inverse of [`FromSimpleVector`].
pub unsafe trait IntoSimpleVector<'scope> {
    /// Must be `Self` with the `'scope` lifetime replaced by `'s`.
    type InScope<'s>: IntoSimpleVector<'s> + FromSimpleVector<'s>;

    /// Convert `self` to a `SimpleVector`.
    fn into_simple_vector<'target, Tgt: Target<'target>>(
        self,
        target: Tgt,
    ) -> SimpleVectorData<'target, Tgt>;
}

unsafe impl<'scope> IntoSimpleVector<'scope> for () {
    type InScope<'s> = ();

    fn into_simple_vector<'target, Tgt: Target<'target>>(
        self,
        target: Tgt,
    ) -> SimpleVectorData<'target, Tgt> {
        SimpleVector::emptysvec(&target).root(target)
    }
}

unsafe impl<'scope, T1: Managed<'scope, 'static>> IntoSimpleVector<'scope> for (T1,) {
    type InScope<'s> = (T1::InScope<'s>,);

    fn into_simple_vector<'target, Tgt: Target<'target>>(
        self,
        target: Tgt,
    ) -> SimpleVectorData<'target, Tgt> {
        unsafe {
            let svec = SimpleVector::with_capacity_uninit(&target, 1).as_managed();
            let data = svec.data();
            data.set(0, Some(self.0.as_value())).ok();
            svec.root(target)
        }
    }
}

unsafe impl<'scope, T1, T2> IntoSimpleVector<'scope> for (T1, T2)
where
    T1: Managed<'scope, 'static>,
    T2: Managed<'scope, 'static>,
{
    type InScope<'s> = (T1::InScope<'s>, T2::InScope<'s>);

    fn into_simple_vector<'target, Tgt: Target<'target>>(
        self,
        target: Tgt,
    ) -> SimpleVectorData<'target, Tgt> {
        unsafe {
            let svec = SimpleVector::with_capacity_uninit(&target, 2).as_managed();
            let data = svec.data();
            data.set(0, Some(self.0.as_value())).ok();
            data.set(1, Some(self.1.as_value())).ok();
            svec.root(target)
        }
    }
}

unsafe impl<'scope, T1, T2, T3> IntoSimpleVector<'scope> for (T1, T2, T3)
where
    T1: Managed<'scope, 'static>,
    T2: Managed<'scope, 'static>,
    T3: Managed<'scope, 'static>,
{
    type InScope<'s> = (T1::InScope<'s>, T2::InScope<'s>, T3::InScope<'s>);

    fn into_simple_vector<'target, Tgt: Target<'target>>(
        self,
        target: Tgt,
    ) -> SimpleVectorData<'target, Tgt> {
        unsafe {
            let svec = SimpleVector::with_capacity_uninit(&target, 3).as_managed();
            let data = svec.data();
            data.set(0, Some(self.0.as_value())).ok();
            data.set(1, Some(self.1.as_value())).ok();
            data.set(2, Some(self.2.as_value())).ok();
            svec.root(target)
        }
    }
}

unsafe impl<'scope, T1, T2, T3, T4> IntoSimpleVector<'scope> for (T1, T2, T3, T4)
where
    T1: Managed<'scope, 'static>,
    T2: Managed<'scope, 'static>,
    T3: Managed<'scope, 'static>,
    T4: Managed<'scope, 'static>,
{
    type InScope<'s> = (
        T1::InScope<'s>,
        T2::InScope<'s>,
        T3::InScope<'s>,
        T4::InScope<'s>,
    );

    fn into_simple_vector<'target, Tgt: Target<'target>>(
        self,
        target: Tgt,
    ) -> SimpleVectorData<'target, Tgt> {
        unsafe {
            let svec = SimpleVector::with_capacity_uninit(&target, 4).as_managed();
            let data = svec.data();
            data.set(0, Some(self.0.as_value())).ok();
            data.set(1, Some(self.1.as_value())).ok();
            data.set(2, Some(self.2.as_value())).ok();
            data.set(3, Some(self.3.as_value())).ok();
            svec.root(target)
        }
    }
}

unsafe impl<'scope, T1, T2, T3, T4, T5> IntoSimpleVector<'scope> for (T1, T2, T3, T4, T5)
where
    T1: Managed<'scope, 'static>,
    T2: Managed<'scope, 'static>,
    T3: Managed<'scope, 'static>,
    T4: Managed<'scope, 'static>,
    T5: Managed<'scope, 'static>,
{
    type InScope<'s> = (
        T1::InScope<'s>,
        T2::InScope<'s>,
        T3::InScope<'s>,
        T4::InScope<'s>,
        T5::InScope<'s>,
    );

    fn into_simple_vector<'target, Tgt: Target<'target>>(
        self,
        target: Tgt,
    ) -> SimpleVectorData<'target, Tgt> {
        unsafe {
            let svec = SimpleVector::with_capacity_uninit(&target, 5).as_managed();
            let data = svec.data();
            data.set(0, Some(self.0.as_value())).ok();
            data.set(1, Some(self.1.as_value())).ok();
            data.set(2, Some(self.2.as_value())).ok();
            data.set(3, Some(self.3.as_value())).ok();
            data.set(4, Some(self.4.as_value())).ok();
            svec.root(target)
        }
    }
}

unsafe impl<'scope, T1, T2, T3, T4, T5, T6> IntoSimpleVector<'scope> for (T1, T2, T3, T4, T5, T6)
where
    T1: Managed<'scope, 'static>,
    T2: Managed<'scope, 'static>,
    T3: Managed<'scope, 'static>,
    T4: Managed<'scope, 'static>,
    T5: Managed<'scope, 'static>,
    T6: Managed<'scope, 'static>,
{
    type InScope<'s> = (
        T1::InScope<'s>,
        T2::InScope<'s>,
        T3::InScope<'s>,
        T4::InScope<'s>,
        T5::InScope<'s>,
        T6::InScope<'s>,
    );

    fn into_simple_vector<'target, Tgt: Target<'target>>(
        self,
        target: Tgt,
    ) -> SimpleVectorData<'target, Tgt> {
        unsafe {
            let svec = SimpleVector::with_capacity_uninit(&target, 6).as_managed();
            let data = svec.data();
            data.set(0, Some(self.0.as_value())).ok();
            data.set(1, Some(self.1.as_value())).ok();
            data.set(2, Some(self.2.as_value())).ok();
            data.set(3, Some(self.3.as_value())).ok();
            data.set(4, Some(self.4.as_value())).ok();
            data.set(5, Some(self.5.as_value())).ok();
            svec.root(target)
        }
    }
}

unsafe impl<'scope, T1, T2, T3, T4, T5, T6, T7> IntoSimpleVector<'scope>
    for (T1, T2, T3, T4, T5, T6, T7)
where
    T1: Managed<'scope, 'static>,
    T2: Managed<'scope, 'static>,
    T3: Managed<'scope, 'static>,
    T4: Managed<'scope, 'static>,
    T5: Managed<'scope, 'static>,
    T6: Managed<'scope, 'static>,
    T7: Managed<'scope, 'static>,
{
    type InScope<'s> = (
        T1::InScope<'s>,
        T2::InScope<'s>,
        T3::InScope<'s>,
        T4::InScope<'s>,
        T5::InScope<'s>,
        T6::InScope<'s>,
        T7::InScope<'s>,
    );

    fn into_simple_vector<'target, Tgt: Target<'target>>(
        self,
        target: Tgt,
    ) -> SimpleVectorData<'target, Tgt> {
        unsafe {
            let svec = SimpleVector::with_capacity_uninit(&target, 7).as_managed();
            let data = svec.data();
            data.set(0, Some(self.0.as_value())).ok();
            data.set(1, Some(self.1.as_value())).ok();
            data.set(2, Some(self.2.as_value())).ok();
            data.set(3, Some(self.3.as_value())).ok();
            data.set(4, Some(self.4.as_value())).ok();
            data.set(5, Some(self.5.as_value())).ok();
            data.set(6, Some(self.6.as_value())).ok();
            svec.root(target)
        }
    }
}

unsafe impl<'scope, T1, T2, T3, T4, T5, T6, T7, T8> IntoSimpleVector<'scope>
    for (T1, T2, T3, T4, T5, T6, T7, T8)
where
    T1: Managed<'scope, 'static>,
    T2: Managed<'scope, 'static>,
    T3: Managed<'scope, 'static>,
    T4: Managed<'scope, 'static>,
    T5: Managed<'scope, 'static>,
    T6: Managed<'scope, 'static>,
    T7: Managed<'scope, 'static>,
    T8: Managed<'scope, 'static>,
{
    type InScope<'s> = (
        T1::InScope<'s>,
        T2::InScope<'s>,
        T3::InScope<'s>,
        T4::InScope<'s>,
        T5::InScope<'s>,
        T6::InScope<'s>,
        T7::InScope<'s>,
        T8::InScope<'s>,
    );

    fn into_simple_vector<'target, Tgt: Target<'target>>(
        self,
        target: Tgt,
    ) -> SimpleVectorData<'target, Tgt> {
        unsafe {
            let svec = SimpleVector::with_capacity_uninit(&target, 8).as_managed();
            let data = svec.data();
            data.set(0, Some(self.0.as_value())).ok();
            data.set(1, Some(self.1.as_value())).ok();
            data.set(2, Some(self.2.as_value())).ok();
            data.set(3, Some(self.3.as_value())).ok();
            data.set(4, Some(self.4.as_value())).ok();
            data.set(5, Some(self.5.as_value())).ok();
            data.set(6, Some(self.6.as_value())).ok();
            data.set(7, Some(self.7.as_value())).ok();
            svec.root(target)
        }
    }
}

/// Extract data from a `SimpleVector`.
///
/// Safety: Must be the inverse of [`IntoSimpleVector`].
pub unsafe trait FromSimpleVector<'scope>: IntoSimpleVector<'scope> {
    /// Convert a `SimpleVector` to `Self`.
    ///
    /// Safety: `svec` must have been created by calling `Self::into_simple_vector`.
    unsafe fn from_simple_vector(svec: SimpleVector) -> Self;
}

unsafe impl<'scope> FromSimpleVector<'scope> for () {
    unsafe fn from_simple_vector(_svec: SimpleVector) -> Self {}
}

unsafe impl<'scope, T1> FromSimpleVector<'scope> for (T1,)
where
    T1: Managed<'scope, 'static>,
{
    unsafe fn from_simple_vector(svec: SimpleVector) -> Self {
        unsafe {
            let weak_handle = weak_handle_unchecked!();
            let data = svec.data();
            let t1 = data
                .get(&weak_handle, 0)
                .unwrap()
                .leak()
                .as_value()
                .cast_unchecked::<T1>();

            (t1,)
        }
    }
}

unsafe impl<'scope, T1, T2> FromSimpleVector<'scope> for (T1, T2)
where
    T1: Managed<'scope, 'static>,
    T2: Managed<'scope, 'static>,
{
    unsafe fn from_simple_vector(svec: SimpleVector) -> Self {
        unsafe {
            let weak_handle = weak_handle_unchecked!();
            let data = svec.data();
            let t1 = data
                .get(&weak_handle, 0)
                .unwrap()
                .leak()
                .as_value()
                .cast_unchecked::<T1>();

            let t2 = data
                .get(&weak_handle, 1)
                .unwrap()
                .leak()
                .as_value()
                .cast_unchecked::<T2>();

            (t1, t2)
        }
    }
}

unsafe impl<'scope, T1, T2, T3> FromSimpleVector<'scope> for (T1, T2, T3)
where
    T1: Managed<'scope, 'static>,
    T2: Managed<'scope, 'static>,
    T3: Managed<'scope, 'static>,
{
    unsafe fn from_simple_vector(svec: SimpleVector) -> Self {
        unsafe {
            let weak_handle = weak_handle_unchecked!();
            let data = svec.data();
            let t1 = data
                .get(&weak_handle, 0)
                .unwrap()
                .leak()
                .as_value()
                .cast_unchecked::<T1>();

            let t2 = data
                .get(&weak_handle, 1)
                .unwrap()
                .leak()
                .as_value()
                .cast_unchecked::<T2>();

            let t3 = data
                .get(&weak_handle, 2)
                .unwrap()
                .leak()
                .as_value()
                .cast_unchecked::<T3>();

            (t1, t2, t3)
        }
    }
}

unsafe impl<'scope, T1, T2, T3, T4> FromSimpleVector<'scope> for (T1, T2, T3, T4)
where
    T1: Managed<'scope, 'static>,
    T2: Managed<'scope, 'static>,
    T3: Managed<'scope, 'static>,
    T4: Managed<'scope, 'static>,
{
    unsafe fn from_simple_vector(svec: SimpleVector) -> Self {
        unsafe {
            let weak_handle = weak_handle_unchecked!();
            let data = svec.data();
            let t1 = data
                .get(&weak_handle, 0)
                .unwrap()
                .leak()
                .as_value()
                .cast_unchecked::<T1>();

            let t2 = data
                .get(&weak_handle, 1)
                .unwrap()
                .leak()
                .as_value()
                .cast_unchecked::<T2>();

            let t3 = data
                .get(&weak_handle, 2)
                .unwrap()
                .leak()
                .as_value()
                .cast_unchecked::<T3>();

            let t4 = data
                .get(&weak_handle, 3)
                .unwrap()
                .leak()
                .as_value()
                .cast_unchecked::<T4>();

            (t1, t2, t3, t4)
        }
    }
}

unsafe impl<'scope, T1, T2, T3, T4, T5> FromSimpleVector<'scope> for (T1, T2, T3, T4, T5)
where
    T1: Managed<'scope, 'static>,
    T2: Managed<'scope, 'static>,
    T3: Managed<'scope, 'static>,
    T4: Managed<'scope, 'static>,
    T5: Managed<'scope, 'static>,
{
    unsafe fn from_simple_vector(svec: SimpleVector) -> Self {
        unsafe {
            let weak_handle = weak_handle_unchecked!();
            let data = svec.data();
            let t1 = data
                .get(&weak_handle, 0)
                .unwrap()
                .leak()
                .as_value()
                .cast_unchecked::<T1>();

            let t2 = data
                .get(&weak_handle, 1)
                .unwrap()
                .leak()
                .as_value()
                .cast_unchecked::<T2>();

            let t3 = data
                .get(&weak_handle, 2)
                .unwrap()
                .leak()
                .as_value()
                .cast_unchecked::<T3>();

            let t4 = data
                .get(&weak_handle, 3)
                .unwrap()
                .leak()
                .as_value()
                .cast_unchecked::<T4>();

            let t5 = data
                .get(&weak_handle, 4)
                .unwrap()
                .leak()
                .as_value()
                .cast_unchecked::<T5>();

            (t1, t2, t3, t4, t5)
        }
    }
}

unsafe impl<'scope, T1, T2, T3, T4, T5, T6> FromSimpleVector<'scope> for (T1, T2, T3, T4, T5, T6)
where
    T1: Managed<'scope, 'static>,
    T2: Managed<'scope, 'static>,
    T3: Managed<'scope, 'static>,
    T4: Managed<'scope, 'static>,
    T5: Managed<'scope, 'static>,
    T6: Managed<'scope, 'static>,
{
    unsafe fn from_simple_vector(svec: SimpleVector) -> Self {
        unsafe {
            let weak_handle = weak_handle_unchecked!();
            let data = svec.data();
            let t1 = data
                .get(&weak_handle, 0)
                .unwrap()
                .leak()
                .as_value()
                .cast_unchecked::<T1>();

            let t2 = data
                .get(&weak_handle, 1)
                .unwrap()
                .leak()
                .as_value()
                .cast_unchecked::<T2>();

            let t3 = data
                .get(&weak_handle, 2)
                .unwrap()
                .leak()
                .as_value()
                .cast_unchecked::<T3>();

            let t4 = data
                .get(&weak_handle, 3)
                .unwrap()
                .leak()
                .as_value()
                .cast_unchecked::<T4>();

            let t5 = data
                .get(&weak_handle, 4)
                .unwrap()
                .leak()
                .as_value()
                .cast_unchecked::<T5>();

            let t6 = data
                .get(&weak_handle, 5)
                .unwrap()
                .leak()
                .as_value()
                .cast_unchecked::<T6>();

            (t1, t2, t3, t4, t5, t6)
        }
    }
}

unsafe impl<'scope, T1, T2, T3, T4, T5, T6, T7> FromSimpleVector<'scope>
    for (T1, T2, T3, T4, T5, T6, T7)
where
    T1: Managed<'scope, 'static>,
    T2: Managed<'scope, 'static>,
    T3: Managed<'scope, 'static>,
    T4: Managed<'scope, 'static>,
    T5: Managed<'scope, 'static>,
    T6: Managed<'scope, 'static>,
    T7: Managed<'scope, 'static>,
{
    unsafe fn from_simple_vector(svec: SimpleVector) -> Self {
        unsafe {
            let weak_handle = weak_handle_unchecked!();
            let data = svec.data();
            let t1 = data
                .get(&weak_handle, 0)
                .unwrap()
                .leak()
                .as_value()
                .cast_unchecked::<T1>();

            let t2 = data
                .get(&weak_handle, 1)
                .unwrap()
                .leak()
                .as_value()
                .cast_unchecked::<T2>();

            let t3 = data
                .get(&weak_handle, 2)
                .unwrap()
                .leak()
                .as_value()
                .cast_unchecked::<T3>();

            let t4 = data
                .get(&weak_handle, 3)
                .unwrap()
                .leak()
                .as_value()
                .cast_unchecked::<T4>();

            let t5 = data
                .get(&weak_handle, 4)
                .unwrap()
                .leak()
                .as_value()
                .cast_unchecked::<T5>();

            let t6 = data
                .get(&weak_handle, 5)
                .unwrap()
                .leak()
                .as_value()
                .cast_unchecked::<T6>();

            let t7 = data
                .get(&weak_handle, 6)
                .unwrap()
                .leak()
                .as_value()
                .cast_unchecked::<T7>();

            (t1, t2, t3, t4, t5, t6, t7)
        }
    }
}

unsafe impl<'scope, T1, T2, T3, T4, T5, T6, T7, T8> FromSimpleVector<'scope>
    for (T1, T2, T3, T4, T5, T6, T7, T8)
where
    T1: Managed<'scope, 'static>,
    T2: Managed<'scope, 'static>,
    T3: Managed<'scope, 'static>,
    T4: Managed<'scope, 'static>,
    T5: Managed<'scope, 'static>,
    T6: Managed<'scope, 'static>,
    T7: Managed<'scope, 'static>,
    T8: Managed<'scope, 'static>,
{
    unsafe fn from_simple_vector(svec: SimpleVector) -> Self {
        unsafe {
            let weak_handle = weak_handle_unchecked!();
            let data = svec.data();
            let t1 = data
                .get(&weak_handle, 0)
                .unwrap()
                .leak()
                .as_value()
                .cast_unchecked::<T1>();

            let t2 = data
                .get(&weak_handle, 1)
                .unwrap()
                .leak()
                .as_value()
                .cast_unchecked::<T2>();

            let t3 = data
                .get(&weak_handle, 2)
                .unwrap()
                .leak()
                .as_value()
                .cast_unchecked::<T3>();

            let t4 = data
                .get(&weak_handle, 3)
                .unwrap()
                .leak()
                .as_value()
                .cast_unchecked::<T4>();

            let t5 = data
                .get(&weak_handle, 4)
                .unwrap()
                .leak()
                .as_value()
                .cast_unchecked::<T5>();

            let t6 = data
                .get(&weak_handle, 5)
                .unwrap()
                .leak()
                .as_value()
                .cast_unchecked::<T6>();

            let t7 = data
                .get(&weak_handle, 6)
                .unwrap()
                .leak()
                .as_value()
                .cast_unchecked::<T7>();

            let t8 = data
                .get(&weak_handle, 7)
                .unwrap()
                .leak()
                .as_value()
                .cast_unchecked::<T8>();

            (t1, t2, t3, t4, t5, t6, t7, t8)
        }
    }
}
