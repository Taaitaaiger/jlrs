use quote::format_ident;
use syn::{Expr, Ident, ItemFn, parse_quote};

use crate::ir::alias_ir::AliasesIR;

pub struct AliasCodegen<'a> {
    init_fn_name: &'a Ident,
    ir: &'a AliasesIR<'a>,
}

impl<'a> AliasCodegen<'a> {
    pub fn new(init_fn_name: &'a Ident, ir: &'a AliasesIR) -> Self {
        AliasCodegen { init_fn_name, ir }
    }

    pub fn init_fn(&self) -> ItemFn {
        let fn_ident = format_ident!("{}_aliases", self.init_fn_name);
        let fragments = self.ir.aliases.iter().map(|alias| {
            let name = &alias.export_name.name_string();
            let ty = &alias.ty;

            let expr: Expr = parse_quote! {
                frame.local_scope::<_, 2>(#[inline(never)] move |mut frame| {
                    let value = <#ty as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame);
                    module.set_const(&mut frame, #name, value).unwrap();
                })
            };

            expr
        });

        parse_quote! {
            fn #fn_ident<'target, Tgt: ::jlrs::memory::target::Target<'target>>(
                frame: &Tgt,
                module: ::jlrs::data::managed::module::Module,
            ) {
                #(#fragments;)*
            }
        }
    }
}
