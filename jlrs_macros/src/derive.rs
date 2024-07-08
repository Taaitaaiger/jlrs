use proc_macro::TokenStream;
use proc_macro2::TokenStream as TS2;
use quote::{format_ident, quote};
use syn::{self, punctuated::Punctuated, token::Comma, Token};

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
    constructor_for: Option<String>,
    zst: bool,
    scope_lifetime: bool,
    data_lifetime: bool,
    layout_params: Vec<String>,
    elided_params: Vec<String>,
    all_params: Vec<String>,
}

impl JlrsTypeAttrs {
    fn parse(ast: &syn::DeriveInput) -> Self {
        let mut julia_type: Option<String> = None;
        let mut constructor_for: Option<String> = None;
        let mut scope_lifetime = false;
        let mut data_lifetime = false;
        let mut layout_params = Vec::new();
        let mut elided_params = Vec::new();
        let mut all_params = Vec::new();
        let mut zst = false;

        for attr in &ast.attrs {
            if attr.path().is_ident("jlrs") {
                let nested = attr
                    .parse_args_with(Punctuated::<syn::Meta, Token![,]>::parse_terminated)
                    .unwrap();
                for meta in nested {
                    match meta {
                        syn::Meta::Path(path) if path.is_ident("zero_sized_type") => {
                            zst = true;
                        }
                        syn::Meta::NameValue(mnv) if mnv.path.is_ident("julia_type") => {
                            if let syn::Expr::Lit(lit) = mnv.value {
                                if let syn::Lit::Str(s) = lit.lit {
                                    julia_type = Some(s.value());
                                }
                            }
                        }
                        syn::Meta::NameValue(mnv) if mnv.path.is_ident("constructor_for") => {
                            if let syn::Expr::Lit(lit) = mnv.value {
                                if let syn::Lit::Str(s) = lit.lit {
                                    constructor_for = Some(s.value());
                                }
                            }
                        }
                        syn::Meta::NameValue(mnv) if mnv.path.is_ident("scope_lifetime") => {
                            if let syn::Expr::Lit(lit) = mnv.value {
                                if let syn::Lit::Bool(b) = lit.lit {
                                    scope_lifetime = b.value;
                                }
                            }
                        }
                        syn::Meta::NameValue(mnv) if mnv.path.is_ident("data_lifetime") => {
                            if let syn::Expr::Lit(lit) = mnv.value {
                                if let syn::Lit::Bool(b) = lit.lit {
                                    data_lifetime = b.value;
                                }
                            }
                        }
                        syn::Meta::NameValue(mnv) if mnv.path.is_ident("layout_params") => {
                            if let syn::Expr::Array(arr) = mnv.value {
                                let tys = arr.elems.iter().filter_map(|x| match x {
                                    syn::Expr::Lit(lit) => {
                                        if let syn::Lit::Str(ref s) = lit.lit {
                                            Some(s.value())
                                        } else {
                                            None
                                        }
                                    }
                                    _ => None,
                                });

                                layout_params.extend(tys)
                            }
                        }
                        syn::Meta::NameValue(mnv) if mnv.path.is_ident("elided_params") => {
                            if let syn::Expr::Array(arr) = mnv.value {
                                let tys = arr.elems.iter().filter_map(|x| match x {
                                    syn::Expr::Lit(lit) => {
                                        if let syn::Lit::Str(ref s) = lit.lit {
                                            Some(s.value())
                                        } else {
                                            None
                                        }
                                    }
                                    _ => None,
                                });

                                elided_params.extend(tys)
                            }
                        }
                        syn::Meta::NameValue(mnv) if mnv.path.is_ident("all_params") => {
                            if let syn::Expr::Array(arr) = mnv.value {
                                let tys = arr.elems.iter().filter_map(|x| match x {
                                    syn::Expr::Lit(lit) => {
                                        if let syn::Lit::Str(ref s) = lit.lit {
                                            Some(s.value())
                                        } else {
                                            None
                                        }
                                    }
                                    _ => None,
                                });

                                all_params.extend(tys)
                            }
                        }
                        _ => (),
                    }
                }
            }
        }

        JlrsTypeAttrs {
            julia_type,
            zst,
            constructor_for,
            scope_lifetime,
            data_lifetime,
            layout_params,
            elided_params,
            all_params,
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
                .parse_args_with(Punctuated::<syn::Meta, Token![,]>::parse_terminated)
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

pub fn impl_into_julia(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let is_enum = is_enum(&ast.data);

    if !is_enum && !is_repr_c(ast) {
        panic!(
            "IntoJulia can only be derived for types with the attribute #[repr(C)] {:?}.",
            ast.data
        );
    } else if is_enum && !is_repr_int(ast) {
        panic!("IntoJulia can only be derived for enums with an integer repr.");
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

    into_julia_impl.into()
}

pub fn impl_into_julia_fn(attrs: &JlrsTypeAttrs) -> Option<TS2> {
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
pub fn impl_into_julia_fn_enum() -> Option<TS2> {
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

pub fn impl_unbox(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let is_enum = is_enum(&ast.data);

    if !is_enum && !is_repr_c(ast) {
        panic!(
            "Unbox can only be derived for types with the attribute #[repr(C)] {:?}.",
            ast.data
        );
    } else if is_enum && !is_repr_int(ast) {
        panic!("Unbox can only be derived for enums with an integer repr.");
    }

    let generics = &ast.generics;
    let where_clause = match ast.generics.where_clause.as_ref() {
        Some(wc) => {
            let mut wc = wc.clone();
            for generic in generics.type_params() {
                let clause: syn::WherePredicate = syn::parse_quote! {
                    #generic: Clone
                };
                wc.predicates.push(clause)
            }
            wc
        }
        None => {
            let mut predicates = Punctuated::<_, Comma>::new();
            for generic in generics.type_params() {
                let clause: syn::WherePredicate = syn::parse_quote! {
                    #generic: Clone
                };
                predicates.push(clause)
            }

            syn::parse_quote! {
                where #predicates
            }
        }
    };

    let unbox_impl = quote! {
        unsafe impl #generics ::jlrs::convert::unbox::Unbox for #name #generics #where_clause {
            type Output = Self;
        }
    };

    unbox_impl.into()
}

pub fn impl_typecheck(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let is_enum = is_enum(&ast.data);

    if !is_enum && !is_repr_c(ast) {
        panic!(
            "Typecheck can only be derived for types with the attribute #[repr(C)] {:?}.",
            ast.data
        );
    } else if is_enum && !is_repr_int(ast) {
        panic!("Typecheck can only be derived for enums with an integer repr.");
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
            let mut predicates = Punctuated::<_, Comma>::new();
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

    typecheck_impl.into()
}

pub fn impl_construct_type(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let mut attrs = JlrsTypeAttrs::parse(ast);
    let jl_type = attrs.julia_type
        .take()
        .expect("ConstructType can only be derived if the corresponding Julia type is set with #[julia_type = \"Main.MyModule.Submodule.StructType\"]");

    let lifetimes = ast.generics.lifetimes().map(|_| -> syn::LifetimeParam {
        syn::parse_quote! { 'static }
    });

    let static_types = ast.generics.type_params().map(|p| -> syn::Type {
        let name = &p.ident;
        syn::parse_quote! { #name::Static }
    });

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
            let mut predicates = Punctuated::<_, Comma>::new();
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

    let n_names = ast.generics.type_params().count();
    let n_generics = ast.generics.params.len();

    let (cacheable, construct_expr, construct_with_context_expr): (
        Option<syn::Stmt>,
        syn::Expr,
        syn::Expr,
    ) = if n_names == 0 {
        let cacheable = syn::parse_quote! {
            const CACHEABLE: bool = false;
        };

        let construct_expr = syn::parse_quote! {
            base_type.root(target)
        };

        let construct_with_context_expr = syn::parse_quote! {
            base_type.root(target)
        };

        (Some(cacheable), construct_expr, construct_with_context_expr)
    } else {
        // FIXME
        let param_names = ast.generics.type_params().map(|p| &p.ident);
        let n_names = ast.generics.type_params().count();

        let n_slots = n_generics + 2;
        let nth_generic = 0..n_names;

        let construct_expr = syn::parse_quote! {
            target.with_local_scope::<_, _, #n_slots>(|target, mut frame| {

                if #n_names == 0 {
                    return base_type.root(target);
                }

                let mut types: [Option<::jlrs::data::managed::value::Value>; #n_names] = [None; #n_names];
                #(
                    types[#nth_generic] = Some(<#param_names as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame));
                )*
                unsafe {
                    let types = std::mem::transmute::<&[Option<::jlrs::data::managed::value::Value>; #n_names], &[::jlrs::data::managed::value::Value; #n_names]>(&types);
                    base_type
                        .apply_type(&mut frame, types)
                        .unwrap()
                        .cast::<::jlrs::data::managed::datatype::DataType>()
                        .unwrap()
                        .rewrap(target)
                }
            })
        };

        let nth_generic = 0..n_names;
        let param_names = ast.generics.type_params().map(|p| &p.ident);
        let construct_with_context_expr = syn::parse_quote! {
            target.with_local_scope::<_, _, #n_slots>(|target, mut frame| {
                if #n_names == 0 {
                    return base_type.root(target);
                }

                let mut types: [Option<::jlrs::data::managed::value::Value>; #n_names] = [None; #n_names];
                #(
                    types[#nth_generic] = Some(<#param_names as ::jlrs::data::types::construct_type::ConstructType>::construct_type_with_env(&mut frame, env));
                )*
                unsafe {
                    let types = std::mem::transmute::<&[Option<::jlrs::data::managed::value::Value>; #n_names], &[::jlrs::data::managed::value::Value; #n_names]>(&types);
                    base_type
                        .apply_type(&mut frame, types)
                        .unwrap()
                        .cast::<::jlrs::data::managed::datatype::DataType>()
                        .unwrap()
                        .wrap_with_env(target, env)
                }
            })
        };

        (None, construct_expr, construct_with_context_expr)
    };

    let construct_type_impl = quote! {
        unsafe impl #generics ::jlrs::data::types::construct_type::ConstructType for #name #generics #wc {
            type Static = #name <#(#lifetimes,)* #(#static_types,)*>;

            #cacheable

            fn construct_type_uncached<'target, Tgt>(
                target: Tgt,
            ) -> ::jlrs::data::managed::value::ValueData<'target, 'static, Tgt>
            where
                Tgt: ::jlrs::memory::target::Target<'target>,
            {
                let base_type = Self::base_type(&target).unwrap();
                #construct_expr
            }

            fn construct_type_with_env_uncached<'target, Tgt>(
                target: Tgt,
                env: &::jlrs::data::types::construct_type::TypeVarEnv,
            ) -> ::jlrs::data::managed::value::ValueData<'target, 'static, Tgt>
            where
                Tgt: ::jlrs::memory::target::Target<'target>,
            {
                let base_type = Self::base_type(&target).unwrap();
                #construct_with_context_expr
            }

            #[inline]
            fn base_type<'target, Tgt>(
                target: &Tgt
            ) -> Option<::jlrs::data::managed::value::Value<'target, 'static>>
            where
                Tgt: ::jlrs::memory::target::Target<'target>,
            {
                unsafe {
                    let value = ::jlrs::inline_static_ref!(STATIC, Value, #jl_type, target);
                    Some(value)
                }
            }
        }
    };

    construct_type_impl.into()
}

pub fn impl_has_layout(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let attrs = JlrsTypeAttrs::parse(ast);

    let layout_type = format_ident!(
        "{}",
        attrs
            .constructor_for
            .as_ref()
            .expect("HasLayout can only be implemented when a layout type is provided")
    );

    let all_params = attrs.all_params.iter().map(|i| format_ident!("{}", i));
    let all_params2 = all_params.clone();
    let all_generics: syn::Generics = syn::parse_quote! {
        <'scope, 'data, #(#all_params,)*>
    };

    let constructor_generics: syn::Generics = syn::parse_quote! {
        <#(#all_params2,)*>
    };

    let layout_params = attrs.layout_params.iter().map(|i| format_ident!("{}", i));
    let mut layout_generics: syn::Generics = syn::parse_quote! {
        <#(#layout_params,)*>
    };

    if attrs.scope_lifetime {
        layout_generics
            .params
            .insert(0, syn::parse_quote! { 'scope });
    }

    // 'data implies 'scope
    if attrs.data_lifetime {
        layout_generics
            .params
            .insert(1, syn::parse_quote! { 'data });
    }

    let where_clause: syn::WhereClause = {
        let mut predicates = Punctuated::<_, Comma>::new();

        for generic in attrs.layout_params.iter().map(|i| format_ident!("{}", i)) {
            let clause: syn::WherePredicate = syn::parse_quote! {
                #generic: ::jlrs::data::types::construct_type::ConstructType + ::jlrs::data::layout::valid_layout::ValidField
            };

            predicates.push(clause)
        }

        for generic in attrs.elided_params.iter().map(|i| format_ident!("{}", i)) {
            let clause: syn::WherePredicate = syn::parse_quote! {
                #generic: ::jlrs::data::types::construct_type::ConstructType
            };

            predicates.push(clause)
        }

        syn::parse_quote! {
            where #predicates
        }
    };

    let has_layout_impl = quote! {
        unsafe impl #all_generics ::jlrs::data::layout::typed_layout::HasLayout<'scope, 'data> for #name #constructor_generics #where_clause {
            type Layout = #layout_type #layout_generics;
        }
    };

    has_layout_impl.into()
}

pub fn impl_is_bits(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let is_enum = is_enum(&ast.data);

    if !is_enum && !is_repr_c(ast) {
        panic!(
            "IsBits can only be derived for types with the attribute #[repr(C)] {:?}.",
            ast.data
        );
    } else if is_enum && !is_repr_int(ast) {
        panic!("IsBits can only be derived for enums with an integer repr.");
    }

    let generics = &ast.generics;
    let wc = match ast.generics.where_clause.as_ref() {
        Some(wc) => {
            let mut wc = wc.clone();
            for generic in generics.type_params() {
                let clause: syn::WherePredicate = syn::parse_quote! {
                    #generic: ::jlrs::data::layout::is_bits::IsBits
                };
                wc.predicates.push(clause)
            }
            wc
        }
        None => {
            let mut predicates = Punctuated::<_, Comma>::new();
            for generic in generics.type_params() {
                let clause: syn::WherePredicate = syn::parse_quote! {
                    #generic: ::jlrs::data::layout::is_bits::IsBits
                };
                predicates.push(clause)
            }

            syn::parse_quote! {
                where #predicates
            }
        }
    };

    let is_bits_impl = quote! {
        unsafe impl #generics ::jlrs::data::layout::is_bits::IsBits for #name #generics #wc {}
    };

    is_bits_impl.into()
}

pub fn impl_valid_layout(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let is_enum = is_enum(&ast.data);

    if !is_enum && !is_repr_c(ast) {
        panic!(
            "ValidLayout can only be derived for types with the attribute #[repr(C)] {:?}.",
            ast.data
        );
    } else if is_enum && !is_repr_int(ast) {
        panic!("ValidLayout can only be derived for enums with an integer repr.");
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
            let mut predicates = Punctuated::<_, Comma>::new();
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

        valid_layout_impl.into()
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

        valid_layout_impl.into()
    }
}

pub fn impl_enum(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let repr = get_repr_int(ast).expect("Enum can only be derived for enums with an integer repr.");

    let syn::Data::Enum(data) = &ast.data else {
        unreachable!()
    };

    let mut variants = Vec::with_capacity(data.variants.len());
    for variant in data.variants.iter() {
        'variant: for attr in variant.attrs.iter() {
            if attr.path().is_ident("jlrs") {
                let parsed: Punctuated<syn::Meta, Comma> = attr
                    .parse_args_with(Punctuated::<syn::Meta, Token![,]>::parse_terminated)
                    .unwrap();

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
        panic!("All enum variants must be annotated with `julia_enum_variant`")
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

    enum_impl.into()
}

pub fn impl_valid_field(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let is_enum = is_enum(&ast.data);

    if !is_enum && !is_repr_c(ast) {
        panic!(
            "ValidField can only be derived for types with the attribute #[repr(C)] {:?}.",
            ast.data
        );
    } else if is_enum && !is_repr_int(ast) {
        panic!("ValidField can only be derived for enums with an integer repr.");
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
            let mut predicates = Punctuated::<_, Comma>::new();
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

    let valid_field_impl = quote! {
        unsafe impl #generics ::jlrs::data::layout::valid_layout::ValidField for #name #generics #where_clause {
            #[inline]
            fn valid_field(v: ::jlrs::data::managed::value::Value) -> bool {
                <Self as ::jlrs::data::layout::valid_layout::ValidLayout>::valid_layout(v)
            }
        }
    };

    valid_field_impl.into()
}

pub fn impl_ccall_arg(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let is_enum = is_enum(&ast.data);

    if !is_enum && !is_repr_c(ast) {
        panic!(
            "CCallArg can only be derived for types with the attribute #[repr(C)] {:?}.",
            ast.data
        );
    } else if is_enum && !is_repr_int(ast) {
        panic!("CCallArg can only be derived for enums with an integer repr.");
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
            let mut predicates = Punctuated::<_, Comma>::new();
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

    ccall_arg_impl.into()
}

pub fn impl_ccall_return(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let is_enum = is_enum(&ast.data);

    if !is_enum && !is_repr_c(ast) {
        panic!(
            "CCallReturn can only be derived for types with the attribute #[repr(C)] {:?}.",
            ast.data
        );
    } else if is_enum && !is_repr_int(ast) {
        panic!("CCallReturn can only be derived for enums with an integer repr.");
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
            let mut predicates = Punctuated::<_, Comma>::new();
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
        unsafe impl #generics ::jlrs::convert::ccall_types::CCallReturn for #name #generics #wc {
            type CCallReturnType = Self;
            type FunctionReturnType = Self;
            type ReturnAs = Self;

            #[inline]
            unsafe fn return_or_throw(self) -> Self::ReturnAs {
                self
            }
        }
    };

    ccall_arg_impl.into()
}

fn is_repr_c(ast: &syn::DeriveInput) -> bool {
    for attr in &ast.attrs {
        if attr.path().is_ident("repr") {
            let p: Result<syn::Path, _> = attr.parse_args();
            if let Ok(p) = p {
                if p.is_ident("C") {
                    return true;
                }
            }
        }
    }

    false
}

fn is_repr_int(ast: &syn::DeriveInput) -> bool {
    for attr in &ast.attrs {
        if attr.path().is_ident("repr") {
            let p: Result<syn::Path, _> = attr.parse_args();
            if let Ok(p) = p {
                if p.is_ident("i8")
                    || p.is_ident("i16")
                    || p.is_ident("i32")
                    || p.is_ident("i64")
                    || p.is_ident("isize")
                    || p.is_ident("u8")
                    || p.is_ident("u16")
                    || p.is_ident("u32")
                    || p.is_ident("u64")
                    || p.is_ident("usize")
                {
                    return true;
                }
            }
        }
    }

    false
}

fn get_repr_int(ast: &syn::DeriveInput) -> Option<syn::Ident> {
    for attr in &ast.attrs {
        if attr.path().is_ident("repr") {
            let p: Result<syn::Path, _> = attr.parse_args();
            if let Ok(p) = p {
                if p.is_ident("i8")
                    || p.is_ident("i16")
                    || p.is_ident("i32")
                    || p.is_ident("i64")
                    || p.is_ident("isize")
                    || p.is_ident("u8")
                    || p.is_ident("u16")
                    || p.is_ident("u32")
                    || p.is_ident("u64")
                    || p.is_ident("usize")
                {
                    return p.get_ident().map(Clone::clone);
                }
            }
        }
    }

    None
}

fn is_enum(data: &syn::Data) -> bool {
    match data {
        syn::Data::Struct(_) => false,
        syn::Data::Enum(_) => true,
        syn::Data::Union(_) => panic!("Union types are not supported"),
    }
}
