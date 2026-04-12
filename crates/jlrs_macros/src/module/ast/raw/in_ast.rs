//! `in <type_path> <fn_ast>`

use quote::ToTokens;
use syn::{
    Path, Result, Token,
    parse::{Parse, ParseStream},
};

use crate::module::ast::raw::fn_ast::FnAst;

#[derive(Clone)]
pub struct InAst {
    pub in_token: Token![in],
    pub parent: Path,
    pub pub_token: Option<Token![pub]>,
    pub function_ast: FnAst,
}

impl Parse for InAst {
    fn parse(input: ParseStream) -> Result<Self> {
        let in_token = input.parse()?;
        let parent = input.parse()?;
        let pub_token = input.parse()?;
        let function_ast = input.parse()?;

        Ok(InAst {
            in_token,
            parent,
            pub_token,
            function_ast,
        })
    }
}

impl ToTokens for InAst {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.in_token.to_tokens(tokens);
        self.parent.to_tokens(tokens);
        self.pub_token.to_tokens(tokens);
        self.function_ast.to_tokens(tokens);
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::InAst;

    #[test]
    fn parse_private_method() {
        let method_ast: InAst = parse_quote! { in Foo fn foo() };
        let type_path = method_ast.parent.get_ident().unwrap().to_string();
        assert_eq!(type_path, "Foo");

        assert!(method_ast.pub_token.is_none());
    }

    #[test]
    fn parse_public_method() {
        let method_ast: InAst = parse_quote! { in Foo pub fn foo() };
        let type_path = method_ast.parent.get_ident().unwrap().to_string();
        assert_eq!(type_path, "Foo");

        assert!(method_ast.pub_token.is_some());
    }
}
