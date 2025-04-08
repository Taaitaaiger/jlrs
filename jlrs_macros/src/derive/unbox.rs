use proc_macro::TokenStream;
use quote::quote;
use syn::{spanned::Spanned as _, Token};

use super::{is_enum, is_repr_c, is_repr_int};

pub fn impl_unbox(ast: &syn::DeriveInput) -> syn::Result<TokenStream> {
    let name = &ast.ident;
    let is_enum = is_enum(&ast.data);

    if !is_enum && !is_repr_c(ast) {
        return Err(syn::Error::new(
            ast.span(),
            "Unbox can only be derived for types with the attribute #[repr(C)]",
        ));
    } else if is_enum && !is_repr_int(ast) {
        return Err(syn::Error::new(
            ast.span(),
            "Unbox can only be derived for enums with an integer repr.",
        ));
    }

    let generics = &ast.generics;
    let where_clause = match ast.generics.where_clause.as_ref() {
        Some(wc) => {
            let mut wc = wc.clone();
            for generic in generics.type_params() {
                let clause: syn::WherePredicate = syn::parse_quote! {
                    #generic: Clone
                };
                wc.predicates.push(clause)
            }
            wc
        }
        None => {
            let mut predicates = syn::punctuated::Punctuated::<_, Token![,]>::new();
            for generic in generics.type_params() {
                let clause: syn::WherePredicate = syn::parse_quote! {
                    #generic: Clone
                };
                predicates.push(clause)
            }

            syn::parse_quote! {
                where #predicates
            }
        }
    };

    let unbox_impl = quote! {
        unsafe impl #generics ::jlrs::convert::unbox::Unbox for #name #generics #where_clause {
            type Output = Self;
        }
    };

    Ok(unbox_impl.into())
}
