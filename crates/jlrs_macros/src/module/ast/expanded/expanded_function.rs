//! Expanded `InAst` or `FnAst`-node

use quote::ToTokens;
use syn::{Attribute, Path, Signature, Token};

use crate::module::ast::{
    expanded::{Environment, expanded_as::ExpandedAs},
    raw::{fn_ast::FnAst, in_ast::InAst, use_ast::UseAst},
};

#[derive(Clone)]
pub enum OriginalAst {
    Function(Option<Token![pub]>, FnAst),
    Method(InAst),
}

impl ToTokens for OriginalAst {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            OriginalAst::Function(pub_token, fn_ast) => {
                pub_token.to_tokens(tokens);
                fn_ast.to_tokens(tokens);
            }
            OriginalAst::Method(in_ast) => in_ast.to_tokens(tokens),
        }
    }
}

#[derive(Clone)]
pub struct ExpandedFunction {
    pub original: OriginalAst,
    pub public: Option<Token![pub]>,
    pub parent: Option<Path>,
    pub signature: Signature,
    pub attrs: Vec<Attribute>,
    pub type_var_env: Option<UseAst>,
    pub environment: Option<Environment>,
    pub name_override: Option<ExpandedAs>,
}

impl ExpandedFunction {
    pub fn from_fn_ast(
        fn_ast: FnAst,
        public: Option<Token![pub]>,
        attrs: Vec<Attribute>,
        environment: Option<Environment>,
    ) -> Self {
        ExpandedFunction {
            original: OriginalAst::Function(public.clone(), fn_ast.clone()),
            public,
            parent: None,
            signature: fn_ast.signature,
            attrs,
            type_var_env: fn_ast.type_var_env,
            environment,
            name_override: fn_ast.name_override.map(ExpandedAs::from_ast),
        }
    }

    pub fn from_in_ast(
        ast: InAst,
        attrs: Vec<Attribute>,
        environment: Option<Environment>,
    ) -> Self {
        let original = ast.clone();
        let fn_ast = ast.function_ast;
        let public = ast.pub_token;
        let parent = ast.parent;

        ExpandedFunction {
            original: OriginalAst::Method(original),
            public,
            parent: Some(parent),
            signature: fn_ast.signature,
            attrs,
            type_var_env: fn_ast.type_var_env,
            environment,
            name_override: fn_ast.name_override.map(ExpandedAs::from_ast),
        }
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use crate::module::ast::{
        expanded::expanded_function::ExpandedFunction,
        raw::{fn_ast::FnAst, in_ast::InAst},
    };
    #[test]
    fn expand_function() {
        let ast: FnAst = parse_quote! { fn foo() };
        let expanded = ExpandedFunction::from_fn_ast(ast, None, vec![], None);
        assert!(expanded.parent.is_none());
    }

    #[test]
    fn expand_pub_method() {
        let ast: InAst = parse_quote! { in Foo pub fn foo() };
        let expanded = ExpandedFunction::from_in_ast(ast, vec![], None);
        assert!(expanded.parent.is_some());
        assert!(expanded.public.is_some());
    }

    #[test]
    fn expand_method() {
        let ast: InAst = parse_quote! { in Foo fn foo() };
        let expanded = ExpandedFunction::from_in_ast(ast, vec![], None);
        assert!(expanded.parent.is_some());
        assert!(expanded.public.is_none());
    }
}
