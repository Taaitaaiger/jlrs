use proc_macro::TokenStream;
use quote::quote;
use syn::{Token, parse_quote, spanned::Spanned as _};

use super::attrs::JlrsTypeAttrs;

pub fn impl_foreign_type(ast: &syn::DeriveInput) -> syn::Result<TokenStream> {
    // TODO: Support enums?
    match &ast.data {
        syn::Data::Struct(_) => (),
        _ => {
            return Err(syn::Error::new(
                ast.span(),
                "ForeignType can only be derived for structs",
            ));
        }
    }

    let name = &ast.ident;
    let attrs = JlrsTypeAttrs::parse(ast);

    let super_type_fn = generate_super_type_fn(attrs.super_type()?.as_ref());
    let mark_fn = generate_mark_fn(&ast)?;

    let opaque_type_impl = quote! {
        unsafe impl ::jlrs::data::types::foreign_type::ForeignType for #name {
            #super_type_fn

            #mark_fn
        }
    };

    Ok(opaque_type_impl.into())
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

fn generate_mark_fn(ast: &syn::DeriveInput) -> syn::Result<syn::ImplItemFn> {
    let fields = match &ast.data {
        syn::Data::Struct(s) => &s.fields,
        _ => unreachable!(),
    };

    let mark_calls = fields.iter().enumerate().filter_map(|(idx, field)| {
        for attr in field.attrs.iter() {
            match ForeignFieldAttr::parse(attr) {
                Some(ForeignFieldAttr::Mark) => {
                    let field_name = field.ident.as_ref();
                    let stmt: syn::Stmt = match field_name {
                        Some(field_name) => parse_quote! {
                            n += data.#field_name.mark(ptls, parent);
                        },
                        None => parse_quote! {
                            n += data.#idx.mark(ptls, parent);
                        },
                    };

                    return Some(stmt);
                }
                Some(ForeignFieldAttr::MarkWith(mark_fn)) => {
                    let field_name = field.ident.as_ref();
                    let stmt: syn::Stmt = match field_name {
                        Some(field_name) => parse_quote! {
                            n += #mark_fn(&data.#field_name, ptls, parent);
                        },
                        None => parse_quote! {
                            n += #mark_fn(&data.#idx, ptls, parent);
                        },
                    };

                    return Some(stmt);
                }
                _ => (),
            }
        }

        None
    });

    Ok(parse_quote! {
        unsafe fn mark<P: ::jlrs::data::types::foreign_type::ForeignType>(ptls: ::jlrs::memory::PTls, data: &Self, parent: &P) -> usize {
            use ::jlrs::data::types::foreign_type::mark::Mark as _;
            let mut n = 0;
            unsafe {
                #(#mark_calls)*
            }

            n
        }
    })
}

enum ForeignFieldAttr {
    Mark,
    MarkWith(syn::Path),
}

impl ForeignFieldAttr {
    pub fn parse(attr: &syn::Attribute) -> Option<Self> {
        if attr.path().is_ident("jlrs") {
            let nested = attr
                .parse_args_with(
                    syn::punctuated::Punctuated::<syn::Meta, Token![,]>::parse_terminated,
                )
                .unwrap();

            for meta in nested {
                match meta {
                    syn::Meta::Path(path) => {
                        if path.is_ident("mark") {
                            return Some(ForeignFieldAttr::Mark);
                        }
                    }
                    syn::Meta::NameValue(pair) => {
                        if pair.path.is_ident("mark_with") {
                            match pair.value {
                                syn::Expr::Path(expr_path) => {
                                    let path = expr_path.path;
                                    return Some(ForeignFieldAttr::MarkWith(path));
                                }
                                _ => (),
                            }
                        }
                    }
                    syn::Meta::List(_) => (),
                }
            }
        }

        None
    }
}
