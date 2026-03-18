//! Expanded AST of `julia_module!`
//!
//! In the expanded AST, all `PubAst` and `WithAttributes` nodes are moved to the affected item,
//! and the content of `ForAst` nodes has been expanded.

use syn::{Attribute, Error, Ident, Result};

use crate::{
    ast::expanded::environment::{Environment, Parameter},
    module::ast::{
        expanded::{
            expanded_alias::ExpandedAlias, expanded_const::ExpandedConst,
            expanded_function::ExpandedFunction, expanded_struct::ExpandedStruct,
        },
        raw::{
            JuliaModuleAst, ModuleItem,
            for_ast::{ForAst, ForItem, ForItemInner},
            pub_ast::{PubAst, PubItem},
            with_attributes::{Attributed, WithAttributes},
        },
    },
};

pub mod environment;
pub mod expanded_alias;
pub mod expanded_as;
pub mod expanded_const;
pub mod expanded_function;
pub mod expanded_struct;

pub struct ExpandedModule {
    pub init_fn: Ident,
    pub items: Vec<ExpandedModuleItem>,
}

impl ExpandedModule {
    pub fn from_ast(ast: JuliaModuleAst) -> Result<Self> {
        let JuliaModuleAst {
            init_fn,
            _semicolon: _,
            items,
        } = ast;

        let init_fn = init_fn.init_fn;
        let mut expanded = vec![];
        for item in items {
            expand_item(&mut expanded, item)?;
        }

        Ok(ExpandedModule {
            init_fn,
            items: expanded,
        })
    }
}

pub enum ExpandedModuleItem {
    Const(ExpandedConst),
    Function(ExpandedFunction),
    Struct(ExpandedStruct),
    Alias(ExpandedAlias),
}

fn expand_pub_item(
    pub_ast: PubAst,
    attrs: Vec<Attribute>,
    environment: Option<Environment>,
) -> Result<ExpandedModuleItem> {
    let item = pub_ast.item;
    let pub_token = Some(pub_ast.pub_token);
    let item = match item {
        PubItem::Const(const_ast) => {
            if environment.is_some() {
                Err(Error::new(
                    const_ast.name.span(),
                    "Constant cannot be exported from an environment",
                ))?;
            }
            ExpandedModuleItem::Const(ExpandedConst::from_ast(const_ast, pub_token, attrs)?)
        }
        PubItem::Fn(fn_ast) => ExpandedModuleItem::Function(ExpandedFunction::from_fn_ast(
            fn_ast,
            pub_token,
            attrs,
            environment,
        )),
        PubItem::Struct(struct_ast) => ExpandedModuleItem::Struct(ExpandedStruct::from_ast(
            struct_ast,
            pub_token,
            attrs,
            environment,
        )?),
        PubItem::Type(type_ast) => {
            if environment.is_some() {
                Err(Error::new(
                    type_ast.name.span(),
                    "Alias cannot be exported from an environment",
                ))?;
            }
            ExpandedModuleItem::Alias(ExpandedAlias::from_ast(type_ast, pub_token, attrs))
        }
    };

    Ok(item)
}

fn expand_with_attributes(
    ast: WithAttributes,
    environment: Option<Environment>,
) -> Result<ExpandedModuleItem> {
    let WithAttributes { attributes, item } = ast;
    let res = match item {
        Attributed::Const(const_ast) => {
            if environment.is_some() {
                Err(Error::new(
                    const_ast.name.span(),
                    "Constant cannot be exported from an environment",
                ))?;
            }
            ExpandedModuleItem::Const(ExpandedConst::from_ast(const_ast, None, attributes)?)
        }
        Attributed::Fn(fn_ast) => ExpandedModuleItem::Function(ExpandedFunction::from_fn_ast(
            fn_ast,
            None,
            attributes,
            environment,
        )),
        Attributed::In(in_ast) => ExpandedModuleItem::Function(ExpandedFunction::from_in_ast(
            in_ast,
            attributes,
            environment,
        )),
        Attributed::Struct(struct_ast) => ExpandedModuleItem::Struct(ExpandedStruct::from_ast(
            struct_ast,
            None,
            attributes,
            environment,
        )?),
        Attributed::Pub(pub_ast) => expand_pub_item(pub_ast, attributes, environment)?,
        Attributed::Type(type_ast) => {
            if environment.is_some() {
                Err(Error::new(
                    type_ast.name.span(),
                    "Alias cannot be exported from an environment",
                ))?;
            }
            ExpandedModuleItem::Alias(ExpandedAlias::from_ast(type_ast, None, attributes))
        }
    };

    Ok(res)
}

fn expand_for(
    expanded: &mut Vec<ExpandedModuleItem>,
    for_ast: ForAst,
    environment: Option<Environment>,
) -> Result<()> {
    let parameter = Parameter::from_for_ast(&for_ast);
    let environment = if let Some(environment) = environment {
        environment.add_parameter(parameter)
    } else {
        parameter.into_environment()
    };

    for item in for_ast.items {
        let env = Some(environment.clone());
        match item {
            ForItem::Entry(for_item_inner) => match for_item_inner {
                ForItemInner::Fn(fn_ast) => {
                    let item = ExpandedModuleItem::Function(ExpandedFunction::from_fn_ast(
                        fn_ast,
                        None,
                        vec![],
                        env,
                    ));
                    expanded.push(item);
                }
                ForItemInner::In(in_ast) => {
                    let item = ExpandedModuleItem::Function(ExpandedFunction::from_in_ast(
                        in_ast,
                        vec![],
                        env,
                    ));
                    expanded.push(item);
                }
                ForItemInner::Struct(struct_ast) => {
                    let item = ExpandedModuleItem::Struct(ExpandedStruct::from_ast(
                        struct_ast,
                        None,
                        vec![],
                        env,
                    )?);
                    expanded.push(item);
                }
                ForItemInner::Pub(pub_ast) => {
                    let item = expand_pub_item(pub_ast, vec![], env)?;
                    expanded.push(item)
                }
                ForItemInner::WithAttributes(with_attributes) => {
                    let item = expand_with_attributes(with_attributes, env)?;
                    expanded.push(item);
                }
            },
            ForItem::Nested(for_ast) => expand_for(expanded, for_ast, env)?,
        }
    }

    Ok(())
}

fn expand_item(expanded: &mut Vec<ExpandedModuleItem>, item: ModuleItem) -> Result<()> {
    match item {
        ModuleItem::Const(const_ast) => {
            let item = ExpandedModuleItem::Const(ExpandedConst::from_ast(const_ast, None, vec![])?);
            expanded.push(item);
        }
        ModuleItem::Fn(fn_ast) => {
            let item = ExpandedModuleItem::Function(ExpandedFunction::from_fn_ast(
                fn_ast,
                None,
                vec![],
                None,
            ));
            expanded.push(item);
        }
        ModuleItem::In(in_ast) => {
            let item =
                ExpandedModuleItem::Function(ExpandedFunction::from_in_ast(in_ast, vec![], None));
            expanded.push(item);
        }
        ModuleItem::Struct(struct_ast) => {
            let item = ExpandedModuleItem::Struct(ExpandedStruct::from_ast(
                struct_ast,
                None,
                vec![],
                None,
            )?);
            expanded.push(item);
        }
        ModuleItem::Type(type_ast) => {
            let item = ExpandedModuleItem::Alias(ExpandedAlias::from_ast(type_ast, None, vec![]));
            expanded.push(item);
        }
        ModuleItem::For(for_ast) => expand_for(expanded, for_ast, None)?,
        ModuleItem::Pub(pub_ast) => {
            let item = expand_pub_item(pub_ast, vec![], None)?;
            expanded.push(item);
        }
        ModuleItem::WithAttributes(with_attributes) => {
            let item = expand_with_attributes(with_attributes, None)?;
            expanded.push(item);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use crate::{
        ast::expanded::ExpandedModuleItem,
        module::ast::{
            expanded::{ExpandedModule, expand_for, expand_pub_item, expand_with_attributes},
            raw::{
                JuliaModuleAst, for_ast::ForAst, pub_ast::PubAst, with_attributes::WithAttributes,
            },
        },
    };

    #[test]
    fn expand_pub_alias() {
        let ast: PubAst = parse_quote! {
            pub type Foo = Bar
        };

        let expanded = expand_pub_item(ast, vec![], None);
        assert!(expanded.is_ok());
    }

    #[test]
    fn expand_pub_alias_with_attrs() {
        let ast: WithAttributes = parse_quote! {
            /// doc
            pub type Foo = Bar
        };

        let expanded = expand_with_attributes(ast, None);
        assert!(expanded.is_ok());
    }

    #[test]
    fn expand_pub_const() {
        let ast: PubAst = parse_quote! {
            pub const Foo: usize
        };

        let expanded = expand_pub_item(ast, vec![], None);
        assert!(expanded.is_ok());
    }

    #[test]
    fn expand_pub_const_with_attrs() {
        let ast: WithAttributes = parse_quote! {
            /// doc
            pub const Foo: usize
        };

        let expanded = expand_with_attributes(ast, None);
        assert!(expanded.is_ok());
    }

    #[test]
    fn expand_pub_function() {
        let ast: PubAst = parse_quote! {
            pub fn foo()
        };

        let expanded = expand_pub_item(ast, vec![], None);
        assert!(expanded.is_ok());
    }

    #[test]
    fn expand_pub_function_with_attrs() {
        let ast: WithAttributes = parse_quote! {
            /// doc
            pub fn foo()
        };

        let expanded = expand_with_attributes(ast, None);
        assert!(expanded.is_ok());
    }

    #[test]
    fn expand_pub_struct() {
        let ast: PubAst = parse_quote! {
            pub struct Foo
        };

        let expanded = expand_pub_item(ast, vec![], None);
        assert!(expanded.is_ok());
    }

    #[test]
    fn expand_pub_struct_with_attrs() {
        let ast: WithAttributes = parse_quote! {
            /// doc
            pub struct Foo
        };

        let expanded = expand_with_attributes(ast, None);
        assert!(expanded.is_ok());
    }

    #[test]
    fn expand_empty_environment() {
        let ast: ForAst = parse_quote! {
            for T in [f32, f64] {

            }
        };

        let mut expanded_items = Vec::new();
        let expanded = expand_for(&mut expanded_items, ast, None);
        assert!(expanded.is_ok());
        assert_eq!(expanded_items.len(), 0);
    }

    #[test]
    fn expand_environment() {
        let ast: ForAst = parse_quote! {
            for T in [f32, f64] {
                struct Foo<T>
            }
        };

        let mut expanded_items = Vec::new();
        let expanded = expand_for(&mut expanded_items, ast, None);
        assert!(expanded.is_ok());
        assert_eq!(expanded_items.len(), 1);

        let expanded_item = expanded_items.pop().unwrap();
        match expanded_item {
            ExpandedModuleItem::Struct(expanded_struct) => {
                let parameters = expanded_struct.environment.unwrap().parameters;
                assert_eq!(parameters.len(), 1);
                let parameter = &parameters[0];
                assert_eq!(parameter.name.to_string(), "T");
                assert_eq!(parameter.types.len(), 2);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn expand_nested_environment() {
        let ast: ForAst = parse_quote! {
            for T in [f32, f64] {
                for U in [f32, f64, usize] {
                    struct Foo<T, U>
                }
            }
        };

        let mut expanded_items = Vec::new();
        let expanded = expand_for(&mut expanded_items, ast, None);
        assert!(expanded.is_ok());
        assert_eq!(expanded_items.len(), 1);

        let expanded_item = expanded_items.pop().unwrap();
        match expanded_item {
            ExpandedModuleItem::Struct(expanded_struct) => {
                let parameters = expanded_struct.environment.unwrap().parameters;
                assert_eq!(parameters.len(), 2);
                let parameter = &parameters[0];
                assert_eq!(parameter.name.to_string(), "T");
                assert_eq!(parameter.types.len(), 2);
                let parameter = &parameters[1];
                assert_eq!(parameter.name.to_string(), "U");
                assert_eq!(parameter.types.len(), 3);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn expand_module() {
        let ast: JuliaModuleAst = parse_quote! {
            become foo;

            fn foo_bar();
        };

        let expanded = ExpandedModule::from_ast(ast);
        assert!(expanded.is_ok());
        let expanded = expanded.unwrap();
        assert_eq!(expanded.items.len(), 1);
        assert_eq!(expanded.init_fn.to_string(), "foo")
    }
}
