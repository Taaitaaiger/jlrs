use std::collections::HashMap;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_quote, spanned::Spanned as _};

use super::attrs::{JlrsTypeAttrs, TypeBound};

pub fn impl_opaque_type(ast: &syn::DeriveInput) -> syn::Result<TokenStream> {
    let name = &ast.ident;
    let attrs = JlrsTypeAttrs::parse(ast);

    let generics = prepare_generics(ast)?;
    let bounds = attrs.bounds()?;
    let impl_params = generics.impl_params.as_slice();
    bounds.validate(impl_params)?;

    let names = generics.impl_params.as_slice().iter().map(|p| &p.ident);
    let bounds_map = bounds.to_map();
    let type_vars = type_var_iter(names, &bounds_map);
    let key_type = attrs.key_type(ast)?;
    let super_type_fn = generate_super_type_fn(attrs.super_type()?.as_ref());

    let n_params = generics.impl_params.len();
    let types = generics.impl_params.iter().map(|p| &p.ident);
    let where_clause = &generics.where_clause;
    let tvar_idxs = 0..n_params;
    let variant_idxs = tvar_idxs.clone();

    let opaque_type_impl = quote! {
        unsafe impl < #(#impl_params,)* > ::jlrs::data::types::foreign_type::OpaqueType for #name < #(#impl_params,)* > #where_clause {
            type Key = #key_type;
            const N_PARAMS: usize = #n_params;

            #super_type_fn

            fn type_parameters<'target, Tgt>(
                target: Tgt,
            ) -> ::jlrs::data::managed::simple_vector::SimpleVectorData<'target, Tgt>
            where
                Tgt: ::jlrs::memory::target::Target<'target>,
            {
                use ::jlrs::memory::scope::LocalScopeExt as _;
                target.with_local_scope::<2>(
                    |target, mut frame| unsafe {
                        let mut output = frame.output();
                        let tvars = ::jlrs::data::managed::simple_vector::SimpleVector::with_capacity_uninit(&mut frame, #n_params);
                        let mut tvars_data = tvars.data();
                        #(
                            let tvar = #type_vars;
                            tvars_data.set(#tvar_idxs, Some(tvar.as_value())).unwrap();
                        )*
                        <::jlrs::data::managed::simple_vector::SimpleVector as ::jlrs::data::managed::Managed>::root(tvars, target)
                    })
            }

            fn variant_parameters<'target, Tgt>(
                target: Tgt,
            ) -> ::jlrs::data::managed::simple_vector::SimpleVectorData<'target, Tgt>
            where
                Tgt: ::jlrs::memory::target::Target<'target>,
            {
                use ::jlrs::memory::scope::LocalScopeExt as _;
                target.with_local_scope::<2>(
                    |target, mut frame| unsafe {
                        let mut output = frame.output();
                        let types = ::jlrs::data::managed::simple_vector::SimpleVector::with_capacity_uninit(&mut frame, #n_params);
                        let mut types_data = types.data();
                        #(
                            let ty = <#types as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut output);
                            types_data.set(#variant_idxs, Some(ty.as_value())).unwrap();
                        )*
                        <::jlrs::data::managed::simple_vector::SimpleVector as ::jlrs::data::managed::Managed>::root(types, target)
                    })
            }
        }
    };

    Ok(opaque_type_impl.into())
}

struct OpaqueTypeGenerics {
    impl_params: Vec<syn::TypeParam>,
    where_clause: syn::WhereClause,
}

fn prepare_generics(ast: &syn::DeriveInput) -> syn::Result<OpaqueTypeGenerics> {
    let generics = &ast.generics;
    let params = &generics.params;
    let mut where_clause = generics.where_clause.clone();

    // We're going to move all trait bounds in the parameter list to the where clause.
    let mut impl_params = Vec::with_capacity(params.len());
    let mut additional_where: Vec<syn::WherePredicate> = Vec::with_capacity(params.len());

    // OpaqueType can only be implemented for types that are safe to use from different threads.
    // This requirement exists for two reasons: this data might be used from multiple threads in
    // Julia, and we make the GC responsible for dropping this data which isn't guaranteed to
    // happen on the same thread.
    additional_where.push(parse_quote! {
        Self: 'static + Send + Sync
    });

    // Move all bounds in the parameter list to the where clause, and add the requirement that
    // the generic type must be a type constructor. Reject types with unsupported generics, i.e.
    // lifetimes or const generics.
    for param in params {
        match param {
            syn::GenericParam::Type(type_param) => {
                let name = &type_param.ident;
                let tp: syn::TypeParam = parse_quote! { #name };
                impl_params.push(tp);

                if !type_param.bounds.is_empty() {
                    additional_where.push(parse_quote! { #type_param + ::jlrs::data::types::construct_type::ConstructType });
                } else {
                    additional_where.push(
                        parse_quote! { #name: ::jlrs::data::types::construct_type::ConstructType },
                    );
                }
            }
            _ => {
                return Err(syn::Error::new(
                    param.span(),
                    "OpaqueType cannot be derived for types with lifetimes or const generics",
                ));
            }
        }
    }

    // Extend the existing where clause if it exists, create a new one otherwise.
    match where_clause.as_mut() {
        Some(where_clause_ref) => {
            for additional in additional_where.iter() {
                where_clause_ref.predicates.push(parse_quote! {
                    #additional
                });
            }
        }
        None => {
            let where_clause_ref = &mut where_clause;
            *where_clause_ref = Some(parse_quote! {
                where #(#additional_where,)*
            });
        }
    };

    Ok(OpaqueTypeGenerics {
        impl_params,
        where_clause: where_clause.unwrap(),
    })
}

fn generate_super_type_fn(super_type: Option<&syn::Type>) -> Option<syn::ImplItemFn> {
    let super_type = super_type?;
    Some(parse_quote! {
        fn super_type<'target, Tgt>(target: Tgt) -> ::jlrs::data::managed::datatype::DataTypeData<'target, Tgt>
        where
            Tgt: ::jlrs::memory::target::Target<'target>,
        {
            let super_ty = <#super_type as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&target);
            unsafe {
                super_ty.as_value()
                    .cast::<::jlrs::data::managed::datatype::DataType>()
                    .expect("super type is not a datatype")
                    .root(target)
            }
        }
    })
}

// TODO: should the environment be taken into account?
fn type_var_iter<'a>(
    names: impl Iterator<Item = &'a syn::Ident> + 'a,
    bounds_map: &'a HashMap<&'a syn::Ident, &'a TypeBound>,
) -> impl Iterator<Item = syn::Expr> + 'a {
    names.map(|name| -> syn::Expr { match bounds_map.get(name) {
        Some(TypeBound::Subtype { name: _, supertype }) => {
            let name = name.to_string();
            parse_quote! {
                {
                    (&mut output).with_local_scope::<1>(|target, mut frame| {
                        let upper_bound = <#supertype as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame);
                        ::jlrs::data::managed::type_var::TypeVar::new(target, #name, None, Some(upper_bound))
                            .expect("Could not create tvar")
                    })
                }
            }
        }
        Some(TypeBound::SubSupertype {
            subtype,
            name: _,
            supertype,
        }) => {
            let name = name.to_string();
            parse_quote! {
                {
                    (&mut output).with_local_scope::<2>(|target, mut frame| {
                        let lower_bound = <#subtype as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame);
                        let upper_bound = <#supertype as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame);
                        ::jlrs::data::managed::type_var::TypeVar::new(target, #name, Some(lower_bound), Some(upper_bound))
                            .expect("Could not create tvar")
                    })
                }
            }
        }
        Some(TypeBound::Supertype { name:_, subtype }) => {
            let name = name.to_string();
            parse_quote! {
                {
                    (&mut output).with_local_scope::<1>(|target, mut frame| {
                        let lower_bound = <#subtype as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame);
                        ::jlrs::data::managed::type_var::TypeVar::new(target, #name, Some(lower_bound), None)
                            .expect("Could not create tvar")
                    })
                }
            }
        }
        None => {
            let name = name.to_string();
            parse_quote! {
                ::jlrs::data::managed::type_var::TypeVar::new(&mut output, #name, None, None)
                    .expect("Could not create tvar")
            }
        }
    }})
}
