use proc_macro::TokenStream;
use quote::quote;
use syn::{Token, spanned::Spanned as _};

use super::{is_enum, is_repr_c, is_repr_int};

pub fn impl_valid_field(ast: &syn::DeriveInput) -> syn::Result<TokenStream> {
    let name = &ast.ident;
    let is_enum = is_enum(&ast.data);

    if !is_enum && !is_repr_c(ast) {
        return Err(syn::Error::new(
            ast.span(),
            "ValidField can only be derived for types with the attribute #[repr(C)]",
        ));
    } else if is_enum && !is_repr_int(ast) {
        return Err(syn::Error::new(
            ast.span(),
            "ValidField can only be derived for enums with an integer repr.",
        ));
    }

    let generics = &ast.generics;
    let where_clause = match ast.generics.where_clause.as_ref() {
        Some(wc) => {
            let mut wc = wc.clone();
            for generic in generics.type_params() {
                let clause: syn::WherePredicate = syn::parse_quote! {
                    #generic: ::jlrs::data::layout::valid_layout::ValidField
                };
                wc.predicates.push(clause)
            }
            wc
        }
        None => {
            let mut predicates = syn::punctuated::Punctuated::<_, Token![,]>::new();
            for generic in generics.type_params() {
                let clause: syn::WherePredicate = syn::parse_quote! {
                    #generic: ::jlrs::data::layout::valid_layout::ValidField
                };
                predicates.push(clause)
            }

            syn::parse_quote! {
                where #predicates
            }
        }
    };

    let valid_field_impl = quote! {
        unsafe impl #generics ::jlrs::data::layout::valid_layout::ValidField for #name #generics #where_clause {
            #[inline]
            fn valid_field(v: ::jlrs::data::managed::value::Value) -> bool {
                <Self as ::jlrs::data::layout::valid_layout::ValidLayout>::valid_layout(v)
            }
        }
    };

    Ok(valid_field_impl.into())
}
