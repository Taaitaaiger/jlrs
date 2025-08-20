use proc_macro::TokenStream;
use quote::quote;
use syn::{Token, spanned::Spanned as _};

use super::{attrs::JlrsTypeAttrs, is_enum, is_repr_c, is_repr_int};

pub fn impl_valid_layout(ast: &syn::DeriveInput) -> syn::Result<TokenStream> {
    let name = &ast.ident;
    let is_enum = is_enum(&ast.data);

    if !is_enum && !is_repr_c(ast) {
        return Err(syn::Error::new(
            ast.span(),
            "ValidLayout can only be derived for types with the attribute #[repr(C)]",
        ));
    } else if is_enum && !is_repr_int(ast) {
        return Err(syn::Error::new(
            ast.span(),
            "ValidLayout can only be derived for enums with an integer repr.",
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

    let mut attrs = JlrsTypeAttrs::parse(ast);
    let jl_type = attrs.julia_type
        .take()
        .expect("ValidLayout can only be derived if the corresponding Julia type is set with #[julia_type = \"Main.MyModule.Submodule.StructType\"]");

    if !is_enum {
        let fields = match &ast.data {
            syn::Data::Struct(s) => &s.fields,
            _ => {
                return Err(syn::Error::new(
                    ast.span(),
                    "ValidLayout can only be derived for structs.",
                ));
            }
        };

        let classified_fields = match fields {
            syn::Fields::Named(n) => ClassifiedFields::classify(n.named.iter()),
            syn::Fields::Unit => ClassifiedFields::default(),
            _ => {
                return Err(syn::Error::new(
                    ast.span(),
                    "ValidLayout cannot be derived for tuple structs.",
                ));
            }
        };

        let rs_flag_fields = classified_fields.rs_flag_fields.iter();
        let rs_align_fields = classified_fields.rs_align_fields.iter();
        let rs_union_fields = classified_fields.rs_union_fields.iter();
        let rs_non_union_fields = classified_fields.rs_non_union_fields.iter();
        let jl_union_field_idxs = classified_fields.jl_union_field_idxs.iter();
        let jl_non_union_field_idxs = classified_fields.jl_non_union_field_idxs.iter();

        let n_fields = classified_fields.jl_union_field_idxs.len()
            + classified_fields.jl_non_union_field_idxs.len();

        let valid_layout_impl = quote! {
            unsafe impl #generics ::jlrs::data::layout::valid_layout::ValidLayout for #name #generics #where_clause {
                fn valid_layout(v: ::jlrs::data::managed::value::Value) -> bool {
                    unsafe {
                        if v.is::<::jlrs::data::managed::datatype::DataType>() {
                            let dt = unsafe { v.cast_unchecked::<::jlrs::data::managed::datatype::DataType>() };
                            if dt.n_fields().unwrap() as usize != #n_fields {
                                return false;
                            }

                            let field_types = dt.field_types();
                            let field_types_data = field_types.data();
                            let field_types = field_types_data.as_atomic_slice().assume_immutable_non_null();

                            #(
                                if !<#rs_non_union_fields as ::jlrs::data::layout::valid_layout::ValidField>::valid_field(field_types[#jl_non_union_field_idxs]) {
                                    return false;
                                }
                            )*

                            #(
                                {
                                    let field_type = field_types[#jl_union_field_idxs];
                                    if field_type.is::<::jlrs::data::managed::union::Union>() {
                                        let u = field_type.cast_unchecked::<::jlrs::data::managed::union::Union>();
                                        if !::jlrs::data::layout::union::correct_layout_for::<#rs_align_fields, #rs_union_fields, #rs_flag_fields>(u) {
                                            return false
                                        }
                                    } else {
                                        return false
                                    }
                                }
                            )*

                            return true;
                        }
                    }

                    false
                }

                #[inline]
                fn type_object<'target, Tgt>(
                    target: &Tgt
                ) -> ::jlrs::data::managed::value::Value<'target, 'static>
                where
                    Tgt: ::jlrs::memory::target::Target<'target>,
                {
                    unsafe {
                        ::jlrs::data::managed::module::Module::typed_global_cached::<::jlrs::data::managed::value::Value, _, _>(target, #jl_type).unwrap()
                    }
                }

                const IS_REF: bool = false;
            }
        };

        Ok(valid_layout_impl.into())
    } else {
        let valid_layout_impl = quote! {
            unsafe impl #generics ::jlrs::data::layout::valid_layout::ValidLayout for #name #generics #where_clause {
                fn valid_layout(v: ::jlrs::data::managed::value::Value) -> bool {
                    unsafe {
                        if v.is::<::jlrs::data::managed::datatype::DataType>() {
                            let dt = v.cast_unchecked::<::jlrs::data::managed::datatype::DataType>();
                            let target = <::jlrs::data::managed::datatype::DataType as ::jlrs::data::managed::Managed>::unrooted_target(dt);
                            let ct = <Self as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&target).as_value();

                            return dt == ct;
                        }
                    }

                    false
                }

                #[inline]
                fn type_object<'target, Tgt>(
                    target: &Tgt
                ) -> ::jlrs::data::managed::value::Value<'target, 'static>
                where
                    Tgt: ::jlrs::memory::target::Target<'target>,
                {
                    unsafe {
                        ::jlrs::data::managed::module::Module::typed_global_cached::<::jlrs::data::managed::value::Value, _, _>(target, #jl_type).unwrap()
                    }
                }

                const IS_REF: bool = false;
            }
        };

        Ok(valid_layout_impl.into())
    }
}

#[derive(Default)]
struct ClassifiedFields<'a> {
    rs_flag_fields: Vec<&'a syn::Type>,
    rs_align_fields: Vec<&'a syn::Type>,
    rs_union_fields: Vec<&'a syn::Type>,
    rs_non_union_fields: Vec<&'a syn::Type>,
    jl_union_field_idxs: Vec<usize>,
    jl_non_union_field_idxs: Vec<usize>,
}

impl<'a> ClassifiedFields<'a> {
    fn classify<I>(fields_iter: I) -> Self
    where
        I: Iterator<Item = &'a syn::Field> + ExactSizeIterator,
    {
        let mut rs_flag_fields = vec![];
        let mut rs_align_fields = vec![];
        let mut rs_union_fields = vec![];
        let mut rs_non_union_fields = vec![];
        let mut jl_union_field_idxs = vec![];
        let mut jl_non_union_field_idxs = vec![];
        let mut offset = 0;

        'outer: for (idx, field) in fields_iter.enumerate() {
            for attr in &field.attrs {
                match JlrsFieldAttr::parse(attr) {
                    Some(JlrsFieldAttr::BitsUnion) => {
                        rs_union_fields.push(&field.ty);
                        jl_union_field_idxs.push(idx - offset);
                        continue 'outer;
                    }
                    Some(JlrsFieldAttr::BitsUnionAlign) => {
                        rs_align_fields.push(&field.ty);
                        offset += 1;
                        continue 'outer;
                    }
                    Some(JlrsFieldAttr::BitsUnionFlag) => {
                        rs_flag_fields.push(&field.ty);
                        offset += 1;
                        continue 'outer;
                    }
                    _ => (),
                }
            }

            rs_non_union_fields.push(&field.ty);
            jl_non_union_field_idxs.push(idx - offset);
        }

        ClassifiedFields {
            rs_flag_fields,
            rs_align_fields,
            rs_union_fields,
            rs_non_union_fields,
            jl_union_field_idxs,
            jl_non_union_field_idxs,
        }
    }
}

enum JlrsFieldAttr {
    BitsUnionAlign,
    BitsUnion,
    BitsUnionFlag,
}

impl JlrsFieldAttr {
    pub fn parse(attr: &syn::Attribute) -> Option<Self> {
        if attr.path().is_ident("jlrs") {
            let nested = attr
                .parse_args_with(
                    syn::punctuated::Punctuated::<syn::Meta, Token![,]>::parse_terminated,
                )
                .unwrap();
            for meta in nested {
                let syn::Meta::Path(path) = meta else {
                    return None;
                };

                if path.is_ident("bits_union") {
                    return Some(JlrsFieldAttr::BitsUnion);
                } else if path.is_ident("bits_union_align") {
                    return Some(JlrsFieldAttr::BitsUnionAlign);
                } else if path.is_ident("bits_union_flag") {
                    return Some(JlrsFieldAttr::BitsUnionFlag);
                }
            }
        }

        None
    }
}
