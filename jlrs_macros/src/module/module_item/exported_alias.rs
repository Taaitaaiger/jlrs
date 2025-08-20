use quote::format_ident;
use syn::{
    Expr, Ident, ItemFn, Result, Token, Type,
    parse::{Parse, ParseStream},
    parse_quote,
};

use super::init_fn::InitFn;
use crate::JuliaModule;

pub struct ExportedAlias {
    pub is_pub: bool,
    pub _type_token: Token![type],
    pub name: Ident,
    pub _is: Token![=],
    pub ty: Type,
}

impl Parse for ExportedAlias {
    fn parse(input: ParseStream) -> Result<Self> {
        let type_token = input.parse()?;
        let name = input.parse()?;
        let is = input.parse()?;
        let ty = input.parse()?;

        Ok(ExportedAlias {
            is_pub: false,
            _type_token: type_token,
            name,
            _is: is,
            ty,
        })
    }
}

pub struct AliasFragments {
    pub alias_init_fn: ItemFn,
    pub alias_init_ident: Ident,
}

impl AliasFragments {
    pub fn generate(module: &JuliaModule, init_fn: &InitFn) -> Self {
        let alias_init_ident = format_ident!("{}_aliases", init_fn.init_fn);
        let alias_init_fragments = module.get_exported_aliases().map(alias_info_fragment);

        let alias_init_fn = parse_quote! {
            fn #alias_init_ident<'target, Tgt: ::jlrs::memory::target::Target<'target>>(
                frame: &Tgt,
                module: ::jlrs::data::managed::module::Module,
            ) {
                #(#alias_init_fragments;)*
            }
        };

        AliasFragments {
            alias_init_ident,
            alias_init_fn,
        }
    }
}

fn alias_info_fragment(info: &ExportedAlias) -> Expr {
    let name = &info.name.to_string();
    let ty = &info.ty;

    parse_quote! {
        frame.local_scope::<_, 2>(move |mut frame| {
            let value = <#ty as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame);
            module.set_const(&mut frame, #name, value).unwrap();
        })
    }
}
