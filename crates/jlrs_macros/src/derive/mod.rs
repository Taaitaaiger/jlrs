pub mod attrs;
pub mod ccall_arg;
pub mod ccall_return;
pub mod construct_type;
pub mod enum_impl;
pub mod foreign_type;
pub mod has_layout;
pub mod into_julia;
pub mod is_bits;
pub mod opaque_type;
pub mod typecheck;
pub mod unbox;
pub mod valid_field;
pub mod valid_layout;

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
