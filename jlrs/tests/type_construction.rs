mod util;

#[cfg(feature = "local-rt")]
mod tests {
    use jlrs::{
        convert::to_symbol::ToSymbol,
        data::{
            managed::{type_var::TypeVar, union::Union, union_all::UnionAll},
            types::{
                abstract_type::{
                    AbstractArray, AbstractChar, AbstractString, Integer, Real, RefTypeConstructor,
                },
                construct_type::{
                    ArrayTypeConstructor, ConstantIsize, ConstructType, Name, TypeVarConstructor,
                    TypeVarName, TypeVars, UnionTypeConstructor,
                },
            },
        },
        prelude::*,
        tvar, tvars,
    };

    use super::util::JULIA;

    fn construct_array_type() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>().scope(|mut frame| {
                    let ty = <ArrayTypeConstructor<AbstractChar, ConstantIsize<2>> as ConstructType>::construct_type(&mut frame);
                    assert!(ty.is::<DataType>());
                    let inner_ty = unsafe {ty.cast_unchecked::<DataType>()};
                    assert!(inner_ty.is::<Array>());

                    let elem_param = inner_ty.parameter(0).unwrap();
                    assert_eq!(elem_param, AbstractChar::construct_type(&mut frame));
                    assert!(elem_param.cast::<DataType>().unwrap().is_abstract());

                    let rank_param = inner_ty.parameter(1).unwrap();
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
                .returning::<JlrsResult<_>>().scope(|mut frame| {
                    let ty = <ArrayTypeConstructor<AbstractChar, TypeVarConstructor<Name<'N'>>> as ConstructType>::construct_type(&mut frame);
                    let ua = ty.cast::<UnionAll>().unwrap();
                    let base = ua.base_type();

                    let elem_param = base.parameter(0).unwrap();
                    assert_eq!(elem_param, AbstractChar::construct_type(&mut frame));

                    let rank_param = base.parameter(1).unwrap();
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
                .returning::<JlrsResult<_>>().scope(|mut frame| {
                    let ty = <ArrayTypeConstructor<TypeVarConstructor<Name<'T'>>, ConstantIsize<1>> as ConstructType>::construct_type(&mut frame);
                    let ua = ty.cast::<UnionAll>().unwrap();
                    let base = ua.base_type();

                    let elem_param = base.parameter(0).unwrap();
                    assert!(elem_param.is::<TypeVar>());

                    let rank_param = base.parameter(1).unwrap();
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
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let ty = <ArrayTypeConstructor<
                        TypeVarConstructor<Name<'T'>, AbstractChar>,
                        ConstantIsize<1>,
                    > as ConstructType>::construct_type(&mut frame);
                    let ua = ty.cast::<UnionAll>().unwrap();
                    let base = ua.base_type();

                    let elem_param = base.parameter(0).unwrap();
                    assert!(elem_param.is::<TypeVar>());

                    let rank_param = base.parameter(1).unwrap();
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
                .returning::<JlrsResult<_>>().scope(|mut frame| {
                    let ty = <UnionTypeConstructor<AbstractChar, Integer> as ConstructType>::construct_type(&mut frame);
                    let un = ty.cast::<Union>().unwrap();
                    let variants = un.variants();
                    assert_eq!(variants.len(), 2);

                    let abstr_char_ty = AbstractChar::construct_type(&mut frame);
                    let integer_ty = Integer::construct_type(&mut frame);

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
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let ty = <UnionTypeConstructor<
                        AbstractChar,
                        UnionTypeConstructor<AbstractString, Real>,
                    > as ConstructType>::construct_type(&mut frame);
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
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    let ty = <UnionTypeConstructor<
                        Integer,
                        UnionTypeConstructor<AbstractChar, Real>,
                    > as ConstructType>::construct_type(&mut frame);
                    let un = ty.cast::<Union>().unwrap();
                    let variants = un.variants();
                    assert_eq!(variants.len(), 2); // Integer <: Real

                    Ok(())
                })
                .unwrap();
        });
    }

    fn construct_with_env() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| {
                    type Foo = encode_as_constant_bytes!("Foo");
                    type Env = tvars!(
                        tvar!(Foo; Integer),
                        tvar!('M'),
                        tvar!('A'; AbstractArray<tvar!(Foo), tvar!('M')>)
                    );
                    type Ty = RefTypeConstructor<tvar!('A')>;

                    let sym = Foo::symbol(&frame);
                    assert_eq!(sym.as_str().unwrap(), "Foo");

                    let env = Env::into_env(&mut frame);

                    let ty = Ty::construct_type_with_env(&mut frame, &env);
                    assert!(ty.is::<UnionAll>());
                    let ua = unsafe { ty.cast_unchecked::<UnionAll>() };

                    let unwrapped_ty = ua.body().cast::<DataType>().unwrap();
                    let param = unwrapped_ty.parameter(0).unwrap();

                    assert!(param.is::<TypeVar>());
                    let tvar = unsafe { param.cast_unchecked::<TypeVar>() };
                    let env_param = env.get("A".to_symbol(&frame)).unwrap();

                    assert_eq!(tvar.as_value(), env_param);

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
        construct_with_env();
    }
}
