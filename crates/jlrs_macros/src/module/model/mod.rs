//! Analyzed model of `julia_module!`
//!
//! In the analysis phase, items are collected by export name, environments are expanded, and the
//! following is checked:
//!
//! - Exported constants are unique
//!
//! - Exported aliases are unique
//!
//! - Items exported under the same name have a consistent export type
//!
//! - Items exported under the same name are either consistently private or public
//!
//! - Every exported item is documented at most once
//!
//! - Only supported attributes are used

use std::collections::{HashMap, hash_map::Entry};

use syn::{Error, Ident, Result};

use crate::module::{
    ast::expanded::{ExpandedModule, ExpandedModuleItem},
    model::{
        alias_model::AliasModel, const_model::ConstModel, export_name::ExportName,
        function_model::FunctionModel, struct_model::StructModel,
    },
};

pub mod alias_model;
pub mod attributes;
pub mod const_model;
pub mod export_name;
pub mod function_model;
pub mod parameters;
pub mod struct_model;

pub struct JuliaModuleModel<'a> {
    pub init_fn: &'a Ident,
    pub exports: Vec<ItemModel<'a>>,
}
pub enum ItemModel<'a> {
    Const(ConstModel<'a>),
    Alias(AliasModel<'a>),
    Function(FunctionModel<'a>),
    Struct(StructModel<'a>),
}

impl<'a> JuliaModuleModel<'a> {
    pub fn from_expanded(module: &'a ExpandedModule) -> Result<Self> {
        let ExpandedModule { init_fn, items } = module;

        let mut consts: HashMap<ExportName, ConstModel> = HashMap::new();
        let mut structs: HashMap<ExportName, StructModel> = HashMap::new();
        let mut functions: HashMap<ExportName, FunctionModel> = HashMap::new();
        let mut aliases: HashMap<ExportName, AliasModel> = HashMap::new();

        for item in items {
            match item {
                ExpandedModuleItem::Function(expanded_function) => {
                    let model = FunctionModel::from_expanded(expanded_function)?;
                    if consts.contains_key(&model.export_name) {
                        Err(Error::new_spanned(
                            model.export_name,
                            "Function already exported as a constant",
                        ))?
                    } else {
                        let entry = functions.entry(model.export_name.clone());
                        match entry {
                            Entry::Occupied(mut entry) => entry.get_mut().merge(model)?,
                            Entry::Vacant(entry) => {
                                entry.insert(model);
                            }
                        }
                    }
                }
                ExpandedModuleItem::Struct(expanded_struct) => {
                    let model = StructModel::from_expanded(expanded_struct)?;
                    if consts.contains_key(&model.export_name)
                        || aliases.contains_key(&model.export_name)
                    {
                        Err(Error::new_spanned(
                            model.export_name,
                            "Struct already exported as a constant or an alias",
                        ))?
                    } else {
                        let entry = structs.entry(model.export_name.clone());
                        match entry {
                            Entry::Occupied(mut entry) => entry.get_mut().merge(model)?,
                            Entry::Vacant(entry) => {
                                entry.insert(model);
                            }
                        }
                    }
                }
                ExpandedModuleItem::Const(expanded_const) => {
                    let model = ConstModel::from_expanded(expanded_const)?;

                    if consts.contains_key(&model.export_name)
                        || structs.contains_key(&model.export_name)
                        || functions.contains_key(&model.export_name)
                        || aliases.contains_key(&model.export_name)
                    {
                        Err(Error::new_spanned(
                            model.export_name,
                            "Constant already exported",
                        ))?
                    } else {
                        consts.insert(model.export_name.clone(), model);
                    }
                }
                ExpandedModuleItem::Alias(expanded_alias) => {
                    let model = AliasModel::from_expanded(expanded_alias)?;
                    if consts.contains_key(&model.export_name)
                        || aliases.contains_key(&model.export_name)
                    {
                        Err(Error::new_spanned(
                            model.export_name,
                            "Alias already exported or exported as a contant",
                        ))?
                    } else {
                        aliases.insert(model.export_name.clone(), model);
                    }
                }
            }
        }

        let n_consts = consts.len();
        let n_functions = functions.len();
        let n_structs = structs.len();
        let n_aliases = aliases.len();
        let mut exports = Vec::with_capacity(n_consts + n_functions + n_structs + n_aliases);
        exports.extend(consts.into_values().map(ItemModel::Const));
        exports.extend(functions.into_values().map(ItemModel::Function));
        exports.extend(structs.into_values().map(ItemModel::Struct));
        exports.extend(aliases.into_values().map(ItemModel::Alias));

        Ok(JuliaModuleModel { init_fn, exports })
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use crate::{
        model::ItemModel,
        module::{
            ast::{expanded::ExpandedModule, raw::JuliaModuleAst},
            model::JuliaModuleModel,
        },
    };

    #[test]
    fn structs_are_merged() {
        let ast: JuliaModuleAst = parse_quote! {
            become init_fn;

            struct Foo<f32>;
            struct Foo<f64>;
        };

        let expanded = ExpandedModule::from_ast(ast).unwrap();
        let model = JuliaModuleModel::from_expanded(&expanded).unwrap();
        assert_eq!(model.exports.len(), 1);

        match &model.exports[0] {
            ItemModel::Struct(struct_model) => {
                assert_eq!(struct_model.kinds.variants.len(), 2);
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn structs_are_merged_with_repeated() {
        let ast: JuliaModuleAst = parse_quote! {
            become init_fn;

            struct Foo<f32>;

            for T in [f64] {
                struct Foo<T>;
            };

            for T in [usize] {
                struct Foo<T>;
            }
        };

        let expanded = ExpandedModule::from_ast(ast).unwrap();
        let model = JuliaModuleModel::from_expanded(&expanded).unwrap();
        assert_eq!(model.exports.len(), 1);

        match &model.exports[0] {
            ItemModel::Struct(struct_model) => {
                assert_eq!(struct_model.kinds.variants.len(), 3);
            }
            _ => assert!(false),
        }
    }
}
