use syn::{Error, Ident, Result, Type, spanned::Spanned};

use crate::module::{
    ast::expanded::expanded_const::ExpandedConst,
    model::{
        attributes::{Attributes, Documentation},
        export_name::ExportName,
    },
};

pub struct ConstModel<'a> {
    pub _public: bool,
    pub original_name: &'a Ident,
    pub export_name: ExportName<'a>,
    pub documentation: Option<Documentation<'a>>,
    pub ty: &'a Type,
}

impl<'a> ConstModel<'a> {
    pub fn from_expanded(expanded: &'a ExpandedConst) -> Result<Self> {
        let ExpandedConst {
            public,
            name,
            attrs,
            ty,
            name_override,
        } = expanded;

        let public = public.is_some();
        let original_name = name;
        let export_name = ExportName::from_name_or_local(original_name, name_override.as_ref());
        let attrs = Attributes::from_attributes(attrs)?;

        if attrs.attrs.len() != 0 || attrs.gc_safe {
            Err(Error::new(
                attrs.attrs.first().span(),
                "Exported constants only support doc attributes",
            ))?
        }

        Ok(ConstModel {
            _public: public,
            original_name,
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
            model::const_model::ConstModel,
        },
    };

    #[test]
    fn documented_const() {
        let ast: JuliaModuleAst = parse_quote! {
            become init_fn;

            /// Foo
            const FOO: usize;
        };

        let mut expanded = ExpandedModule::from_ast(ast).unwrap();
        assert_eq!(expanded.items.len(), 1);

        match expanded.items.pop().unwrap() {
            ExpandedModuleItem::Const(expanded_const) => {
                let model = ConstModel::from_expanded(&expanded_const).unwrap();
                assert!(model.documentation.is_some());
                assert!(!model._public);
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn documented_pub_const() {
        let ast: JuliaModuleAst = parse_quote! {
            become init_fn;

            /// Foo
            pub const FOO: usize;
        };

        let mut expanded = ExpandedModule::from_ast(ast).unwrap();
        assert_eq!(expanded.items.len(), 1);

        match expanded.items.pop().unwrap() {
            ExpandedModuleItem::Const(expanded_const) => {
                let model = ConstModel::from_expanded(&expanded_const).unwrap();
                assert!(model.documentation.is_some());
                assert!(model._public);
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn reject_gc_safe_const() {
        let ast: JuliaModuleAst = parse_quote! {
            become init_fn;

            #[gc_safe]
            pub const FOO: usize;
        };

        let mut expanded = ExpandedModule::from_ast(ast).unwrap();
        assert_eq!(expanded.items.len(), 1);

        match expanded.items.pop().unwrap() {
            ExpandedModuleItem::Const(expanded_const) => {
                let model = ConstModel::from_expanded(&expanded_const);
                assert!(model.is_err());
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn reject_const_with_random_attrs() {
        let ast: JuliaModuleAst = parse_quote! {
            become init_fn;

            #[random]
            pub const FOO: usize;
        };

        let mut expanded = ExpandedModule::from_ast(ast).unwrap();
        assert_eq!(expanded.items.len(), 1);

        match expanded.items.pop().unwrap() {
            ExpandedModuleItem::Const(expanded_const) => {
                let model = ConstModel::from_expanded(&expanded_const);
                assert!(model.is_err());
            }
            _ => assert!(false),
        }
    }
}
