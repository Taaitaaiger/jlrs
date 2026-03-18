//! use <generic_env_type>

use quote::ToTokens;
use syn::{
    Result, Token, Type,
    parse::{Parse, ParseStream},
};

#[derive(Debug, Clone)]
pub struct UseAst {
    use_token: Option<Token![use]>,
    pub ty: Type,
}

impl Parse for UseAst {
    fn parse(input: ParseStream) -> Result<Self> {
        let use_token = input.parse()?;
        let ty = input.parse()?;

        Ok(Self { use_token, ty })
    }
}

impl ToTokens for UseAst {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.use_token.to_tokens(tokens);
        self.ty.to_tokens(tokens);
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::UseAst;

    #[test]
    fn parse_type() {
        let ast: UseAst = parse_quote! { use Foo };
        match &ast.ty {
            syn::Type::Path(_) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn parse_macro() {
        let ast: UseAst = parse_quote! { use tvars!(tvar!('D')) };
        match &ast.ty {
            syn::Type::Macro(_) => assert!(true),
            _ => assert!(false),
        }
    }
}
