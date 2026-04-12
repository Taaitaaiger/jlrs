use syn::{Expr, Ident, parse_quote};

use crate::module::{ast::expanded::expanded_as::ExpandedAs, model::export_name::ExportName};

pub fn module_codegen(module: &Ident, name_override: &ExportName) -> Expr {
    match name_override {
        ExportName::Name(_) | ExportName::Override(ExpandedAs::Local(_)) => {
            parse_quote! { module }
        }
        ExportName::Override(ExpandedAs::Global(name_override)) => {
            let n_parts = name_override.name_override.len();
            if n_parts == 1 {
                return parse_quote! { #module };
            }

            let modules = name_override
                .name_override
                .iter()
                .take(n_parts - 1)
                .map(|ident| ident.to_string());

            parse_quote! {
                {
                    let mut #module = ::jlrs::data::managed::module::Module::main(&frame);

                    #(
                        #module = module
                            .submodule(&frame, #modules)
                            .expect("Submodule does not exist")
                            .as_managed();
                    )*

                    #module
                }
            }
        }
    }
}
