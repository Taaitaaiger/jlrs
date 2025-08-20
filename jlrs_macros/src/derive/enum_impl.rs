use proc_macro::TokenStream;
use quote::quote;
use syn::{Token, spanned::Spanned};

use super::get_repr_int;

pub fn impl_enum(ast: &syn::DeriveInput) -> syn::Result<TokenStream> {
    let name = &ast.ident;
    let repr = match get_repr_int(ast) {
        Some(repr) => repr,
        None => {
            return Err(syn::Error::new(
                ast.span(),
                "`Enum` can only be derived for enums with an integer repr.",
            ));
        }
    };

    let syn::Data::Enum(data) = &ast.data else {
        return Err(syn::Error::new(ast.span(), "Not an enum."));
    };

    let mut variants = Vec::with_capacity(data.variants.len());
    for variant in data.variants.iter() {
        'variant: for attr in variant.attrs.iter() {
            if attr.path().is_ident("jlrs") {
                let parsed: syn::punctuated::Punctuated<syn::Meta, Token![,]> = attr
                    .parse_args_with(
                        syn::punctuated::Punctuated::<syn::Meta, Token![,]>::parse_terminated,
                    )?;

                for meta in parsed {
                    let syn::Meta::NameValue(pair) = meta else {
                        continue;
                    };

                    if !pair.path.is_ident("julia_enum_variant") {
                        continue;
                    }

                    let syn::Expr::Lit(syn::PatLit {
                        lit: syn::Lit::Str(s),
                        ..
                    }) = pair.value
                    else {
                        continue;
                    };

                    let variant_path = s.token().clone();
                    variants.push(variant_path);
                    break 'variant;
                }
            }
        }
    }

    if data.variants.len() != variants.len() {
        return Err(syn::Error::new(
            ast.span(),
            "All enum variants must be annotated with `julia_enum_variant`",
        ));
    }

    let idents = data.variants.iter().map(|x| &x.ident);

    let enum_impl = quote! {
        unsafe impl ::jlrs::data::layout::julia_enum::Enum for #name {
            type Super = #repr;
            fn as_value<'target, Tgt: Target<'target>>(&self, target: &Tgt) -> Value<'target, 'static> {
                match self {
                    #(
                        #name::#idents => ::jlrs::inline_static_ref!(VARIANT, Value, #variants, target),
                    )*
                }
            }

            fn as_super(&self) -> Self::Super {
                *self as _
            }
        }
    };

    Ok(enum_impl.into())
}
