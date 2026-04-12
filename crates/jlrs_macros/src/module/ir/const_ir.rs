use syn::{Ident, Type};

use crate::model::{ItemModel, const_model::ConstModel, export_name::ExportName};

pub struct ConstIR<'a> {
    pub original_name: &'a Ident,
    pub export_name: &'a ExportName<'a>,
    pub ty: &'a Type,
}

impl<'a> ConstIR<'a> {
    pub fn from_model(model: &'a ConstModel) -> Self {
        ConstIR {
            export_name: &model.export_name,
            ty: &model.ty,
            original_name: &model.original_name,
        }
    }
}

pub struct ConstsIR<'a> {
    pub consts: Vec<ConstIR<'a>>,
}

impl<'a> ConstsIR<'a> {
    pub fn from_models(models: &'a [ItemModel]) -> Self {
        let mut consts = Vec::new();

        for item in models {
            match item {
                ItemModel::Const(const_model) => consts.push(ConstIR::from_model(const_model)),
                _ => (),
            }
        }

        ConstsIR { consts }
    }
}
