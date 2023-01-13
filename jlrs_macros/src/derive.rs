use proc_macro::TokenStream;
use proc_macro2::TokenStream as TS2;
use quote::quote;
use syn::{self, punctuated::Punctuated, token::Comma, Meta, WherePredicate};

#[derive(Default)]
pub struct ClassifiedFields<'a> {
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

pub struct JlrsTypeAttrs {
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

pub fn impl_into_julia(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    if !is_repr_c(ast) {
        panic!("IntoJulia can only be derived for types with the attribute #[repr(C)].");
    }

    let mut attrs = JlrsTypeAttrs::parse(ast);
    let jl_type = attrs.julia_type
        .take()
        .expect("IntoJulia can only be derived if the corresponding Julia type is set with #[julia_type = \"Main.MyModule.Submodule.StructType\"]");

    let mut type_it = jl_type.split('.');
    let func: syn::Expr = match type_it.next() {
        Some("Main") => syn::parse_quote! { 
            {
                ::jlrs::data::managed::module::Module::main(&global)
            }
        },
        Some("Base") => syn::parse_quote! { 
            {
                ::jlrs::data::managed::module::Module::base(&global)
            }
        },
        Some("Core") => syn::parse_quote! { 
            {
                ::jlrs::data::managed::module::Module::corr(&global)
            }
        },
        Some(pkg) => syn::parse_quote! {
            {
                let module = ::jlrs::data::managed::module::Module::package_root_module(&global, #pkg);
                match module {
                    Some(module) => module,
                    _ => panic!("Package {} cannot be found", #pkg)
                }
            }
        },
        _ => panic!("IntoJulia can only be derived if the first module of \"julia_type\" is either \"Main\", \"Base\" or \"Core\", or a package name."),
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
                    #func
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

pub fn impl_into_julia_fn(attrs: &JlrsTypeAttrs) -> Option<TS2> {
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

pub fn impl_unbox(ast: &syn::DeriveInput) -> TokenStream {
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

pub fn impl_typecheck(ast: &syn::DeriveInput) -> TokenStream {
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

pub fn impl_construct_type(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    if !is_repr_c(ast) {
        panic!("ConstructType can only be derived for types with the attribute #[repr(C)].");
    }
    let mut attrs = JlrsTypeAttrs::parse(ast);
    let jl_type = attrs.julia_type
        .take()
        .expect("ConstructType can only be derived if the corresponding Julia type is set with #[julia_type = \"Main.MyModule.Submodule.StructType\"]");

    let mut type_it = jl_type.split('.');
    let func: syn::Expr = match type_it.next() {
        Some("Main") => syn::parse_quote! { 
            {
                ::jlrs::data::managed::module::Module::main(target)
            }
        },
        Some("Base") => syn::parse_quote! { 
            {
                ::jlrs::data::managed::module::Module::base(target)
            }
        },
        Some("Core") => syn::parse_quote! { 
            {
                ::jlrs::data::managed::module::Module::corr(target)
            }
        },
        Some(pkg) => syn::parse_quote! {
            {
                let module = ::jlrs::data::managed::module::Module::package_root_module(target, #pkg);
                match module {
                    Some(module) => module,
                    _ => panic!("Package {} cannot be found", #pkg)
                }
            }
        },
        _ => panic!("ConstructType can only be derived if the first module of \"julia_type\" is either \"Main\", \"Base\" or \"Core\", or a package name."),
    };

    let mut modules = type_it.collect::<Vec<_>>();
    let ty = modules.pop().expect("ConstructType can only be derived if the corresponding Julia type is set with #[jlrs(julia_type = \"Main.MyModule.Submodule.StructType\")]");
    let modules_it = modules.iter();
    let modules_it_b = modules_it.clone();

    let generics = &ast.generics;
    let wc = match ast.generics.where_clause.as_ref() {
        Some(wc) => {
            let mut wc = wc.clone();
            for generic in generics.type_params() {
                let clause: WherePredicate = syn::parse_quote! {
                    #generic: ::jlrs::convert::construct_type::ConstructType
                };
                wc.predicates.push(clause)
            }
            wc
        }
        None => {
            let mut predicates = Punctuated::<_, Comma>::new();
            for generic in generics.type_params() {
                let clause: WherePredicate = syn::parse_quote! {
                    #generic: ::jlrs::convert::construct_type::ConstructType
                };
                predicates.push(clause)
            }

            syn::parse_quote! {
                where #predicates
            }
        }
    };

    let param_names = ast
        .generics
        .type_params()
        .map(|p| &p.ident)
        .collect::<Vec<_>>();
    let n_generics = param_names.len();
    let param_names = param_names.iter();
    let nth_generic = 0..n_generics;

    let construct_type_impl = quote! {
        unsafe impl #generics ::jlrs::convert::construct_type::ConstructType for #name #generics #wc {
            fn base_type<'target, TARGET>(target: &TARGET) -> ::jlrs::data::managed::value::Value<'target, 'static>
            where
                TARGET: ::jlrs::memory::target::Target<'target>,
            {
                unsafe {
                        #func
                        #(
                            .submodule(target, #modules_it)
                            .expect(&format!("Submodule {} cannot be found", #modules_it_b))
                            .as_managed()
                        )*
                        .global(target, #ty)
                        .expect(&format!("Type {} cannot be found in module", #ty))
                        .as_value()
                }
            }

            fn construct_type<'target, 'current, 'borrow, TARGET>(
                target: ::jlrs::memory::target::ExtendedTarget<'target, 'current, 'borrow, TARGET>,
            ) -> ::jlrs::data::managed::datatype::DataTypeData<'target, TARGET>
            where
                TARGET: ::jlrs::memory::target::Target<'target>,
            {
                let (target, frame) = target.split();

                frame.scope(|mut frame| {
                    let base_type = Self::base_type(&frame);
                    if let Ok(ty) = base_type.cast::<::jlrs::data::managed::datatype::DataType>() {
                        Ok(ty.root(target))
                    } else if let Ok(ua) = base_type.cast::<::jlrs::data::managed::union_all::UnionAll>() {
                        let mut types: [Option<::jlrs::data::managed::value::Value>; #n_generics] = [None; #n_generics];
                        #(
                            types[#nth_generic] = Some(<#param_names as ::jlrs::convert::construct_type::ConstructType>::construct_type(frame.as_extended_target()).as_value());
                        )*
                        unsafe {
                            let types = std::mem::transmute::<&[Option<::jlrs::data::managed::value::Value>; #n_generics], &[::jlrs::data::managed::value::Value; #n_generics]>(&types);
                            let applied = ua
                                .apply_types_unchecked(&target, types)
                                .as_value()
                                .cast::<::jlrs::data::managed::datatype::DataType>()
                                .expect("UnionAll is not a DataType after applying generic types")
                                .root(target);

                            Ok(applied)
                        }
                    } else {
                        panic!("Type is neither a DataType or UnionAll")
                    }
                }).unwrap()
            }
        }
    };

    construct_type_impl.into()
}

pub fn impl_valid_layout(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    if !is_repr_c(ast) {
        panic!("ValidLayout can only be derived for types with the attribute #[repr(C)].");
    }

    let generics = &ast.generics;
    let where_clause = &ast.generics.where_clause;

    let fields = match &ast.data {
        syn::Data::Struct(s) => &s.fields,
        _ => panic!("ValidLayout can only be derived for structs."),
    };

    let classified_fields = match fields {
        syn::Fields::Named(n) => ClassifiedFields::classify(n.named.iter()),
        syn::Fields::Unit => ClassifiedFields::default(),
        _ => panic!("ValidLayout cannot be derived for tuple structs."),
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

pub fn impl_valid_field(ast: &syn::DeriveInput) -> TokenStream {
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

pub fn impl_ccall_arg(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    if !is_repr_c(ast) {
        panic!("ValidLayout can only be derived for types with the attribute #[repr(C)].");
    }

    let generics = &ast.generics;
    let wc = match ast.generics.where_clause.as_ref() {
        Some(wc) => {
            let mut wc = wc.clone();
            for generic in generics.type_params() {
                let clause: WherePredicate = syn::parse_quote! {
                    #generic: ::jlrs::convert::construct_type::ConstructType
                };
                wc.predicates.push(clause)
            }
            wc
        }
        None => {
            let mut predicates = Punctuated::<_, Comma>::new();
            for generic in generics.type_params() {
                let clause: WherePredicate = syn::parse_quote! {
                    #generic: ::jlrs::convert::construct_type::ConstructType
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

        unsafe impl #generics ::jlrs::convert::ccall_types::CCallReturn for #name #generics #wc {
            type CCallReturnType = Self;
            type FunctionReturnType = Self;
        }
    };

    ccall_arg_impl.into()
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
