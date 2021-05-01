extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{self, Meta};

use syn::visit_mut::VisitMut;
struct MissingLifetimes(Vec<String>);

impl VisitMut for MissingLifetimes {
    fn visit_generics_mut(&mut self, def: &mut syn::Generics) {
        for s in self.0.iter() {
            let gp = syn::GenericParam::Lifetime(syn::LifetimeDef::new(syn::Lifetime::new(
                s,
                ::proc_macro2::Span::call_site(),
            )));
            def.params.insert(0, gp);
        }
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
        I: Iterator<Item = &'a syn::Field> + ExactSizeIterator + Clone,
    {
        let mut rs_flag_fields = vec![];
        let mut rs_align_fields = vec![];
        let mut rs_union_fields = vec![];
        let mut rs_non_union_fields = vec![];
        let mut jl_union_field_idxs = vec![];
        let mut jl_non_union_field_idxs = vec![];
        let mut offset = 0;

        'outer: for (idx, field) in fields_iter.enumerate() {
            for attr in field.attrs.iter() {
                match JlrsAttr::parse(attr) {
                    Some(JlrsAttr::BitsUnion) => {
                        rs_union_fields.push(&field.ty);
                        jl_union_field_idxs.push(idx - offset);
                        continue 'outer;
                    }
                    Some(JlrsAttr::BitsUnionAlign) => {
                        rs_align_fields.push(&field.ty);
                        offset += 1;
                        continue 'outer;
                    }
                    Some(JlrsAttr::BitsUnionFlag) => {
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

#[proc_macro_derive(IntoJulia)]
pub fn into_julia_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_into_julia(&ast)
}

#[proc_macro_derive(JuliaStruct, attributes(jlrs))]
pub fn julia_struct_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_julia_struct(&ast)
}

fn impl_julia_struct(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    if !is_repr_c(ast) {
        panic!("JuliaStruct can only be derived for types with the attribute #[repr(C)].");
    }

    let generics = &ast.generics;
    let jl_type = corresponding_julia_type(ast).expect("JuliaStruct can only be derived if the corresponding Julia type is set with #[julia_type = \"Main.MyModule.Submodule.StructType\"]");
    let mut type_it = jl_type.split('.');
    let func = match type_it.next() {
        Some("Main") => quote::format_ident!("main"),
        Some("Base") => quote::format_ident!("base"),
        Some("Core") => quote::format_ident!("core"),
        _ => panic!("JuliaStruct can only be derived if the first module of \"julia_type\" is either \"Main\", \"Base\" or \"Core\"."),
    };

    let mut modules = type_it.collect::<Vec<_>>();
    let ty = modules.pop().expect("JuliaStruct can only be derived if the corresponding Julia type is set with #[jlrs(julia_type = \"Main.MyModule.Submodule.StructType\")]");
    let modules_it = modules.iter();
    let modules_it_b = modules_it.clone();

    let mut missing_lifetimes = MissingLifetimes(Vec::with_capacity(2));

    let data_lt = generics
        .lifetimes()
        .find(|l| l.lifetime.ident.to_string() == "data");
    if data_lt.is_none() {
        missing_lifetimes.0.push("'data".into());
    }
    let frame_lt = generics
        .lifetimes()
        .find(|l| l.lifetime.ident.to_string() == "frame");
    if frame_lt.is_none() {
        missing_lifetimes.0.push("'frame".into());
    }

    let mut extended_generics = generics.clone();
    missing_lifetimes.visit_generics_mut(&mut extended_generics);

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

    let julia_struct_impl = quote! {
        unsafe impl #generics ::jlrs::layout::valid_layout::ValidLayout for #name #generics #where_clause {
            unsafe fn valid_layout(v: ::jlrs::value::Value) -> bool {
                if let Ok(dt) = v.cast::<DataType>() {
                    if dt.nfields() as usize != #n_fields {
                        return false;
                    }

                    let field_types = dt.field_types().data();

                    #(
                        if !<#rs_non_union_fields as ::jlrs::layout::valid_layout::ValidLayout>::valid_layout(field_types[#jl_non_union_field_idxs].assume_valid_unchecked()) {
                            return false;
                        }
                    )*

                    #(
                        if let Ok(u) = field_types[#jl_union_field_idxs].assume_valid_unchecked().cast::<::jlrs::value::union::Union>() {
                            if !::jlrs::value::union::correct_layout_for::<#rs_align_fields, #rs_union_fields, #rs_flag_fields>(u) {
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

        unsafe impl #generics ::jlrs::layout::julia_typecheck::JuliaTypecheck for #name #generics #where_clause {
            unsafe fn julia_typecheck(t: ::jlrs::value::datatype::DataType) -> bool {
                <Self as ::jlrs::layout::valid_layout::ValidLayout>::valid_layout(t.into())
            }
        }

        unsafe impl #generics ::jlrs::layout::julia_type::JuliaType for #name #generics #where_clause {
            unsafe fn julia_type() -> *mut ::jlrs::jl_sys_export::jl_datatype_t {
                let global = ::jlrs::memory::global::Global::new();

                let julia_type = ::jlrs::value::module::Module::#func(global)
                    #(.submodule(#modules_it).expect(&format!("Submodule {} cannot be found", #modules_it_b)))*
                    .global(#ty).expect(&format!("Type {} cannot be found in module", #ty));

                if let Ok(dt) = julia_type.cast::<::jlrs::value::datatype::DataType>() {
                    dt.inner().as_ptr()
                } else if let Ok(ua) = julia_type.cast::<::jlrs::value::union_all::UnionAll>() {
                    ua.base_type().assume_valid_unchecked().inner().as_ptr()
                } else {
                    panic!("Invalid type: {:?}", julia_type.datatype());
                }
            }
        }

        unsafe impl #extended_generics ::jlrs::convert::cast::Cast<'frame, 'data> for #name #generics #where_clause {
            type Output = Self;

            fn cast(value: ::jlrs::value::Value<'frame, 'data>) -> ::jlrs::error::JlrsResult<Self::Output> {
                if value.is::<::jlrs::value::datatype::Nothing>() {
                    Err(::jlrs::error::JlrsError::Nothing)?
                }

                unsafe {
                    if <Self as ::jlrs::layout::valid_layout::ValidLayout>::valid_layout(value.datatype().into()) {
                        return Ok(Self::cast_unchecked(value));
                    }
                }

                Err(::jlrs::error::JlrsError::WrongType)?
            }

            unsafe fn cast_unchecked(value: ::jlrs::value::Value<'frame, 'data>) -> Self::Output {
                *(value.inner().as_ptr().cast::<Self::Output>())
            }
        }
    };

    julia_struct_impl.into()
}

fn impl_into_julia(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    if !is_repr_c(ast) {
        panic!("IntoJulia can only be derived for types with the attribute #[repr(C)].");
    }

    let into_julia_impl = quote! {
        unsafe impl ::jlrs::convert::into_julia::IntoJulia for #name {
            unsafe fn into_julia(&self) -> *mut ::jlrs::jl_sys_export::jl_value_t {
                let ty = <Self as ::jlrs::layout::julia_type::JuliaType>::julia_type();
                let container = ::jlrs::jl_sys_export::jl_new_struct_uninit(ty.cast());
                let data: *mut Self = container.cast();
                ::std::ptr::write(data, *self);

                container
            }
        }
    };

    into_julia_impl.into()
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

fn corresponding_julia_type(ast: &syn::DeriveInput) -> Option<String> {
    for attr in &ast.attrs {
        if attr.path.is_ident("jlrs") {
            if let Ok(Meta::List(p)) = attr.parse_meta() {
                if let syn::NestedMeta::Meta(syn::Meta::NameValue(nv)) = p.nested.first().unwrap() {
                    if nv.path.is_ident("julia_type") {
                        if let syn::Lit::Str(string) = &nv.lit {
                            return Some(string.value());
                        }
                    }
                }
            }
        }
    }

    None
}

enum JlrsAttr {
    Rename(String),
    Type(String),
    BitsUnionAlign,
    BitsUnion,
    BitsUnionFlag,
}

impl JlrsAttr {
    pub fn parse(attr: &syn::Attribute) -> Option<Self> {
        if let Ok(Meta::List(p)) = attr.parse_meta() {
            if let syn::NestedMeta::Meta(syn::Meta::NameValue(nv)) = p.nested.first().unwrap() {
                if nv.path.is_ident("rename") {
                    if let syn::Lit::Str(string) = &nv.lit {
                        return Some(JlrsAttr::Rename(string.value()));
                    }
                }

                if nv.path.is_ident("julia_type") {
                    if let syn::Lit::Str(string) = &nv.lit {
                        return Some(JlrsAttr::Type(string.value()));
                    }
                }

                return None;
            }

            if let Some(syn::NestedMeta::Meta(syn::Meta::Path(m))) = p.nested.first() {
                if m.is_ident("bits_union") {
                    return Some(JlrsAttr::BitsUnion);
                }

                if m.is_ident("bits_union_align") {
                    return Some(JlrsAttr::BitsUnionAlign);
                }

                if m.is_ident("bits_union_flag") {
                    return Some(JlrsAttr::BitsUnionFlag);
                }
            }
        }

        None
    }
}
