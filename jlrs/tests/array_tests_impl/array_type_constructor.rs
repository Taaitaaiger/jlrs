#[cfg(feature = "local-rt")]
pub(crate) mod tests {
    use jlrs::{
        data::{
            managed::{
                array::{RankedArray, TypedRankedArray},
                union_all::UnionAll,
            },
            types::{
                abstract_type::Number,
                construct_type::{
                    ArrayTypeConstructor, ConstantChar, ConstantIsize, ConstantSize, ConstructType,
                    TypeVarConstructor,
                },
            },
        },
        prelude::*,
    };

    use crate::util::JULIA;

    fn fully_qualified_ctor() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let ty =
                        ArrayTypeConstructor::<f32, ConstantIsize<1>>::construct_type(&mut frame);
                    let ty2 = TypedRankedArray::<f32, 1>::construct_type(&mut frame);
                    assert_eq!(ty, ty2);
                    assert!(ty.is::<DataType>());

                    let ty = unsafe { ty.cast_unchecked::<DataType>() };
                    assert!(ty.is::<Array>());
                    assert!(ty.is::<TypedArray<f32>>());
                    assert!(!ty.is::<TypedArray<f64>>());
                    assert!(ty.is::<RankedArray<1>>());
                    assert!(ty.is::<RankedArray<-1>>());
                    assert!(!ty.is::<RankedArray<2>>());
                    assert!(ty.is::<TypedRankedArray<f32, 1>>());
                    assert!(ty.is::<TypedRankedArray<f32, -1>>());
                    assert!(!ty.is::<TypedRankedArray<f32, 2>>());
                    assert!(!ty.is::<TypedRankedArray<f64, 1>>());

                    Ok(())
                })
                .unwrap();
        });
    }

    fn no_rank_ctor() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>().scope(|mut frame| {
                    let ty =
                        ArrayTypeConstructor::<f32, TypeVarConstructor<ConstantChar<'N'>>>::construct_type(&mut frame);
                    let ty2 = TypedRankedArray::<f32, -1>::construct_type(&mut frame);
                    assert_eq!(ty, ty2);
                    assert!(ty.is::<UnionAll>());

                    let base = unsafe { ty.cast_unchecked::<UnionAll>().base_type() };
                    assert!(base.is::<Array>());
                    assert!(base.is::<TypedArray<f32>>());
                    assert!(!base.is::<TypedArray<f64>>());
                    assert!(base.is::<RankedArray<-1>>());
                    assert!(!base.is::<RankedArray<1>>());
                    assert!(base.is::<TypedRankedArray<f32, -1>>());
                    assert!(!base.is::<TypedRankedArray<f32, 1>>());
                    assert!(!base.is::<TypedRankedArray<f64, -1>>());

                    Ok(())
                })
                .unwrap();
        });
    }

    fn no_type_ctor() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let ty = ArrayTypeConstructor::<
                        TypeVarConstructor<ConstantChar<'T'>>,
                        ConstantIsize<1>,
                    >::construct_type(&mut frame);
                    let ty2 = RankedArray::<1>::construct_type(&mut frame);
                    assert_eq!(ty, ty2);
                    assert!(ty.is::<UnionAll>());

                    let base = unsafe { ty.cast_unchecked::<UnionAll>().base_type() };
                    assert!(base.is::<Array>());
                    assert!(!base.is::<TypedArray<f32>>());
                    assert!(base.is::<RankedArray<-1>>());
                    assert!(base.is::<RankedArray<1>>());
                    assert!(!base.is::<TypedRankedArray<f32, -1>>());
                    assert!(!base.is::<TypedRankedArray<f32, 1>>());
                    assert!(!base.is::<TypedRankedArray<f64, -1>>());

                    Ok(())
                })
                .unwrap();
        });
    }

    fn generic_ctor() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let ty = ArrayTypeConstructor::<
                        TypeVarConstructor<ConstantChar<'T'>>,
                        TypeVarConstructor<ConstantChar<'N'>>,
                    >::construct_type(&mut frame);
                    let ty2 = Array::construct_type(&mut frame);
                    assert_eq!(ty, ty2);
                    assert!(ty.is::<UnionAll>());

                    let base = unsafe { ty.cast_unchecked::<UnionAll>().base_type() };
                    assert!(base.is::<Array>());
                    assert!(!base.is::<TypedArray<f32>>());
                    assert!(base.is::<RankedArray<-1>>());
                    assert!(!base.is::<RankedArray<1>>());
                    assert!(!base.is::<TypedRankedArray<f32, -1>>());
                    assert!(!base.is::<TypedRankedArray<f32, 1>>());
                    assert!(!base.is::<TypedRankedArray<f64, -1>>());

                    Ok(())
                })
                .unwrap();
        });
    }

    fn ua_type_ctor() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let ty = ArrayTypeConstructor::<
                        RankedArray<1>,
                        TypeVarConstructor<ConstantChar<'N'>>,
                    >::construct_type(&mut frame);
                    let ty2 = TypedArray::<RankedArray<1>>::construct_type(&mut frame);
                    assert_eq!(ty, ty2);
                    assert!(ty.is::<UnionAll>());

                    let base = unsafe { ty.cast_unchecked::<UnionAll>().base_type() };
                    assert!(base.is::<Array>());
                    assert!(base.is::<TypedArray<RankedArray<1>>>());
                    assert!(!base.is::<TypedArray<RankedArray<-1>>>());

                    Ok(())
                })
                .unwrap();
        });
    }

    fn restricted_tvar_type_ctor() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>().scope(|mut frame| {
                    let ty =
                        ArrayTypeConstructor::<TypeVarConstructor<ConstantChar<'T'>, Number>, ConstantSize<1>>::construct_type(&mut frame);
                    let ty2 = TypedRankedArray::<TypeVarConstructor<ConstantChar<'T'>, Number>, 1>::construct_type(&mut frame);
                    assert_eq!(ty, ty2);
                    assert!(ty.is::<UnionAll>());

                    let base = unsafe { ty.cast_unchecked::<UnionAll>().base_type() };
                    assert!(base.is::<Array>());
                    assert!(base.is::<TypedArray::<TypeVarConstructor<ConstantChar<'T'>, Number>>>());
                    assert!(base.is::<TypedRankedArray::<TypeVarConstructor<ConstantChar<'T'>, Number>, 1>>());

                    Ok(())
                })
                .unwrap();
        });
    }

    pub(crate) fn array_type_constructor_tests() {
        fully_qualified_ctor();
        no_rank_ctor();
        no_type_ctor();
        generic_ctor();
        ua_type_ctor();
        restricted_tvar_type_ctor();
    }
}
