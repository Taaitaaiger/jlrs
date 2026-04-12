use syn::{Signature, Type};

use crate::{
    model::{
        ItemModel,
        function_model::{FunctionKind, FunctionModel},
    },
    module::model::export_name::ExportName,
};

pub struct FunctionIR<'a> {
    pub kind: &'a FunctionKind,
    pub export_name: &'a ExportName<'a>,
    pub gc_safe: bool,
    pub signature: &'a Signature,
    pub type_var_env: Option<&'a Type>,
}

impl<'a> FunctionIR<'a> {
    fn from_model(ir: &mut Vec<FunctionIR<'a>>, model: &'a FunctionModel) {
        let export_name = &model.export_name;
        ir.reserve(model.variants.len());

        for variant in model.variants.iter() {
            let gc_safe = variant.gc_safe;
            let signature = &variant.signature;
            let type_var_env = variant.type_var_env.as_ref().map(|a| &a.ty);
            let kind = &variant.kind;

            ir.push(FunctionIR {
                kind,
                export_name: export_name,
                gc_safe,
                signature,
                type_var_env,
            });
        }
    }
}

pub struct FunctionsIR<'a> {
    pub functions: Vec<FunctionIR<'a>>,
}

impl<'a> FunctionsIR<'a> {
    pub fn n_exported_functions(&'a self) -> usize {
        self.functions.len()
    }

    pub fn from_models(models: &'a [ItemModel]) -> Self {
        let mut functions = Vec::new();

        for model in models {
            match model {
                ItemModel::Function(function_model) => {
                    FunctionIR::from_model(&mut functions, function_model)
                }
                _ => (),
            }
        }

        FunctionsIR { functions }
    }
}
