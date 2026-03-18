use syn::Type;

use crate::{
    model::{ItemModel, export_name::ExportName},
    module::model::alias_model::AliasModel,
};

pub struct AliasIR<'a> {
    pub export_name: &'a ExportName<'a>,
    pub ty: &'a Type,
}

impl<'a> AliasIR<'a> {
    pub fn from_model(model: &'a AliasModel) -> Self {
        AliasIR {
            export_name: &model.export_name,
            ty: &model.ty,
        }
    }
}

pub struct AliasesIR<'a> {
    pub aliases: Vec<AliasIR<'a>>,
}

impl<'a> AliasesIR<'a> {
    pub fn from_models(models: &'a [ItemModel]) -> Self {
        let mut aliases = Vec::new();

        for item in models {
            match item {
                ItemModel::Alias(alias_model) => aliases.push(AliasIR::from_model(alias_model)),
                _ => (),
            }
        }

        AliasesIR { aliases }
    }
}

#[cfg(test)]
mod tests {}
