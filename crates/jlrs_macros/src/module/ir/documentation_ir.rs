use crate::{model::ItemModel, module::model::export_name::ExportName};

pub struct DocIR<'a> {
    pub doc: String,
    pub export_name: &'a ExportName<'a>,
}

impl<'a> DocIR<'a> {
    fn new(doc: String, export_name: &'a ExportName) -> Self {
        DocIR { doc, export_name }
    }
}

pub struct DocsIR<'a> {
    pub docs: Vec<DocIR<'a>>,
}

impl<'a> DocsIR<'a> {
    pub fn from_models(models: &'a [ItemModel]) -> Self {
        let mut docs = Vec::new();

        for item in models {
            match item {
                ItemModel::Const(const_model) if const_model.documentation.is_some() => {
                    let export_name = &const_model.export_name;
                    let doc = const_model.documentation.as_ref().unwrap().to_string();
                    let item = DocIR::new(doc, export_name);
                    docs.push(item);
                }
                ItemModel::Alias(alias_model) if alias_model.documentation.is_some() => {
                    let export_name = &alias_model.export_name;
                    let doc = alias_model.documentation.as_ref().unwrap().to_string();
                    let item = DocIR::new(doc, export_name);
                    docs.push(item);
                }
                ItemModel::Function(function_model) if function_model.documentation.is_some() => {
                    let export_name = &function_model.export_name;
                    let doc = function_model.documentation.as_ref().unwrap().to_string();
                    let item = DocIR::new(doc, export_name);
                    docs.push(item);
                }
                ItemModel::Struct(struct_model) if struct_model.documentation.is_some() => {
                    let export_name = &struct_model.export_name;
                    let doc = struct_model.documentation.as_ref().unwrap().to_string();
                    let item = DocIR::new(doc, export_name);
                    docs.push(item);
                }
                _ => (),
            }
        }

        DocsIR { docs }
    }
}
