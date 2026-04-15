//! Expanded `AsAst`-node.

use quote::ToTokens;
use syn::{Error, Ident, Result, Token, punctuated::Punctuated};

use crate::module::ast::raw::as_ast::AsAst;

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct LocalName {
    pub name_override: Ident,
    pub exclamation_mark_token: Option<Token![!]>,
}

impl LocalName {
    pub fn from_ast(ast: AsAst) -> Result<Self> {
        if ast.name_override.len() != 1 {
            Err(Error::new_spanned(
                ast.clone(),
                "Item cannot be exported in another module",
            ))?
        }

        let name_override = ast.name_override.first().unwrap().clone();
        Ok(LocalName {
            name_override,
            exclamation_mark_token: ast.exclamation_mark_token,
        })
    }

    pub fn from_ident(ident: Ident) -> Self {
        LocalName {
            name_override: ident,
            exclamation_mark_token: None,
        }
    }

    pub fn name_string(&self) -> String {
        let s = &self.name_override;
        match self.exclamation_mark_token {
            Some(_) => format!("{s}!"),
            None => s.to_string(),
        }
    }
}

impl ToTokens for LocalName {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.name_override.to_tokens(tokens);
        self.exclamation_mark_token.to_tokens(tokens);
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct GlobalName {
    pub name_override: Punctuated<Ident, Token![.]>,
    pub exclamation_mark_token: Option<Token![!]>,
}

impl GlobalName {
    pub fn from_ast(ast: AsAst) -> Self {
        GlobalName {
            name_override: ast.name_override,
            exclamation_mark_token: ast.exclamation_mark_token,
        }
    }

    pub fn name_string(&self) -> String {
        let s = self.name_override.last().unwrap();
        match self.exclamation_mark_token {
            Some(_) => format!("{s}!"),
            None => s.to_string(),
        }
    }
}

impl ToTokens for GlobalName {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.name_override.to_tokens(tokens);
        self.exclamation_mark_token.to_tokens(tokens);
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub enum ExpandedAs {
    Local(LocalName),
    Global(GlobalName),
}

impl ExpandedAs {
    pub fn from_ast(ast: AsAst) -> Self {
        if ast.name_override.len() == 1 {
            ExpandedAs::Local(LocalName::from_ast(ast).unwrap())
        } else {
            ExpandedAs::Global(GlobalName::from_ast(ast))
        }
    }

    pub fn name_string(&self) -> String {
        match self {
            ExpandedAs::Local(local_name) => local_name.name_string(),
            ExpandedAs::Global(global_name) => global_name.name_string(),
        }
    }

    pub fn is_global(&self)  -> bool {
        match self {
            ExpandedAs::Local(_) => false,
            ExpandedAs::Global(_) => true,
        }
    }
}

impl ToTokens for ExpandedAs {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            ExpandedAs::Local(local_name) => local_name.to_tokens(tokens),
            ExpandedAs::Global(global_name) => global_name.to_tokens(tokens),
        }
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use crate::module::ast::{
        expanded::expanded_as::{ExpandedAs, LocalName},
        raw::as_ast::AsAst,
    };

    #[test]
    fn expand_local() {
        let as_ast: AsAst = parse_quote! { as Foo };
        let name_override = ExpandedAs::from_ast(as_ast);

        match name_override {
            ExpandedAs::Local(local_name) => {
                assert_eq!(local_name.name_override.to_string(), "Foo")
            }
            ExpandedAs::Global(_global_name) => assert!(false),
        }
    }

    #[test]
    fn expand_global() {
        let as_ast: AsAst = parse_quote! { as Foo.Bar };
        let name_override = ExpandedAs::from_ast(as_ast);

        match name_override {
            ExpandedAs::Local(_local_name) => assert!(false),
            ExpandedAs::Global(global_name) => assert_eq!(global_name.name_override.len(), 2),
        }
    }

    #[test]
    fn expand_global_as_local_error() {
        let as_ast: AsAst = parse_quote! { as Foo.Bar };
        let name_override = LocalName::from_ast(as_ast);
        assert!(name_override.is_err());
    }
}
