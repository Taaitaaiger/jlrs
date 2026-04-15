//! Intermediate representation of `julia_module!`

use syn::Ident;

use crate::{
    JuliaModuleModel,
    ir::{
        alias_ir::AliasesIR, const_ir::ConstsIR, documentation_ir::DocsIR, exports_ir::ExportsIR,
        function_ir::FunctionsIR, struct_ir::StructsIR,
    },
};

pub mod alias_ir;
pub mod const_ir;
pub mod documentation_ir;
pub mod exports_ir;
pub mod function_ir;
pub mod struct_ir;

pub struct JuliaModuleIR<'a> {
    pub init_fn: &'a Ident,
    pub docs_ir: DocsIR<'a>,
    pub structs_ir: StructsIR<'a>,
    pub functions_ir: FunctionsIR<'a>,
    pub aliases_ir: AliasesIR<'a>,
    pub consts_ir: ConstsIR<'a>,
    pub exports_ir: ExportsIR<'a>,
}

impl<'a> JuliaModuleIR<'a> {
    pub fn from_model(module_model: &'a JuliaModuleModel) -> Self {
        let init_fn = &module_model.init_fn;
        let docs_ir = DocsIR::from_models(&module_model.exports);
        let exports_ir = ExportsIR::from_models(&module_model.exports);
        let structs_ir = StructsIR::from_models(&module_model.exports);
        let functions_ir = FunctionsIR::from_models(&module_model.exports);
        let aliases_ir = AliasesIR::from_models(&module_model.exports);
        let consts_ir = ConstsIR::from_models(&module_model.exports);

        JuliaModuleIR {
            init_fn,
            docs_ir,
            structs_ir,
            functions_ir,
            aliases_ir,
            consts_ir,
            exports_ir,
        }
    }
}
