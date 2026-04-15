use quote::format_ident;
use syn::{Expr, Ident, ItemFn, parse_quote};

use crate::ir::exports_ir::ExportsIR;

pub struct ExportsCodegen<'a> {
    init_fn_name: &'a Ident,
    ir: &'a ExportsIR<'a>,
}

impl<'a> ExportsCodegen<'a> {
    pub fn new(init_fn_name: &'a Ident, ir: &'a ExportsIR) -> Self {
        ExportsCodegen { init_fn_name, ir }
    }

    pub fn init_fn(&self) -> ItemFn {
        let fn_ident = format_ident!("{}_exports", self.init_fn_name);
        let n_exports = self.ir.exports.len();

        let fragments = self
            .ir
            .exports
            .iter()
            .enumerate()
            .map(|(index, export_name)| {
                let name = export_name.name_string();

                let expr: Expr = parse_quote! {
                    frame.local_scope::<_, 1>(#[inline(never)] |mut frame| {
                        unsafe {
                            let item = ::jlrs::data::managed::symbol::Symbol::new(&frame, #name);
                            accessor.set_value(&mut frame, #index, item.as_value()).unwrap().unwrap();
                        }
                    })
                };

                expr
            });

        parse_quote! {
            unsafe fn #fn_ident<'target, Tgt: ::jlrs::memory::target::Target<'target>>(
                frame: &Tgt,
                array: &mut ::jlrs::data::managed::array::VectorAny<'_, 'static>,
            ) {
                use ::jlrs::data::managed::array::{data::accessor::{AccessorMut1D as _, AccessorMut as _, Accessor as _}, dimensions::Dims as _};

                unsafe {
                    let mut accessor = array.indeterminate_data_mut();
                    accessor.grow_end_unchecked(#n_exports);

                    #(#fragments;)*
                }
            }
        }
    }
}
