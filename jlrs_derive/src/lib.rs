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

#[proc_macro_derive(IntoJulia)]
pub fn into_julia_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_into_julia(&ast)
}

#[proc_macro_derive(Unbox)]
pub fn unbox_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_unbox(&ast)
}

#[proc_macro_derive(Typecheck)]
pub fn typecheck_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_typecheck(&ast)
}

#[proc_macro_derive(ValidLayout)]
pub fn valid_layout_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_valid_layout(&ast)
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
            fn julia_type<'target>(global: ::jlrs::memory::global::Global<'target>) -> ::jlrs::wrapper::ptr::DataTypeRef<'target> {
                unsafe {
                    ::jlrs::wrappers::ptr::module::Module::#func(global)
                        #(
                            .submodule_ref(#modules_it)
                            .expect(&format!("Submodule {} cannot be found", #modules_it_b))
                            .wrapper_unchecked()
                        )*
                        .global_ref(#ty)
                        .expect(&format!("Type {} cannot be found in module", #ty))
                        .value_unchecked()
                        .cast::<::jlrs::wrapper::ptr::datatype::DataType>()
                        .expect("Type is not a DataType")
                        .as_ref()
                }
            }

            #into_julia_fn
        }
    };

    into_julia_impl.into()
}

fn impl_into_julia_fn(attrs: &JlrsTypeAttrs) -> TS2 {
    if attrs.zst {
        quote! {
            unsafe fn into_julia<'target>(self, global: ::jlrs::memory::global::Global<'target>) -> ::jlrs::wrapper::ptr::ValueRef<'target, 'static> {
                let ty = self.julia_type(global);
                unsafe {
                    ty.wrapper_unchecked()
                        .instance()
                        .value()
                        .expect("Instance is undefined")
                        .as_ref()
                }
            }
        }
    } else {
        quote! {}
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
        unsafe impl #generics ::jlrs::layout::typecheck::Typecheck for #name #generics #where_clause {
            fn typecheck(dt: ::jlrs::wrappers::ptr::DataType) -> bool {
                <Self as ::jlrs::layout::valid_layout::ValidLayout>::valid_layout(t.as_value())
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
        unsafe impl #generics ::jlrs::layout::valid_layout::ValidLayout for #name #generics #where_clause {
            unsafe fn valid_layout(v: ::jlrs::wrappers::ptr::value::Value) -> bool {
                if let Ok(dt) = v.cast::<::jlrs::wrappers::ptr::datatype::DataType>() {
                    if dt.nfields() as usize != #n_fields {
                        return false;
                    }

                    let field_types = dt.field_types().data();

                    #(
                        if !<#rs_non_union_fields as ::jlrs::layout::valid_layout::ValidLayout>::valid_layout(field_types[#jl_non_union_field_idxs].wrapper_unchecked()) {
                            return false;
                        }
                    )*

                    #(
                        if let Ok(u) = field_types[#jl_union_field_idxs].wrapper_unchecked().cast::<::jlrs::wrappers::builtin::union::Union>() {
                            if !::jlrs::wrappers::inline::union::correct_layout_for::<#rs_align_fields, #rs_union_fields, #rs_flag_fields>(u) {
                                return false
                            }
                        } else {
                            return false
                        }
                    )*


                    return true;
                }

                false
            }
        }
    };

    valid_layout_impl.into()
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
