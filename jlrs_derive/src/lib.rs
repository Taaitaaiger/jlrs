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

fn impl_julia_tuple(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    if !is_repr_c(ast) {
        panic!("IntoTuple can only be derived for types with the attribute #[repr(C)].");
    }

    let fields = match &ast.data {
        syn::Data::Struct(s) => &s.fields,
        _ => panic!("Enums and unions are not supported."),
    };

    let field_types = match fields {
        syn::Fields::Named(n) => n.named.iter().map(|f| &f.ty).collect::<Vec<_>>(),
        syn::Fields::Unnamed(u) => u.unnamed.iter().map(|f| &f.ty).collect::<Vec<_>>(),
        _ => panic!("Unit structs are not supported."),
    };

    let it = field_types.iter();
    let julia_type_impl = quote! {
        unsafe impl jlrs::traits::JuliaType for #name {
            unsafe fn julia_type() -> *mut jlrs::jl_sys_export::jl_value_t {
                let mut elem_types = [ #( <#it as jlrs::traits::JuliaType>::julia_type(), )* ];
                jlrs::jl_sys_export::jl_apply_tuple_type_v(elem_types.as_mut_ptr().cast(), elem_types.len()).cast()
            }
        }

        unsafe impl jlrs::traits::IntoJulia for #name {
            unsafe fn into_julia(&self) -> *mut jlrs::jl_sys_export::jl_value_t {
                let ty = <Self as jlrs::traits::JuliaType>::julia_type();
                assert!(jlrs::jl_sys_export::jl_isbits(ty.cast()));
                let tuple = jlrs::jl_sys_export::jl_new_struct_uninit(ty.cast());
                let data: *mut Self = tuple.cast();
                std::ptr::write(data, *self);

                tuple
            }
        }

        unsafe impl jlrs::traits::TryUnbox for #name {
            unsafe fn try_unbox(value: *mut jlrs::jl_sys_export::jl_value_t) -> jlrs::error::JlrsResult<Self> {
                let ty = <Self as jlrs::traits::JuliaType>::julia_type();
                assert!(jlrs::jl_sys_export::jl_isbits(ty.cast()));
                if jlrs::jl_sys_export::jl_typeis(value, ty.cast()) {
                    return Ok(*(value as *mut Self));
                }

                Err(jlrs::error::JlrsError::WrongType.into())
            }
        }

        unsafe impl jlrs::traits::JuliaTuple for #name {}
    };

    julia_type_impl.into()
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
