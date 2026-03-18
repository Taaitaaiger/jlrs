use syn::{Error, FnArg, Path, Result, Signature, parse_quote, spanned::Spanned};

use crate::{
    model::parameters::{Apply, ResolvedParameterList},
    module::{
        ast::{
            expanded::expanded_function::{ExpandedFunction, OriginalAst},
            raw::use_ast::UseAst,
        },
        model::{
            attributes::{Attributes, Documentation},
            export_name::ExportName,
        },
    },
};

pub struct FunctionModel<'a> {
    pub public: bool,
    pub documentation: Option<Documentation<'a>>,
    pub export_name: ExportName<'a>,
    pub variants: Vec<FunctionVariant<'a>>,
}

impl<'a> FunctionModel<'a> {
    pub fn from_expanded(expanded: &'a ExpandedFunction) -> Result<Self> {
        let public = expanded.public.is_some();
        let original_name = &expanded.signature.ident;
        let export_name =
            ExportName::from_name_or_override(original_name, expanded.name_override.as_ref());
        let attrs = Attributes::from_attributes(&expanded.attrs)?;

        if attrs.attrs.len() != 0 {
            Err(Error::new(
                attrs.attrs.first().span(),
                "Exported functions only support doc attributes, #[gc_safe], and #[untracked_self]",
            ))?
        }

        let variants = FunctionVariant::from_expanded(&expanded, &attrs)?;

        Ok(FunctionModel {
            public,
            documentation: attrs.documentation,
            export_name,
            variants,
        })
    }

    pub fn merge(&mut self, other: Self) -> Result<()> {
        if self.public != other.public {
            Err(Error::new_spanned(
                &other.variants[0].original,
                "visibility mismatch",
            ))?;
        }

        match (&mut self.documentation, other.documentation) {
            (Some(_), None) => {}
            (current @ None, other) => *current = other,
            (Some(_), Some(_)) => {
                Err(Error::new_spanned(
                    &other.variants[0].original,
                    "multiple docstrings",
                ))?;
            }
        }

        self.variants.extend(other.variants);
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionKind {
    SelfMethod { untracked_self: bool, parent: Path },
    RefSelfMethod { untracked_self: bool, parent: Path },
    MutRefSelfMethod { untracked_self: bool, parent: Path },
    AssocFunction { parent: Path },
    Function,
}

impl FunctionKind {
    pub fn new(
        unresolved_signature: &Signature,
        parent: Option<Path>,
        attributes: &Attributes,
    ) -> Result<Self> {
        let kind = match unresolved_signature.inputs.first() {
            None | Some(FnArg::Typed(_)) => {
                if let Some(parent) = parent {
                    FunctionKind::AssocFunction { parent }
                } else {
                    FunctionKind::Function
                }
            }
            Some(FnArg::Receiver(receiver)) => {
                let parent = if let Some(parent) = parent {
                    parent
                } else {
                    Err(Error::new(
                        unresolved_signature.span(),
                        "method exported as a function, prefix with `in ParentType` to export as a method",
                    ))?
                };

                let mutable = receiver.mutability.is_some();
                let reference = receiver.reference.is_some();
                let untracked_self = attributes.untracked_self;
                match (mutable, reference) {
                    (true, true) => FunctionKind::MutRefSelfMethod {
                        untracked_self,
                        parent,
                    },
                    (false, true) => FunctionKind::RefSelfMethod {
                        untracked_self,
                        parent,
                    },
                    _ => FunctionKind::SelfMethod {
                        untracked_self,
                        parent,
                    },
                }
            }
        };

        Ok(kind)
    }
}

pub struct FunctionVariant<'a> {
    pub original: &'a OriginalAst,
    pub signature: Signature,
    pub gc_safe: bool,
    pub type_var_env: Option<&'a UseAst>,
    pub kind: FunctionKind,
}

impl<'a> FunctionVariant<'a> {
    fn from_expanded(expanded: &'a ExpandedFunction, attrs: &Attributes) -> Result<Vec<Self>> {
        let ExpandedFunction {
            original,
            public: _,
            parent,
            signature,
            attrs: _,
            type_var_env,
            environment,
            name_override: _,
        } = expanded;

        if let Some(env) = environment.as_ref() {
            let n_combinations = env.n_combinations();
            let mut variants = Vec::with_capacity(n_combinations);
            let mut resolved = ResolvedParameterList::new(env);

            for i in 0..n_combinations {
                resolved.resolve(i);

                let parent = parent
                    .as_ref()
                    .map(|parent| resolved.apply(parent))
                    .transpose()?;

                let kind = FunctionKind::new(&signature, parent.clone(), &attrs)?;
                let signature = resolved.apply_with_parent(signature, parent.as_ref())?;
                variants.push(FunctionVariant {
                    original: original,
                    signature,
                    gc_safe: attrs.gc_safe,
                    type_var_env: type_var_env.as_ref(),
                    kind,
                });
            }

            Ok(variants)
        } else {
            let kind = FunctionKind::new(&signature, parent.clone(), &attrs)?;
            let mut signature = signature.clone();

            match signature.inputs.first_mut() {
                Some(x) => {
                    let new = match x {
                        FnArg::Receiver(receiver) => {
                            let mutability = receiver.mutability.as_ref();
                            parse_quote! { #mutability this: ::jlrs::data::managed::value::typed::TypedValue<#parent> }
                        }
                        FnArg::Typed(pat_type) => FnArg::Typed(pat_type.clone()),
                    };

                    *x = new;
                }
                None => (),
            }

            Ok(vec![FunctionVariant {
                original: original,
                signature: signature,
                gc_safe: attrs.gc_safe,
                type_var_env: type_var_env.as_ref(),
                kind,
            }])
        }
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use crate::{
        ast::expanded::ExpandedModuleItem,
        module::{
            ast::{expanded::ExpandedModule, raw::JuliaModuleAst},
            model::function_model::FunctionModel,
        },
    };

    #[test]
    fn documented_function() {
        let ast: JuliaModuleAst = parse_quote! {
            become init_fn;

            /// Foo
            fn foo()
        };

        let mut expanded = ExpandedModule::from_ast(ast).unwrap();
        assert_eq!(expanded.items.len(), 1);

        match expanded.items.pop().unwrap() {
            ExpandedModuleItem::Function(expanded_fn) => {
                let model = FunctionModel::from_expanded(&expanded_fn).unwrap();
                assert!(model.documentation.is_some());
                assert!(!model.public);
                assert_eq!(model.variants.len(), 1);
                assert!(!model.variants[0].gc_safe);
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn documented_pub_function() {
        let ast: JuliaModuleAst = parse_quote! {
            become init_fn;

            /// Foo
            pub fn foo()
        };

        let mut expanded = ExpandedModule::from_ast(ast).unwrap();
        assert_eq!(expanded.items.len(), 1);

        match expanded.items.pop().unwrap() {
            ExpandedModuleItem::Function(expanded_fn) => {
                let model = FunctionModel::from_expanded(&expanded_fn).unwrap();
                assert!(model.documentation.is_some());
                assert!(model.public);
                assert_eq!(model.variants.len(), 1);
                assert!(!model.variants[0].gc_safe);
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn documented_gc_safe_pub_function() {
        let ast: JuliaModuleAst = parse_quote! {
            become init_fn;

            /// Foo
            #[gc_safe]
            pub fn foo()
        };

        let mut expanded = ExpandedModule::from_ast(ast).unwrap();
        assert_eq!(expanded.items.len(), 1);

        match expanded.items.pop().unwrap() {
            ExpandedModuleItem::Function(expanded_fn) => {
                let model = FunctionModel::from_expanded(&expanded_fn).unwrap();
                assert!(model.documentation.is_some());
                assert!(model.public);
                assert_eq!(model.variants.len(), 1);
                assert!(model.variants[0].gc_safe);
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn reject_function_with_random_attr() {
        let ast: JuliaModuleAst = parse_quote! {
            become init_fn;

            /// Foo
            #[gc_safe]
            #[random]
            pub fn foo()
        };

        let mut expanded = ExpandedModule::from_ast(ast).unwrap();
        assert_eq!(expanded.items.len(), 1);

        match expanded.items.pop().unwrap() {
            ExpandedModuleItem::Function(expanded_fn) => {
                let model = FunctionModel::from_expanded(&expanded_fn);
                assert!(model.is_err());
            }
            _ => assert!(false),
        }
    }
}
