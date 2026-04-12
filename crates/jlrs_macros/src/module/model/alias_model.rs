use syn::{Error, Result, Type, spanned::Spanned};

use crate::module::{
    ast::expanded::expanded_alias::ExpandedAlias,
    model::{
        attributes::{Attributes, Documentation},
        export_name::ExportName,
    },
};

pub struct AliasModel<'a> {
    pub _public: bool,
    pub export_name: ExportName<'a>,
    pub documentation: Option<Documentation<'a>>,
    pub ty: &'a Type,
}

impl<'a> AliasModel<'a> {
    pub fn from_expanded(expanded: &'a ExpandedAlias) -> Result<Self> {
        let ExpandedAlias {
            public,
            name,
            attrs,
            ty,
        } = expanded;

        let public = public.is_some();
        let export_name = ExportName::from_name(name);
        let attrs = Attributes::from_attributes(attrs)?;

        if attrs.attrs.len() != 0 || attrs.gc_safe {
            Err(Error::new(
                attrs.attrs.first().span(),
                "Aliases only support doc attributes",
            ))?
        }

        Ok(AliasModel {
            _public: public,
            export_name,
            documentation: attrs.documentation,
            ty,
        })
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use crate::{
        ast::expanded::ExpandedModuleItem,
        module::{
            ast::{expanded::ExpandedModule, raw::JuliaModuleAst},
            model::alias_model::AliasModel,
        },
    };

    #[test]
    fn documented_alias() {
        let ast: JuliaModuleAst = parse_quote! {
            become init_fn;

            /// Foo
            type Foo = Bar;
        };

        let mut expanded = ExpandedModule::from_ast(ast).unwrap();
        assert_eq!(expanded.items.len(), 1);

        match expanded.items.pop().unwrap() {
            ExpandedModuleItem::Alias(expanded_alias) => {
                let model = AliasModel::from_expanded(&expanded_alias).unwrap();
                assert!(model.documentation.is_some());
                assert!(!model._public);
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn documented_pub_alias() {
        let ast: JuliaModuleAst = parse_quote! {
            become init_fn;

            /// Foo
            pub type Foo = Bar;
        };

        let mut expanded = ExpandedModule::from_ast(ast).unwrap();
        assert_eq!(expanded.items.len(), 1);

        match expanded.items.pop().unwrap() {
            ExpandedModuleItem::Alias(expanded_alias) => {
                let model = AliasModel::from_expanded(&expanded_alias).unwrap();
                assert!(model.documentation.is_some());
                assert!(model._public);
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn reject_gc_safe_alias() {
        let ast: JuliaModuleAst = parse_quote! {
            become init_fn;

            #[gc_safe]
            type Foo = Bar;
        };

        let mut expanded = ExpandedModule::from_ast(ast).unwrap();
        assert_eq!(expanded.items.len(), 1);

        match expanded.items.pop().unwrap() {
            ExpandedModuleItem::Alias(expanded_alias) => {
                let model = AliasModel::from_expanded(&expanded_alias);
                assert!(model.is_err());
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn reject_alias_with_random_attrs() {
        let ast: JuliaModuleAst = parse_quote! {
            become init_fn;

            #[random]
            type Foo = Bar;
        };

        let mut expanded = ExpandedModule::from_ast(ast).unwrap();
        assert_eq!(expanded.items.len(), 1);

        match expanded.items.pop().unwrap() {
            ExpandedModuleItem::Alias(expanded_alias) => {
                let model = AliasModel::from_expanded(&expanded_alias);
                assert!(model.is_err());
            }
            _ => assert!(false),
        }
    }
}
