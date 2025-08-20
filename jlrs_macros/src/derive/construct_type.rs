use proc_macro::TokenStream;
use quote::quote;
use syn::{Token, spanned::Spanned};

use super::attrs::JlrsTypeAttrs;

pub fn impl_construct_type(ast: &syn::DeriveInput) -> syn::Result<TokenStream> {
    let name = &ast.ident;

    let mut attrs = JlrsTypeAttrs::parse(ast);
    let Some(jl_type) = attrs.julia_type.take() else {
        return Err(syn::Error::new(
            ast.span(),
            "ConstructType can only be derived if the corresponding Julia type is set with #[julia_type = \"Main.MyModule.Submodule.StructType\"]",
        ));
    };

    let lifetimes = ast.generics.lifetimes().map(|_| -> syn::LifetimeParam {
        syn::parse_quote! { 'static }
    });

    let static_types = ast.generics.type_params().map(|p| -> syn::Type {
        let name = &p.ident;
        syn::parse_quote! { #name::Static }
    });

    let generics = &ast.generics;
    let wc = match ast.generics.where_clause.as_ref() {
        Some(wc) => {
            let mut wc = wc.clone();
            for generic in generics.type_params() {
                let clause: syn::WherePredicate = syn::parse_quote! {
                    #generic: ::jlrs::data::types::construct_type::ConstructType
                };
                wc.predicates.push(clause)
            }
            wc
        }
        None => {
            let mut predicates = syn::punctuated::Punctuated::<_, Token![,]>::new();
            for generic in generics.type_params() {
                let clause: syn::WherePredicate = syn::parse_quote! {
                    #generic: ::jlrs::data::types::construct_type::ConstructType
                };
                predicates.push(clause)
            }

            syn::parse_quote! {
                where #predicates
            }
        }
    };

    let n_names = ast.generics.type_params().count();
    let n_generics = ast.generics.params.len();

    let (cacheable, construct_expr, construct_with_context_expr): (
        Option<syn::Stmt>,
        syn::Expr,
        syn::Expr,
    ) = if n_names == 0 {
        let cacheable = syn::parse_quote! {
            const CACHEABLE: bool = false;
        };

        let construct_expr = syn::parse_quote! {
            base_type.root(target)
        };

        let construct_with_context_expr = syn::parse_quote! {
            base_type.root(target)
        };

        (Some(cacheable), construct_expr, construct_with_context_expr)
    } else {
        let param_names = ast.generics.type_params().map(|p| &p.ident);
        let n_names = ast.generics.type_params().count();

        let n_slots = n_generics + 2;
        let nth_generic = 0..n_names;

        let construct_expr = syn::parse_quote! {
            <Tgt as ::jlrs::memory::scope::LocalScopeExt<
                'target,
            >>::with_local_scope::<_, #n_slots>(target, |target, mut frame| {

                if #n_names == 0 {
                    return base_type.root(target);
                }

                let mut types: [Option<::jlrs::data::managed::value::Value>; #n_names] = [None; #n_names];
                #(
                    types[#nth_generic] = Some(<#param_names as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame));
                )*
                unsafe {
                    let types = std::mem::transmute::<&[Option<::jlrs::data::managed::value::Value>; #n_names], &[::jlrs::data::managed::value::Value; #n_names]>(&types);
                    base_type
                        .apply_type(&mut frame, types)
                        .unwrap()
                        .cast::<::jlrs::data::managed::datatype::DataType>()
                        .unwrap()
                        .rewrap(target)
                }
            })
        };

        let nth_generic = 0..n_names;
        let param_names = ast.generics.type_params().map(|p| &p.ident);
        let construct_with_context_expr = syn::parse_quote! {
            <Tgt as ::jlrs::memory::scope::LocalScopeExt<
                'target,
            >>::with_local_scope::<_, #n_slots>(target, |target, mut frame| {
                if #n_names == 0 {
                    return base_type.root(target);
                }

                let mut types: [Option<::jlrs::data::managed::value::Value>; #n_names] = [None; #n_names];
                #(
                    types[#nth_generic] = Some(<#param_names as ::jlrs::data::types::construct_type::ConstructType>::construct_type_with_env(&mut frame, env));
                )*
                unsafe {
                    let types = std::mem::transmute::<&[Option<::jlrs::data::managed::value::Value>; #n_names], &[::jlrs::data::managed::value::Value; #n_names]>(&types);
                    base_type
                        .apply_type(&mut frame, types)
                        .unwrap()
                        .cast::<::jlrs::data::managed::datatype::DataType>()
                        .unwrap()
                        .wrap_with_env(target, env)
                }
            })
        };

        (None, construct_expr, construct_with_context_expr)
    };

    let construct_type_impl = quote! {
        unsafe impl #generics ::jlrs::data::types::construct_type::ConstructType for #name #generics #wc {
            type Static = #name < #(#lifetimes,)* #(#static_types,)* >;

            #cacheable

            fn construct_type_uncached<'target, Tgt>(
                target: Tgt,
            ) -> ::jlrs::data::managed::value::ValueData<'target, 'static, Tgt>
            where
                Tgt: ::jlrs::memory::target::Target<'target>,
            {
                let base_type = Self::base_type(&target).unwrap();
                #construct_expr
            }

            fn construct_type_with_env_uncached<'target, Tgt>(
                target: Tgt,
                env: &::jlrs::data::types::construct_type::TypeVarEnv,
            ) -> ::jlrs::data::managed::value::ValueData<'target, 'static, Tgt>
            where
                Tgt: ::jlrs::memory::target::Target<'target>,
            {
                let base_type = Self::base_type(&target).unwrap();
                #construct_with_context_expr
            }

            #[inline]
            fn base_type<'target, Tgt>(
                target: &Tgt
            ) -> Option<::jlrs::data::managed::value::Value<'target, 'static>>
            where
                Tgt: ::jlrs::memory::target::Target<'target>,
            {
                unsafe {
                    let value = ::jlrs::inline_static_ref!(STATIC, Value, #jl_type, target);
                    Some(value)
                }
            }
        }
    };

    Ok(construct_type_impl.into())
}
