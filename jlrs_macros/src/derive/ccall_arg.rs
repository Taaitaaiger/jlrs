use proc_macro::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, Token};

use super::{is_enum, is_repr_c, is_repr_int};

pub fn impl_ccall_arg(ast: &syn::DeriveInput) -> syn::Result<TokenStream> {
    let name = &ast.ident;
    let is_enum = is_enum(&ast.data);

    if !is_enum && !is_repr_c(ast) {
        return Err(syn::Error::new(
            ast.span(),
            "CCallArg can only be derived for #[repr(C)] types",
        ));
    } else if is_enum && !is_repr_int(ast) {
        return Err(syn::Error::new(
            ast.span(),
            "CCallArg can only be derived for enums with an integer repr.",
        ));
    }

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

    let ccall_arg_impl = quote! {
        unsafe impl #generics ::jlrs::convert::ccall_types::CCallArg for #name #generics #wc {
            type CCallArgType = Self;
            type FunctionArgType = Self;
        }
    };

    Ok(ccall_arg_impl.into())
}
