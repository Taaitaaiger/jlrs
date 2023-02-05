mod util;

#[cfg(feature = "sync-rt")]
mod tests {
    use jlrs::{
        data::{
            managed::{type_var::TypeVar, union::Union, union_all::UnionAll},
            types::{
                abstract_types::{AbstractChar, AbstractString, Integer, Real},
                construct_type::{
                    ArrayTypeConstructor, ConstantIsize, ConstructType, Name, TypeVarConstructor,
                    UnionTypeConstructor,
                },
            },
        },
        prelude::*,
    };

    use super::util::JULIA;

    fn construct_array_type() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let ty = <ArrayTypeConstructor<AbstractChar, ConstantIsize<2>> as ConstructType>::construct_type(frame.as_extended_target());
                    assert!(ty.is::<DataType>());
                    let inner_ty = unsafe {ty.cast_unchecked::<DataType>()};
                    assert!(inner_ty.is::<Array>());

                    let elem_param = inner_ty.parameter(&mut frame, 0).unwrap();
                    assert_eq!(elem_param, AbstractChar::construct_type(frame.as_extended_target()));
                    assert!(elem_param.cast::<DataType>().unwrap().is_abstract());

                    let rank_param = inner_ty.parameter(&mut frame, 1).unwrap();
                    assert_eq!(rank_param.unbox::<isize>().unwrap(), 2);
                    Ok(())
                })
                .unwrap();
        });
    }

    fn construct_unranked_array_type() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let ty = <ArrayTypeConstructor<AbstractChar, TypeVarConstructor<Name<'N'>>> as ConstructType>::construct_type(frame.as_extended_target());
                    let ua = ty.cast::<UnionAll>().unwrap();
                    let base = ua.base_type();

                    let elem_param = base.parameter(&mut frame, 0).unwrap();
                    assert_eq!(elem_param, AbstractChar::construct_type(frame.as_extended_target()));

                    let rank_param = base.parameter(&mut frame, 1).unwrap();
                    assert!(rank_param.is::<TypeVar>());
                    Ok(())
                })
                .unwrap();
        });
    }

    fn construct_untyped_array_type() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let ty = <ArrayTypeConstructor<TypeVarConstructor<Name<'T'>>, ConstantIsize<1>> as ConstructType>::construct_type(frame.as_extended_target());
                    let ua = ty.cast::<UnionAll>().unwrap();
                    let base = ua.base_type();

                    let elem_param = base.parameter(&mut frame, 0).unwrap();
                    assert!(elem_param.is::<TypeVar>());

                    let rank_param = base.parameter(&mut frame, 1).unwrap();
                    assert_eq!(rank_param.unbox::<isize>().unwrap(), 1);
                    Ok(())
                })
                .unwrap();
        });
    }

    fn construct_array_type_with_bounded_type() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let ty = <ArrayTypeConstructor<
                        TypeVarConstructor<Name<'T'>, AbstractChar>,
                        ConstantIsize<1>,
                    > as ConstructType>::construct_type(
                        frame.as_extended_target()
                    );
                    let ua = ty.cast::<UnionAll>().unwrap();
                    let base = ua.base_type();

                    let elem_param = base.parameter(&mut frame, 0).unwrap();
                    assert!(elem_param.is::<TypeVar>());

                    let rank_param = base.parameter(&mut frame, 1).unwrap();
                    assert_eq!(rank_param.unbox::<isize>().unwrap(), 1);
                    Ok(())
                })
                .unwrap();
        });
    }

    fn construct_union_type() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let ty = <UnionTypeConstructor<AbstractChar, Integer> as ConstructType>::construct_type(frame.as_extended_target());
                    let un = ty.cast::<Union>().unwrap();
                    let variants = un.variants();
                    assert_eq!(variants.len(), 2);

                    let abstr_char_ty = AbstractChar::construct_type(frame.as_extended_target());
                    let integer_ty = Integer::construct_type(frame.as_extended_target());

                    if variants[0] == abstr_char_ty {
                        assert!(variants[1] == integer_ty);
                    } else {
                        assert!(variants[0] == integer_ty);
                        assert!(variants[1] == abstr_char_ty);
                    }

                    Ok(())
                })
                .unwrap();
        });
    }

    fn construct_union_type_three_variants() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let ty = <UnionTypeConstructor<
                        AbstractChar,
                        UnionTypeConstructor<AbstractString, Real>,
                    > as ConstructType>::construct_type(
                        frame.as_extended_target()
                    );
                    let un = ty.cast::<Union>().unwrap();
                    let variants = un.variants();
                    assert_eq!(variants.len(), 3);

                    Ok(())
                })
                .unwrap();
        });
    }

    fn construct_union_type_overlapping_variants() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| {
                    let ty = <UnionTypeConstructor<
                        Integer,
                        UnionTypeConstructor<AbstractChar, Real>,
                    > as ConstructType>::construct_type(
                        frame.as_extended_target()
                    );
                    let un = ty.cast::<Union>().unwrap();
                    let variants = un.variants();
                    assert_eq!(variants.len(), 2); // Integer <: Real

                    Ok(())
                })
                .unwrap();
        });
    }

    #[test]
    fn type_construction_tests() {
        construct_array_type();
        construct_unranked_array_type();
        construct_untyped_array_type();
        construct_array_type_with_bounded_type();
        construct_union_type();
        construct_union_type_three_variants();
        construct_union_type_overlapping_variants();
    }
}
