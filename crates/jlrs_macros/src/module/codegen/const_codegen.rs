use quote::format_ident;
use syn::{Expr, Ident, ItemFn, parse_quote};

use crate::ir::const_ir::ConstsIR;

pub struct ConstCodegen<'a> {
    init_fn_name: &'a Ident,
    ir: &'a ConstsIR<'a>,
}

impl<'a> ConstCodegen<'a> {
    pub fn new(init_fn_name: &'a Ident, ir: &'a ConstsIR) -> Self {
        ConstCodegen { init_fn_name, ir }
    }

    pub fn init_fn(&self) -> ItemFn {
        let fn_ident = format_ident!("{}_consts", self.init_fn_name);
        let fragments = self.ir.consts.iter().map(|constant| {
            let name = &constant.original_name;
            let rename = constant.export_name.name_string();
            let ty = &constant.ty;

            let expr: Expr = parse_quote! {
                frame.local_scope::<_, 2>(move |mut frame| {
                    let v: #ty = #name;
                    let value = ::jlrs::data::managed::value::Value::new(&mut frame, v);
                    module.set_const(&mut frame, #rename, value).unwrap();
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
