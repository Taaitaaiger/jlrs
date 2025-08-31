use std::collections::{HashMap, HashSet};

use itertools::Itertools as _;
use syn::{Token, parse_quote, spanned::Spanned as _};

pub struct JlrsTypeAttrs {
    pub julia_type: Option<String>,
    pub constructor_for: Option<String>,
    pub zst: bool,
    pub scope_lifetime: bool,
    pub data_lifetime: bool,
    pub layout_params: Vec<String>,
    pub elided_params: Vec<String>,
    pub all_params: Vec<String>,
    key_type: Option<String>,
    super_type: Option<String>,
    bounds: Option<String>,
}

impl JlrsTypeAttrs {
    pub fn parse(ast: &syn::DeriveInput) -> Self {
        let mut julia_type: Option<String> = None;
        let mut constructor_for: Option<String> = None;
        let mut key_type: Option<String> = None;
        let mut bounds: Option<String> = None;
        let mut super_type: Option<String> = None;
        let mut scope_lifetime = false;
        let mut data_lifetime = false;
        let mut layout_params = Vec::new();
        let mut elided_params = Vec::new();
        let mut all_params = Vec::new();
        let mut zst = false;

        for attr in &ast.attrs {
            if attr.path().is_ident("jlrs") {
                let nested = attr
                    .parse_args_with(
                        syn::punctuated::Punctuated::<syn::Meta, Token![,]>::parse_terminated,
                    )
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
                        syn::Meta::NameValue(mnv) if mnv.path.is_ident("key") => {
                            if let syn::Expr::Lit(lit) = mnv.value {
                                if let syn::Lit::Str(s) = lit.lit {
                                    key_type = Some(s.value());
                                }
                            }
                        }
                        syn::Meta::NameValue(mnv) if mnv.path.is_ident("super_type") => {
                            if let syn::Expr::Lit(lit) = mnv.value {
                                if let syn::Lit::Str(s) = lit.lit {
                                    super_type = Some(s.value());
                                }
                            }
                        }
                        syn::Meta::NameValue(mnv) if mnv.path.is_ident("bounds") => {
                            if let syn::Expr::Lit(lit) = mnv.value {
                                if let syn::Lit::Str(s) = lit.lit {
                                    bounds = Some(s.value());
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
            key_type,
            super_type,
            bounds,
        }
    }

    pub fn bounds(&self) -> syn::Result<TypeBounds> {
        match self.bounds.as_ref() {
            Some(attrs) => syn::parse_str::<TypeBounds>(attrs),
            None => Ok(TypeBounds::empty()),
        }
    }

    pub fn key_type(&self, ast: &syn::DeriveInput) -> syn::Result<syn::Type> {
        let name = &ast.ident;

        if let Some(key_type) = self.key_type.as_ref() {
            syn::parse_str(key_type)
        } else {
            let key_params = ast.generics.type_params().map(|_param| -> syn::Type {
                parse_quote! { () }
            });
            Ok(parse_quote! { #name < #(#key_params,)* > })
        }
    }

    pub fn super_type(&self) -> syn::Result<Option<syn::Type>> {
        match self.super_type.as_ref() {
            Some(ty) => syn::parse_str(ty).map(|ty| Some(ty)),
            None => Ok(None),
        }
    }
}

pub struct TypeBounds {
    list: syn::punctuated::Punctuated<TypeBound, Token![,]>,
}

impl TypeBounds {
    fn empty() -> Self {
        TypeBounds {
            list: syn::punctuated::Punctuated::default(),
        }
    }

    pub fn validate(&self, params: &[syn::TypeParam]) -> syn::Result<()> {
        let mut names = HashSet::<&syn::Ident>::default();
        let mut seen = HashSet::<&syn::Ident>::default();
        names.extend(params.iter().map(|param| &param.ident));

        for bound in self.list.iter() {
            let name = match bound {
                TypeBound::Subtype { name, .. } => name,
                TypeBound::Supertype { name, .. } => name,
                TypeBound::SubSupertype { name, .. } => name,
            };

            if let syn::Type::Path(path) = name {
                if let Some(ident) = path.path.get_ident() {
                    if !names.contains(ident) {
                        let merged = names.into_iter().map(|s| s.to_string()).join(", ");
                        let msg = format!("Expected one of {}", merged);
                        return Err(syn::Error::new(name.span(), msg));
                    } else {
                        if !seen.insert(ident) {
                            let msg = format!("Multiple bounds for {}", ident.to_string());
                            return Err(syn::Error::new(name.span(), msg));
                        }
                    }
                } else {
                    return Err(syn::Error::new(name.span(), "malformed bounds"));
                }
            } else {
                return Err(syn::Error::new(name.span(), "malformed bounds"));
            }
        }

        Ok(())
    }

    pub fn to_map(&self) -> HashMap<&syn::Ident, &TypeBound> {
        let mut hm = HashMap::default();

        for bound in self.list.iter() {
            match bound {
                tb @ TypeBound::Subtype { name, .. } => {
                    if let syn::Type::Path(path) = name {
                        if let Some(ident) = path.path.get_ident() {
                            hm.insert(ident, tb);
                        }
                    }
                }
                tb @ TypeBound::Supertype { name, .. } => {
                    if let syn::Type::Path(path) = name {
                        if let Some(ident) = path.path.get_ident() {
                            hm.insert(ident, tb);
                        }
                    }
                }
                tb @ TypeBound::SubSupertype { name, .. } => {
                    if let syn::Type::Path(path) = name {
                        if let Some(ident) = path.path.get_ident() {
                            hm.insert(ident, tb);
                        }
                    }
                }
            }
        }

        hm
    }
}

impl syn::parse::Parse for TypeBounds {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let list = syn::punctuated::Punctuated::<TypeBound, Token![,]>::parse_terminated(input)?;
        Ok(TypeBounds { list })
    }
}

syn::custom_punctuation!(SubtypeJL, <:);
syn::custom_punctuation!(SupertypeJL, >:);

pub enum TypeBound {
    // {name} <: {supertype}
    Subtype {
        name: syn::Type,
        supertype: syn::Type,
    },
    // {name} >: {subtype}
    Supertype {
        name: syn::Type,
        subtype: syn::Type,
    },
    // {subtype} <: {name} <: {supertype}
    SubSupertype {
        subtype: syn::Type,
        name: syn::Type,
        supertype: syn::Type,
    },
}

// We want to express `A <: B`, `A >: B`, and `A <: B <: C` where A, B and C are types. The type
// parser fails if a type is followed by <: or >:, so we need to peel off the tokens before those
// operators.
fn parse_until<E1: syn::parse::Peek, E2: syn::parse::Peek>(
    input: syn::parse::ParseStream,
    end_subtype: E1,
    end_supertype: E2,
) -> syn::Result<proc_macro2::TokenStream> {
    let mut tokens = proc_macro2::TokenStream::new();
    while !input.is_empty() && !input.peek(end_subtype) && !input.peek(end_supertype) {
        let next: proc_macro2::TokenTree = input.parse()?;
        tokens.extend(Some(next));
    }
    Ok(tokens)
}

impl syn::parse::Parse for TypeBound {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let input = input;
        let before_separator = parse_until(input, SubtypeJL, SupertypeJL)?;
        let subty_or_name: syn::Type = syn::parse2(before_separator)?;

        if input.peek(SubtypeJL) {
            let _punct: SubtypeJL = input.parse()?;

            // There might be another <:, so we fork the input stream and try to parse a Type,
            // which fails if there is another <:
            let forked_input = input.fork();
            let superty_or_name_f: syn::Result<syn::Type> = forked_input.parse();

            if superty_or_name_f.is_ok() {
                // We could parse the forked stream, so we can parse the original one too. This
                // implies we're dealing with `A <: B`
                let supertype = input.parse()?;
                let name = subty_or_name;
                Ok(TypeBound::Subtype { name, supertype })
            } else {
                // Parsing failed, so there's another <: and we're dealing with `A <: B <: C`
                let before_separator = parse_until(input, SubtypeJL, SubtypeJL)?;
                let name = syn::parse2(before_separator)?;
                let subtype = subty_or_name;
                let _punct: SubtypeJL = input.parse()?;
                let supertype: syn::Type = input.parse()?;

                Ok(TypeBound::SubSupertype {
                    subtype,
                    name,
                    supertype,
                })
            }
        } else if input.peek(SupertypeJL) {
            // Can only be `A >: B`
            let name = subty_or_name;
            let _punct: SupertypeJL = input.parse()?;
            let subtype: syn::Type = input.parse()?;

            Ok(TypeBound::Supertype { name, subtype })
        } else {
            let t = input.to_string();
            Err(input.error(format!("Unexpected token {t}")))
        }
    }
}
