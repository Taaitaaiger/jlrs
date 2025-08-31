use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

use super::{attrs::JlrsTypeAttrs, is_enum, is_repr_c, is_repr_int};

pub fn impl_into_julia(ast: &syn::DeriveInput) -> syn::Result<TokenStream> {
    let name = &ast.ident;
    let is_enum = is_enum(&ast.data);

    if !is_enum && !is_repr_c(ast) {
        return Err(syn::Error::new(
            ast.span(),
            "IntoJulia can only be derived for types with the attribute #[repr(C)]",
        ));
    } else if is_enum && !is_repr_int(ast) {
        return Err(syn::Error::new(
            ast.span(),
            "IntoJulia can only be derived for enums with an integer repr.",
        ));
    }

    let attrs = JlrsTypeAttrs::parse(ast);
    let into_julia_fn = if !is_enum {
        impl_into_julia_fn(&attrs)
    } else {
        impl_into_julia_fn_enum()
    };

    let into_julia_impl = quote! {
        unsafe impl ::jlrs::convert::into_julia::IntoJulia for #name {
            #[inline]
            fn julia_type<'scope, T>(target: T) -> ::jlrs::data::managed::datatype::DataTypeData<'scope, T>
            where
                T: ::jlrs::memory::target::Target<'scope>,
            {
                unsafe {
                    <Self as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&target)
                        .as_value()
                        .cast::<::jlrs::data::managed::datatype::DataType>()
                        .expect("Type is not a DataType")
                        .root(target)
                }
            }

            #into_julia_fn
        }
    };

    Ok(into_julia_impl.into())
}

fn impl_into_julia_fn(attrs: &JlrsTypeAttrs) -> Option<proc_macro2::TokenStream> {
    if attrs.zst {
        Some(quote! {
            #[inline]
            fn into_julia<'target, T>(self, target: T) -> ::jlrs::data::managed::value::ValueData<'target, 'static, T>
            where
                T: ::jlrs::memory::target::Target<'target>,
            {
                let ty = Self::julia_type(&target);
                unsafe {
                    ty.as_managed()
                        .instance()
                        .expect("Instance is undefined")
                        .root(target)
                }
            }
        })
    } else {
        None
    }
}

fn impl_into_julia_fn_enum() -> Option<proc_macro2::TokenStream> {
    Some(quote! {
        #[inline]
        fn into_julia<'target, T>(self, target: T) -> ::jlrs::data::managed::value::ValueData<'target, 'static, T>
        where
            T: ::jlrs::memory::target::Target<'target>,
        {
            <Self as ::jlrs::data::layout::julia_enum::Enum>::as_value(&self, &target).root(target)
        }
    })
}
