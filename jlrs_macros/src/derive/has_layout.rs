use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Token, spanned::Spanned};

use super::attrs::JlrsTypeAttrs;

pub fn impl_has_layout(ast: &syn::DeriveInput) -> syn::Result<TokenStream> {
    let name = &ast.ident;
    let attrs = JlrsTypeAttrs::parse(ast);

    let Some(ty) = attrs.constructor_for.as_ref() else {
        return Err(syn::Error::new(
            ast.span(),
            "HasLayout can only be implemented when a layout type is provided",
        ));
    };

    let layout_type = format_ident!("{}", ty);

    let all_params = attrs.all_params.iter().map(|i| format_ident!("{}", i));
    let all_params2 = all_params.clone();
    let all_generics: syn::Generics = syn::parse_quote! {
        <'scope, 'data, #(#all_params,)*>
    };

    let constructor_generics: syn::Generics = syn::parse_quote! {
        <#(#all_params2,)*>
    };

    let layout_params = attrs.layout_params.iter().map(|i| format_ident!("{}", i));
    let mut layout_generics: syn::Generics = syn::parse_quote! {
        <#(#layout_params,)*>
    };

    if attrs.scope_lifetime {
        layout_generics
            .params
            .insert(0, syn::parse_quote! { 'scope });
    }

    // 'data implies 'scope
    if attrs.data_lifetime {
        layout_generics
            .params
            .insert(1, syn::parse_quote! { 'data });
    }

    let where_clause: syn::WhereClause = {
        let mut predicates = syn::punctuated::Punctuated::<_, Token![,]>::new();

        for generic in attrs.layout_params.iter().map(|i| format_ident!("{}", i)) {
            let clause: syn::WherePredicate = syn::parse_quote! {
                #generic: ::jlrs::data::types::construct_type::ConstructType + ::jlrs::data::layout::valid_layout::ValidField
            };

            predicates.push(clause)
        }

        for generic in attrs.elided_params.iter().map(|i| format_ident!("{}", i)) {
            let clause: syn::WherePredicate = syn::parse_quote! {
                #generic: ::jlrs::data::types::construct_type::ConstructType
            };

            predicates.push(clause)
        }

        syn::parse_quote! {
            where #predicates
        }
    };

    let has_layout_impl = quote! {
        unsafe impl #all_generics ::jlrs::data::layout::typed_layout::HasLayout<'scope, 'data> for #name #constructor_generics #where_clause {
            type Layout = #layout_type #layout_generics;
        }
    };

    Ok(has_layout_impl.into())
}
