extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{self, Meta};

#[proc_macro_derive(JuliaTuple)]
pub fn julia_tuple_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_julia_tuple(&ast)
}

#[proc_macro_derive(JuliaStruct, attributes(julia_type, jlrs))]
pub fn julia_struct_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_julia_struct(&ast)
}

fn impl_julia_tuple(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    if !is_repr_c(ast) {
        panic!("JuliaTuple can only be derived for types with the attribute #[repr(C)]");
    }

    let fields = match &ast.data {
        syn::Data::Struct(s) => &s.fields,
        _ => panic!("JuliaTuple cannot be derived for enums and unions"),
    };

    let field_types_iter = match fields {
        syn::Fields::Unnamed(u) => u.unnamed.iter().map(|f| &f.ty),
        _ => panic!("JuliaTuple cannot be derived for unit structs and structs with named fields"),
    };Cast

    let julia_tuple_impl = quote! {
        unsafe impl ::jlrs::traits::JuliaType for #name {
            unsafe fn julia_type() -> *mut ::jlrs::jl_sys_export::jl_datatype_t {
                let mut elem_types = [ #( <#field_types_iter as ::jlrs::traits::JuliaType>::julia_type(), )* ];
                let ty = ::jlrs::jl_sys_export::jl_apply_tuple_type_v(elem_types.as_mut_ptr().cast(), elem_types.len());
                assert!(::jlrs::jl_sys_export::jl_isbits(ty.cast()));
                ty.cast()
            }
        }

        unsafe impl ::jlrs::traits::JuliaTypecheck for #name {
            unsafe fn julia_typecheck(t: ::jlrs::value::datatype::DataType) -> bool {
                t.ptr() == <Self as ::jlrs::traits::JuliaType>::julia_type()
            }
        }

        unsafe impl ::jlrs::traits::IntoJulia for #name {
            unsafe fn into_julia(&self) -> *mut ::jlrs::jl_sys_export::jl_value_t {
                let ty = <Self as ::jlrs::traits::JuliaType>::julia_type();
                let tuple = ::jlrs::jl_sys_export::jl_new_struct_uninit(ty.cast());
                let data: *mut Self = tuple.cast();
                ::std::ptr::write(data, *self);

                tuple
            }
        }

        unsafe impl<'frame, 'data> ::jlrs::traits::Cast<'frame, 'data> for #name {
            type Output = Self;

            fn cast(value: ::jlrs::value::Value<'frame, 'data>) -> ::jlrs::error::JlrsResult<Self::Output> {
                if value.is::<#name>() {
                    return unsafe { Ok(*(value.ptr().cast::<Self>())) };
                }

                Err(::jlrs::error::JlrsError::WrongType)?
            }

            unsafe fn cast_unchecked<'fr, 'da>(value: ::jlrs::value::Value<'frame, 'data>) -> Self::Output {
                *(value.ptr().cast::<Self>())
            }
        }

        unsafe impl ::jlrs::traits::JuliaTuple for #name {}
    };

    julia_tuple_impl.into()
}

fn impl_julia_struct(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    if !is_repr_c(ast) {
        panic!("JuliaStruct can only be derived for types with the attribute #[repr(C)].");
    }

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

    let fields = match &ast.data {
        syn::Data::Struct(s) => &s.fields,
        _ => panic!("JuliaStruct cannot be derived for enums and unions."),
    };

    let (n_fields, expected_field_names_iter, rs_field_types_iter) = match fields {
        syn::Fields::Named(n) => {
            let n_fields = n.named.len();
            let types = n.named.iter().map(|f| &f.ty);

            let names = n.named.iter().map(expected_field_name);

            (n_fields, names, types)
        }
        _ => panic!("JuliaStruct cannot be derived for unit and tuple structs."),
    };

    let fields_idx_it = 0..n_fields;
    let rs_fields_iter = fields
        .iter()
        .map(|field| field.ident.as_ref().unwrap().to_string());

    let rs_field_types_iter_b = rs_field_types_iter.clone();

    let julia_struct_impl = quote! {
        unsafe impl ::jlrs::traits::JuliaType for #name {
            unsafe fn julia_type() -> *mut ::jlrs::jl_sys_export::jl_datatype_t {
                // Because Julia code can change independently of the Rust code that calls it, we
                // need to check if the Julia type corresponds to the Rust type at runtime. This
                // check happens once, when this function is first called. If the check fails the
                // code will panic and if it succeeds  the Julia type is stored in a thread local
                // static variable.
                thread_local! {
                    static JULIA_TYPE: *mut ::jlrs::jl_sys_export::jl_datatype_t =  unsafe {
                        // The Julia type is a global value in its module
                        let global = ::jlrs::global::Global::new();
                        let julia_type = ::jlrs::value::module::Module::#func(global)
                            #(.submodule(#modules_it).expect(&format!("Submodule {} cannot be found", #modules_it_b)))*
                            .global(#ty).expect(&format!("Type {} cannot be found in module", #ty))
                            .ptr();

                        // Check if a type was given and if it uses the isbits optimization. isbits-types store
                        // their data inline and are compatible with C-style structs.
                        assert!(::jlrs::jl_sys_export::jl_is_datatype(julia_type), "{} is not a Julia type", #ty);
                        assert!(::jlrs::jl_sys_export::jl_isbits(julia_type.cast()), "{} is a Julia type but isbitstype returned false", #ty);

                        // Get the field names, number of fields, and field types.
                        let field_names_svec = ::jlrs::jl_sys_export::jl_field_names(julia_type.cast());
                        let n_fields = ::jlrs::jl_sys_export::jl_svec_len(field_names_svec);
                        assert_eq!(n_fields, #n_fields, "Wrong number of fields (expected {}, found {})", n_fields, #n_fields);

                        let field_names_ptr = ::jlrs::jl_sys_export::jl_svec_data(field_names_svec).cast::<*mut ::jlrs::jl_sys_export::jl_sym_t>();
                        let field_names_slice = ::std::slice::from_raw_parts(field_names_ptr, n_fields);

                        let fieldtypes_svec = ::jlrs::jl_sys_export::jl_get_fieldtypes(julia_type.cast());
                        let fieldtypes_ptr = ::jlrs::jl_sys_export::jl_svec_data(fieldtypes_svec).cast::<*mut ::jlrs::jl_sys_export::jl_datatype_t>();
                        let fieldtypes_slice = ::std::slice::from_raw_parts(fieldtypes_ptr, n_fields);

                        // Check if the field names and types match between Rust and Julia.
                        #(
                            let i = #fields_idx_it;
                            let jl_field_name_str = ::std::ffi::CStr::from_ptr(::jlrs::jl_sys_export::jl_symbol_name(field_names_slice[i]).cast()).to_string_lossy();
                            let assoc_field_type = <#rs_field_types_iter as ::jlrs::traits::JuliaType>::julia_type().cast();

                            let rs_renamed_field_name = #expected_field_names_iter;
                            let rs_concrete_field_name = #rs_fields_iter;

                            if rs_renamed_field_name == rs_concrete_field_name {
                                assert_eq!(
                                    rs_renamed_field_name,
                                    jl_field_name_str,
                                    "The Rust struct {} has field {}, but the corresponding field of the Julia struct {} is {}. You can rename this field explicitly by setting the attribute #[jlrs(rename = \"{}\")] on the field in Rust",
                                    stringify!(#name),
                                    rs_concrete_field_name,
                                    #ty,
                                    jl_field_name_str,
                                    jl_field_name_str,
                                );
                            } else {
                                assert_eq!(
                                    rs_renamed_field_name,
                                    jl_field_name_str,
                                    "The field {} of the Rust struct {} has been renamed to {}, but the corresponding field of the Julia struct {} is {}",
                                    rs_concrete_field_name,
                                    stringify!(#name),
                                    rs_renamed_field_name,
                                    #ty,
                                    jl_field_name_str,
                                );
                            }

                            assert_eq!(
                                fieldtypes_slice[i],
                                assoc_field_type,
                                "The field {} of the Rust struct {} has type {} which corresponds to {} in Julia, but the corresponding field of the Julia struct {}, {}, has type {}",
                                rs_concrete_field_name,
                                stringify!(#name),
                                stringify!(#rs_field_types_iter_b),
                                ::std::ffi::CStr::from_ptr(::jlrs::jl_sys_export::jl_typename_str(assoc_field_type.cast()).cast()).to_string_lossy(),
                                #ty,
                                jl_field_name_str,
                                ::std::ffi::CStr::from_ptr(::jlrs::jl_sys_export::jl_typename_str(fieldtypes_slice[i].cast()).cast()).to_string_lossy(),
                            );
                        )*

                        julia_type.cast()
                    };
                }

                JULIA_TYPE.with(|julia_type| { *julia_type })
            }
        }

        unsafe impl ::jlrs::traits::JuliaTypecheck for #name {
            unsafe fn julia_typecheck(t: ::jlrs::value::datatype::DataType) -> bool {
                t.ptr() == <Self as ::jlrs::traits::JuliaType>::julia_type()
            }
        }

        unsafe impl ::jlrs::traits::IntoJulia for #name {
            unsafe fn into_julia(&self) -> *mut ::jlrs::jl_sys_export::jl_value_t {
                let ty = <Self as ::jlrs::traits::JuliaType>::julia_type();
                let strct = ::jlrs::jl_sys_export::jl_new_struct_uninit(ty.cast());
                let data: *mut Self = strct.cast();
                // Avoid reading uninitialized data
                ::std::ptr::write(data, *self);

                strct
            }
        }

        unsafe impl<'frame, 'data> ::jlrs::traits::Cast<'frame, 'data> for #name {
            type Output = Self;

            fn cast(value: ::jlrs::value::Value<'frame, 'data>) -> ::jlrs::error::JlrsResult<Self::Output> {
                if value.is::<#name>() {
                    return unsafe { Ok(*(value.ptr().cast::<Self>())) };
                }

                Err(::jlrs::error::JlrsError::WrongType)?
            }

            unsafe fn cast_unchecked<'fr, 'da>(value: ::jlrs::value::Value<'frame, 'data>) -> Self::Output {
                *value.ptr().cast::<Self>()
            }
        }
    };

    julia_struct_impl.into()
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

fn expected_field_name(field: &syn::Field) -> String {
    for attr in &field.attrs {
        if attr.path.is_ident("jlrs") {
            if let Ok(Meta::List(p)) = attr.parse_meta() {
                if let syn::NestedMeta::Meta(syn::Meta::NameValue(nv)) = p.nested.first().unwrap() {
                    if nv.path.is_ident("rename") {
                        if let syn::Lit::Str(string) = &nv.lit {
                            return string.value();
                        }
                    }
                }
            }
        }
    }

    field.ident.as_ref().unwrap().to_string()
}
