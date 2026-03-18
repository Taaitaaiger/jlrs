use itertools::Itertools;
use syn::Path;

use crate::{model::ItemModel, module::model::export_name::ExportName};

pub struct StructIR<'a> {
    pub export_name: &'a ExportName<'a>,
    pub key: &'a Path,
    pub paths: Vec<&'a Path>,
}

pub struct StructsIR<'a> {
    pub structs: Vec<StructIR<'a>>,
}

impl<'a> StructsIR<'a> {
    pub fn from_models(models: &'a [ItemModel]) -> Self {
        let mut structs = Vec::new();

        for model in models {
            match model {
                ItemModel::Struct(struct_model) => {
                    let ir = StructIR {
                        export_name: &struct_model.export_name,
                        key: &struct_model.kinds.key,
                        paths: struct_model.kinds.variants.iter().unique().collect(),
                    };

                    structs.push(ir);
                }
                _ => (),
            }
        }

        StructsIR { structs }
    }
}
