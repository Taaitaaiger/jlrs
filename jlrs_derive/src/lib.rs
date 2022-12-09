extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TS2;
use quote::quote;
use syn::{self, Meta};

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

struct JlrsTypeAttrs {
    julia_type: Option<String>,
    zst: bool,
}

impl JlrsTypeAttrs {
    fn parse(ast: &syn::DeriveInput) -> Self {
        let mut julia_type = None;
        let mut zst = false;
        for attr in &ast.attrs {
            if attr.path.is_ident("jlrs") {
                if let Ok(Meta::List(p)) = attr.parse_meta() {
                    for item in &p.nested {
                        match item {
                            syn::NestedMeta::Meta(Meta::NameValue(nv)) => {
                                if nv.path.is_ident("julia_type") {
                                    if let syn::Lit::Str(string) = &nv.lit {
                                        julia_type = Some(string.value())
                                    }
                                }
                            }
                            syn::NestedMeta::Meta(Meta::Path(pt)) => {
                                if pt.is_ident("zst") {
                                    zst = true;
                                }
                            }
                            _ => continue,
                        }
                    }
                }
            }
        }

        JlrsTypeAttrs { julia_type, zst }
    }
}

enum JlrsFieldAttr {
    BitsUnionAlign,
    BitsUnion,
    BitsUnionFlag,
}

impl JlrsFieldAttr {
    pub fn parse(attr: &syn::Attribute) -> Option<Self> {
        if let Ok(Meta::List(p)) = attr.parse_meta() {
            if let Some(syn::NestedMeta::Meta(syn::Meta::Path(m))) = p.nested.first() {
                if m.is_ident("bits_union") {
                    return Some(JlrsFieldAttr::BitsUnion);
                }

                if m.is_ident("bits_union_align") {
                    return Some(JlrsFieldAttr::BitsUnionAlign);
                }

                if m.is_ident("bits_union_flag") {
                    return Some(JlrsFieldAttr::BitsUnionFlag);
                }
            }
        }

        None
    }
}

#[proc_macro_derive(IntoJulia, attributes(jlrs))]
pub fn into_julia_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_into_julia(&ast)
}

#[proc_macro_derive(Unbox, attributes(jlrs))]
pub fn unbox_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_unbox(&ast)
}

#[proc_macro_derive(Typecheck, attributes(jlrs))]
pub fn typecheck_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_typecheck(&ast)
}

#[proc_macro_derive(ValidLayout, attributes(jlrs))]
pub fn valid_layout_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_valid_layout(&ast)
}

#[proc_macro_derive(ValidField, attributes(jlrs))]
pub fn valid_field_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_valid_field(&ast)
}

fn impl_into_julia(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    if !is_repr_c(ast) {
        panic!("IntoJulia can only be derived for types with the attribute #[repr(C)].");
    }

    let mut attrs = JlrsTypeAttrs::parse(ast);
    let jl_type = attrs.julia_type
        .take()
        .expect("IntoJulia can only be derived if the corresponding Julia type is set with #[julia_type = \"Main.MyModule.Submodule.StructType\"]");

    let mut type_it = jl_type.split('.');
    let func = match type_it.next() {
        Some("Main") => quote::format_ident!("main"),
        Some("Base") => quote::format_ident!("base"),
        Some("Core") => quote::format_ident!("core"),
        _ => panic!("IntoJulia can only be derived if the first module of \"julia_type\" is either \"Main\", \"Base\" or \"Core\"."),
    };

    let mut modules = type_it.collect::<Vec<_>>();
    let ty = modules.pop().expect("IntoJulia can only be derived if the corresponding Julia type is set with #[jlrs(julia_type = \"Main.MyModule.Submodule.StructType\")]");
    let modules_it = modules.iter();
    let modules_it_b = modules_it.clone();

    let into_julia_fn = impl_into_julia_fn(&attrs);

    let into_julia_impl = quote! {
        unsafe impl ::jlrs::convert::into_julia::IntoJulia for #name {
            fn julia_type<'scope, T>(target: T) -> ::jlrs::data::managed::datatype::DataTypeData<'scope, T>
            where
                T: ::jlrs::memory::target::Target<'scope>,
            {
                unsafe {
                    let global = target.unrooted();
                    ::jlrs::data::managed::module::Module::#func(&global)
                        #(
                            .submodule(&global, #modules_it)
                            .expect(&format!("Submodule {} cannot be found", #modules_it_b))
                            .as_managed()
                        )*
                        .global(&global, #ty)
                        .expect(&format!("Type {} cannot be found in module", #ty))
                        .as_value()
                        .cast::<::jlrs::data::managed::datatype::DataType>()
                        .expect("Type is not a DataType")
                        .root(target)
                }
            }

            #into_julia_fn
        }
    };

    into_julia_impl.into()
}

fn impl_into_julia_fn(attrs: &JlrsTypeAttrs) -> Option<TS2> {
    if attrs.zst {
        Some(quote! {
            unsafe fn into_julia<'target, T>(self, target: T) -> ::jlrs::data::managed::value::ValueData<'target, 'static, T>
            where
                T: ::jlrs::memory::target::Target<'scope>,
            {
                let ty = self.julia_type(global);
                unsafe {
                    ty.as_managed()
                        .instance()
                        .as_value()
                        .expect("Instance is undefined")
                        .as_ref()
                }
            }
        })
    } else {
        None
    }
}

fn impl_unbox(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    if !is_repr_c(ast) {
        panic!("Unbox can only be derived for types with the attribute #[repr(C)].");
    }

    let generics = &ast.generics;
    let where_clause = &ast.generics.where_clause;

    let unbox_impl = quote! {
        unsafe impl #generics ::jlrs::convert::unbox::Unbox for #name #generics #where_clause {
            type Output = Self;
        }
    };

    unbox_impl.into()
}

fn impl_typecheck(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    if !is_repr_c(ast) {
        panic!("Typecheck can only be derived for types with the attribute #[repr(C)].");
    }

    let generics = &ast.generics;
    let where_clause = &ast.generics.where_clause;

    let typecheck_impl = quote! {
        unsafe impl #generics ::jlrs::data::managed::typecheck::Typecheck for #name #generics #where_clause {
            fn typecheck(dt: ::jlrs::data::managed::datatype::DataType) -> bool {
                <Self as ::jlrs::data::layout::valid_layout::ValidLayout>::valid_layout(dt.as_value())
            }
        }
    };

    typecheck_impl.into()
}

fn impl_valid_layout(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    if !is_repr_c(ast) {
        panic!("ValidLayout can only be derived for types with the attribute #[repr(C)].");
    }

    let generics = &ast.generics;
    let where_clause = &ast.generics.where_clause;

    let fields = match &ast.data {
        syn::Data::Struct(s) => &s.fields,
        _ => panic!("Julia struct can only be derived for structs."),
    };

    let classified_fields = match fields {
        syn::Fields::Named(n) => ClassifiedFields::classify(n.named.iter()),
        syn::Fields::Unit => ClassifiedFields::default(),
        _ => panic!("Julia struct cannot be derived for tuple structs."),
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
                    if let Ok(dt) = v.cast::<::jlrs::data::managed::datatype::DataType>() {
                        if dt.n_fields() as usize != #n_fields {
                            return false;
                        }

                        let global = v.unrooted_target();
                        let field_types = dt.field_types(global);
                        let field_types_svec = field_types.as_managed();
                        let field_types_data = field_types_svec.data();
                        let field_types = field_types_data.as_slice();

                        #(
                            if !<#rs_non_union_fields as ::jlrs::data::layout::valid_layout::ValidField>::valid_field(field_types[#jl_non_union_field_idxs].unwrap().as_managed()) {
                                return false;
                            }
                        )*

                        #(
                            if let Ok(u) = field_types[#jl_union_field_idxs].unwrap().as_managed().cast::<::jlrs::data::managed::union::Union>() {
                                if !::jlrs::data::layout::union::correct_layout_for::<#rs_align_fields, #rs_union_fields, #rs_flag_fields>(u) {
                                    return false
                                }
                            } else {
                                return false
                            }
                        )*


                        return true;
                    }
                }

                false
            }

            const IS_REF: bool = false;
        }
    };

    valid_layout_impl.into()
}

fn impl_valid_field(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    if !is_repr_c(ast) {
        panic!("ValidLayout can only be derived for types with the attribute #[repr(C)].");
    }

    let generics = &ast.generics;
    let where_clause = &ast.generics.where_clause;

    let valid_field_impl = quote! {
        unsafe impl #generics ::jlrs::data::layout::valid_layout::ValidField for #name #generics #where_clause {
            fn valid_field(v: ::jlrs::data::managed::value::Value) -> bool {
                <Self as ::jlrs::data::layout::valid_layout::ValidLayout>::valid_layout(v)
            }
        }
    };

    valid_field_impl.into()
}

fn is_repr_c(ast: &syn::DeriveInput) -> bool {
    for attr in &ast.attrs {
        if attr.path.is_ident("repr") {
            if let Ok(Meta::List(p)) = attr.parse_meta() {
                if let Some(syn::NestedMeta::Meta(syn::Meta::Path(m))) = p.nested.first() {
                    if m.is_ident("C") {
                        return true;
                    }
                }
            }
        }
    }

    false
}
