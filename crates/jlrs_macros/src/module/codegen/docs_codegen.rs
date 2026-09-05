use quote::format_ident;
use syn::{Expr, Ident, ItemFn, parse_quote};

use crate::{codegen::name_codegen::module_codegen, ir::documentation_ir::DocsIR};

pub struct DocsCodegen<'a> {
    init_fn_name: &'a Ident,
    ir: &'a DocsIR<'a>,
}

#[cfg(not(feature = "jl-docs"))]
fn get_filename() -> Expr {
    parse_quote! { "" }
}

#[cfg(feature = "jl-docs")]
fn get_filename() -> Expr {
    parse_quote! { concat!(env!("CARGO_MANIFEST_DIR"), "/", file!()) }
}

impl<'a> DocsCodegen<'a> {
    pub fn new(init_fn_name: &'a Ident, ir: &'a DocsIR) -> Self {
        DocsCodegen { init_fn_name, ir }
    }

    pub fn init_fn(&self) -> ItemFn {
        let fn_ident = format_ident!("{}_docs", self.init_fn_name);
        let module = format_ident!("module");
        let n_docs = self.ir.docs.len();

        let fragments = self.ir.docs.iter().enumerate().map(|(index, doc_item)| {
            let get_module = module_codegen(&module, &doc_item.export_name);
            let name = doc_item.export_name.name_string();
            let doc = doc_item.doc.as_str();
            let line = doc_item.line as isize;
            let filename = get_filename();

            let expr: Expr = parse_quote! {
                frame.local_scope::<_, 5>(#[inline(never)] |mut frame| {
                    unsafe {
                        let #module = #get_module;
                        let item = ::jlrs::data::managed::symbol::Symbol::new(&frame, #name);
                        let signature = ::jlrs::data::managed::value::Value::bottom_type(&frame);
                        let doc = ::jlrs::data::managed::string::JuliaString::new(&mut frame, #doc);
                        let filename = #filename;
                        let filename = ::jlrs::data::managed::string::JuliaString::new(&mut frame, filename);
                        let line = ::jlrs::data::managed::value::Value::new(&mut frame, #line as isize);

                        let doc_it = doc_item_ty.as_value().call(&mut frame, [#module.as_value(), filename.as_value(), line, item.as_value(), signature, doc.as_value()]).unwrap();
                        accessor.set_value(&mut frame, #index, doc_it).unwrap().unwrap();
                    }
                })
            };

            expr
        });

        parse_quote! {
            unsafe fn #fn_ident<'target, Tgt: ::jlrs::memory::target::Target<'target>>(
                frame: &Tgt,
                array: &mut ::jlrs::data::managed::array::Vector<'_, 'static>,
                module: ::jlrs::data::managed::module::Module,
                doc_item_ty: ::jlrs::data::managed::datatype::DataType,
            ) {
                use ::jlrs::data::managed::array::{data::accessor::{AccessorMut1D as _, AccessorMut as _, Accessor as _}, dimensions::Dims as _};

                unsafe {
                    let mut accessor = array.indeterminate_data_mut();
                    accessor.grow_end_unchecked(#n_docs);

                    #(#fragments;)*
                }
            }
        }
    }
}
