//! Expanded `StructAst`-node

use quote::ToTokens;
use syn::{Attribute, Path, Result, Token};

use crate::{
    ast::expanded::expanded_as::ExpandedAs,
    module::ast::{
        expanded::{Environment, expanded_as::LocalName},
        raw::struct_ast::StructAst,
    },
};

#[derive(Clone)]
pub struct ExpandedStruct {
    pub original: StructAst,
    pub public: Option<Token![pub]>,
    pub path: Path,
    pub attrs: Vec<Attribute>,
    pub environment: Option<Environment>,
    pub name_override: Option<ExpandedAs>,
}

impl ExpandedStruct {
    pub fn from_ast(
        ast: StructAst,
        public: Option<Token![pub]>,
        attrs: Vec<Attribute>,
        environment: Option<Environment>,
    ) -> Result<Self> {
        let StructAst {
            struct_token: _,
            path,
            name_override,
        } = ast.clone();
        let name_override = name_override
            .map(LocalName::from_ast)
            .transpose()?
            .map(ExpandedAs::Local);

        Ok(ExpandedStruct {
            original: ast,
            public,
            path,
            attrs,
            environment,
            name_override,
        })
    }
}

impl ToTokens for ExpandedStruct {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.public.to_tokens(tokens);
        self.original.to_tokens(tokens);
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use crate::module::ast::{
        expanded::expanded_struct::ExpandedStruct, raw::struct_ast::StructAst,
    };

    #[test]
    fn expand_renamed_struct() {
        let ast: StructAst = parse_quote! { struct Foo as Bar };
        let expanded = ExpandedStruct::from_ast(ast, None, vec![], None);
        assert!(expanded.is_ok())
    }

    #[test]
    fn expand_global_name_struct_error() {
        let ast: StructAst = parse_quote! { struct Foo as Bar.Baz };
        let expanded = ExpandedStruct::from_ast(ast, None, vec![], None);
        assert!(expanded.is_err())
    }
}
