use syn::{Error, Path, Result, spanned::Spanned};

use crate::{
    ast::raw::struct_ast::StructAst,
    model::parameters::{Apply, ResolvedParameterList},
    module::{
        ast::expanded::{environment::Environment, expanded_struct::ExpandedStruct},
        model::{
            attributes::{Attributes, Documentation},
            export_name::ExportName,
        },
    },
};

pub struct StructModel<'a> {
    pub original: &'a StructAst,
    pub public: bool,
    pub documentation: Option<Documentation<'a>>,
    pub export_name: ExportName<'a>,
    pub kinds: StructVariants,
}

impl<'a> StructModel<'a> {
    pub fn from_expanded(expanded: &'a ExpandedStruct) -> Result<Self> {
        let ExpandedStruct {
            original,
            public,
            path,
            attrs,
            environment,
            name_override,
        } = expanded;

        let public = public.is_some();
        let type_name = &path.segments.last().unwrap().ident;
        let export_name = ExportName::from_name_or_local(type_name, name_override.as_ref());
        let attrs = Attributes::from_attributes(attrs)?;

        if attrs.attrs.len() != 0 || attrs.gc_safe {
            Err(Error::new(
                attrs.attrs.first().span(),
                "Exported structs only support doc attributes",
            ))?
        }

        let kinds = StructVariants::from_variant(&path, environment.as_ref())?;

        Ok(StructModel {
            original,
            public,
            documentation: attrs.documentation,
            export_name,
            kinds,
        })
    }

    pub fn merge(&mut self, other: Self) -> Result<()> {
        if self.public != other.public {
            Err(Error::new_spanned(&other.original, "visibility mismatch"))?;
        }

        match (&mut self.documentation, other.documentation) {
            (Some(_), None) => {}
            (current @ None, other) => *current = other,
            (Some(_), Some(_)) => Err(Error::new_spanned(&other.original, "multiple docstrings"))?,
        }

        self.kinds.merge(other.kinds);
        Ok(())
    }
}

pub struct StructVariants {
    pub key: Path,
    pub variants: Vec<Path>,
}

impl StructVariants {
    fn from_variant(path: &Path, env: Option<&Environment>) -> Result<Self> {
        if path.segments.last().unwrap().arguments.is_none() {
            if env.is_some() {
                Err(Error::new(
                    path.span(),
                    "Path has no generics but is exported with an environment",
                ))?
            }

            Ok(StructVariants {
                key: path.clone(),
                variants: vec![path.clone()],
            })
        } else if let Some(env) = env {
            let n_combinations = env.n_combinations();
            let mut resolved = ResolvedParameterList::new(env);
            let mut variants = Vec::with_capacity(n_combinations);
            for i in 0..n_combinations {
                resolved.resolve(i);
                let path = resolved.apply(path)?;
                variants.push(path);
            }

            Ok(StructVariants {
                key: variants[0].clone(),
                variants,
            })
        } else {
            Ok(StructVariants {
                key: path.clone(),
                variants: vec![path.clone()],
            })
        }
    }

    fn merge(&mut self, other: Self) {
        self.variants.extend(other.variants.into_iter());
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use crate::{
        ast::expanded::ExpandedModuleItem,
        model::struct_model::StructModel,
        module::ast::{expanded::ExpandedModule, raw::JuliaModuleAst},
    };

    #[test]
    fn documented_struct() {
        let ast: JuliaModuleAst = parse_quote! {
            become init_fn;

            /// Foo
            struct Foo;
        };

        let mut expanded = ExpandedModule::from_ast(ast).unwrap();
        assert_eq!(expanded.items.len(), 1);

        match expanded.items.pop().unwrap() {
            ExpandedModuleItem::Struct(expanded_struct) => {
                let model = StructModel::from_expanded(&expanded_struct).unwrap();
                assert!(model.documentation.is_some());
                assert!(!model.public);
                assert_eq!(model.kinds.variants.len(), 1);
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn documented_pub_struct() {
        let ast: JuliaModuleAst = parse_quote! {
            become init_fn;

            /// Foo
            pub struct Foo;
        };

        let mut expanded = ExpandedModule::from_ast(ast).unwrap();
        assert_eq!(expanded.items.len(), 1);

        match expanded.items.pop().unwrap() {
            ExpandedModuleItem::Struct(expanded_struct) => {
                let model = StructModel::from_expanded(&expanded_struct).unwrap();
                assert!(model.documentation.is_some());
                assert!(model.public);
                assert_eq!(model.kinds.variants.len(), 1);
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn reject_struct_with_random_attr() {
        let ast: JuliaModuleAst = parse_quote! {
            become init_fn;

            #[random]
            pub struct Foo;
        };

        let mut expanded = ExpandedModule::from_ast(ast).unwrap();
        assert_eq!(expanded.items.len(), 1);

        match expanded.items.pop().unwrap() {
            ExpandedModuleItem::Struct(expanded_struct) => {
                let model = StructModel::from_expanded(&expanded_struct);
                assert!(model.is_err());
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn generic_struct() {
        let ast: JuliaModuleAst = parse_quote! {
            become init_fn;

            for T in [f32, f64] {
                /// Foo
                pub struct Foo<T>;
            }
        };

        let mut expanded = ExpandedModule::from_ast(ast).unwrap();
        assert_eq!(expanded.items.len(), 1);

        match expanded.items.pop().unwrap() {
            ExpandedModuleItem::Struct(expanded_struct) => {
                let model = StructModel::from_expanded(&expanded_struct).unwrap();
                assert!(model.documentation.is_some());
                assert!(model.public);
                assert_eq!(model.kinds.variants.len(), 2);
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn standalone_generic_struct() {
        let ast: JuliaModuleAst = parse_quote! {
            become init_fn;

            struct Foo<f32>;
        };

        let mut expanded = ExpandedModule::from_ast(ast).unwrap();
        assert_eq!(expanded.items.len(), 1);

        match expanded.items.pop().unwrap() {
            ExpandedModuleItem::Struct(expanded_struct) => {
                let model = StructModel::from_expanded(&expanded_struct).unwrap();
                assert!(model.documentation.is_none());
                assert!(!model.public);
                assert_eq!(model.kinds.variants.len(), 1);
            }
            _ => assert!(false),
        }
    }
}
