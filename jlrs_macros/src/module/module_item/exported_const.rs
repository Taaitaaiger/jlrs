use quote::format_ident;
use syn::{
    parse::{Parse, ParseStream},
    parse_quote, Expr, Ident, ItemFn, Result, Token, Type,
};

use super::init_fn::InitFn;
use crate::JuliaModule;

pub struct ExportedConst {
    pub is_pub: bool,
    pub _const_token: Token![const],
    pub name: Ident,
    pub _colon: Token![:],
    pub ty: Type,
    pub _as_token: Option<Token![as]>,
    pub name_override: Option<Ident>,
}

impl Parse for ExportedConst {
    fn parse(input: ParseStream) -> Result<Self> {
        let const_token = input.parse()?;
        let name = input.parse()?;
        let colon = input.parse()?;
        let ty = input.parse()?;

        let lookahead = input.lookahead1();
        if lookahead.peek(Token![as]) {
            let as_token = input.parse()?;
            let name_override = input.parse()?;

            Ok(ExportedConst {
                is_pub: false,
                _const_token: const_token,
                name: name,
                _colon: colon,
                ty: ty,
                _as_token: Some(as_token),
                name_override: Some(name_override),
            })
        } else {
            Ok(ExportedConst {
                is_pub: false,
                _const_token: const_token,
                name: name,
                _colon: colon,
                ty: ty,
                _as_token: None,
                name_override: None,
            })
        }
    }
}

pub struct ConstFragments {
    pub const_init_fn: ItemFn,
    pub const_init_ident: Ident,
}

impl ConstFragments {
    pub fn generate(module: &JuliaModule, init_fn: &InitFn) -> Self {
        let const_init_ident = format_ident!("{}_consts", init_fn.init_fn);
        let const_init_fragments = module.get_exported_consts().map(const_info_fragment);

        let const_init_fn = parse_quote! {
            fn #const_init_ident<'target, Tgt: ::jlrs::memory::target::Target<'target>>(
                frame: &Tgt,
                module: ::jlrs::data::managed::module::Module,
            ) {
                #(#const_init_fragments;)*
            }
        };

        ConstFragments {
            const_init_ident,
            const_init_fn,
        }
    }
}

fn const_info_fragment(info: &ExportedConst) -> Expr {
    let name = &info.name;
    let rename = info.name_override.as_ref().unwrap_or(name).to_string();
    let ty = &info.ty;

    parse_quote! {
        frame.local_scope::<_, 2>(move |mut frame| {
            let v: #ty = #name;
            let value = ::jlrs::data::managed::value::Value::new(&mut frame, v);
            module.set_const(&mut frame, #rename, value).unwrap();
        })
    }
}
