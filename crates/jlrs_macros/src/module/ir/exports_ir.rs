use crate::model::{ItemModel, export_name::ExportName};

pub struct ExportsIR<'a> {
    pub exports: Vec<&'a ExportName<'a>>,
}

impl<'a> ExportsIR<'a> {
    pub fn from_models(models: &'a [ItemModel]) -> Self {
        let mut exports = Vec::new();

        for item in models {
            match item {
                ItemModel::Const(const_model) if const_model.public => {
                    let export_name = &const_model.export_name;
                    exports.push(export_name);
                }
                ItemModel::Alias(alias_model) if alias_model.public => {
                    let export_name = &alias_model.export_name;
                    exports.push(export_name);
                }
                ItemModel::Function(function_model) if function_model.public => {
                    let export_name = &function_model.export_name;
                    exports.push(export_name);
                }
                ItemModel::Struct(struct_model) if struct_model.public => {
                    let export_name = &struct_model.export_name;
                    exports.push(export_name);
                }
                _ => (),
            }
        }

        ExportsIR { exports }
    }
}
