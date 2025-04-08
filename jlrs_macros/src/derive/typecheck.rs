use proc_macro::TokenStream;
use quote::quote;
use syn::{spanned::Spanned as _, Token};

use super::{is_enum, is_repr_c, is_repr_int};

pub fn impl_typecheck(ast: &syn::DeriveInput) -> syn::Result<TokenStream> {
    let name = &ast.ident;
    let is_enum = is_enum(&ast.data);

    if !is_enum && !is_repr_c(ast) {
        return Err(syn::Error::new(
            ast.span(),
            "Typecheck can only be derived for types with the attribute #[repr(C)]",
        ));
    } else if is_enum && !is_repr_int(ast) {
        return Err(syn::Error::new(
            ast.span(),
            "Typecheck can only be derived for enums with an integer repr.",
        ));
    }

    let generics = &ast.generics;
    let where_clause = match ast.generics.where_clause.as_ref() {
        Some(wc) => {
            let mut wc = wc.clone();
            let clause: syn::WherePredicate = syn::parse_quote! {
                Self: ::jlrs::data::layout::valid_layout::ValidLayout
            };
            wc.predicates.push(clause);
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
            let clause: syn::WherePredicate = syn::parse_quote! {
                Self: ::jlrs::data::layout::valid_layout::ValidLayout
            };
            predicates.push(clause);

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

    let typecheck_impl = quote! {
        unsafe impl #generics ::jlrs::data::types::typecheck::Typecheck for #name #generics #where_clause {
            #[inline]
            fn typecheck(dt: ::jlrs::data::managed::datatype::DataType) -> bool {
                <Self as ::jlrs::data::layout::valid_layout::ValidLayout>::valid_layout(dt.as_value())
            }
        }
    };

    Ok(typecheck_impl.into())
}
