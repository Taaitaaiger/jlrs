//! `<fn_signature> [as <as_ast>] [use <type_env>]` (e.g. `fn foo() as bar!`)

use quote::ToTokens;
use syn::{
    Result, Signature, Token,
    parse::{Parse, ParseStream},
};

use crate::ast::raw::{as_ast::AsAst, use_ast::UseAst};

#[derive(Clone)]
pub struct FnAst {
    pub signature: Signature,
    pub name_override: Option<AsAst>,
    pub type_var_env: Option<UseAst>,
}

impl Parse for FnAst {
    fn parse(input: ParseStream) -> Result<Self> {
        let signature = input.parse()?;
        let name_override = input
            .lookahead1()
            .peek(Token![as])
            .then(|| input.parse())
            .transpose()?;

        let type_var_env = input
            .lookahead1()
            .peek(Token![use])
            .then(|| input.parse())
            .transpose()?;

        Ok(FnAst {
            signature,
            name_override,
            type_var_env,
        })
    }
}

impl ToTokens for FnAst {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.signature.to_tokens(tokens);
        self.name_override.to_tokens(tokens);
        self.type_var_env.to_tokens(tokens);
    }
}

#[cfg(test)]
mod tests {
    use syn::{Signature, parse_quote};

    use super::FnAst;

    fn signature() -> Signature {
        parse_quote! { fn foo(a: usize) -> usize }
    }

    #[test]
    fn parse_function_ast() {
        let signature = signature();
        let function: FnAst = parse_quote! { #signature };
        assert_eq!(function.signature, signature);
        assert!(function.name_override.is_none());
        assert!(function.type_var_env.is_none());
    }

    #[test]
    fn parse_function_ast_renamed() {
        let signature = signature();
        let function: FnAst = parse_quote! { #signature as bar! };
        assert_eq!(function.signature, signature);
        assert!(function.name_override.is_some());
        assert!(function.type_var_env.is_none());

        let name_override = function.name_override.unwrap();
        assert_eq!(name_override.name_override.len(), 1);
        assert!(name_override.exclamation_mark_token.is_some());

        let first_part = name_override.name_override.get(0).unwrap();
        assert_eq!(first_part.to_string(), "bar");
    }

    #[test]
    fn parse_function_ast_with_env() {
        let signature = signature();
        let function: FnAst = parse_quote! { #signature use Environment };
        assert_eq!(function.signature, signature);
        assert!(function.type_var_env.is_some());
        assert!(function.name_override.is_none());
    }

    #[test]
    fn parse_function_ast_with_macro_env() {
        let signature = signature();
        let function: FnAst = parse_quote! { #signature use tvars!(tvar!('D')) };
        assert_eq!(function.signature, signature);
        assert!(function.type_var_env.is_some());
        assert!(function.name_override.is_none());
    }

    #[test]
    fn parse_function_ast_renamed_with_env() {
        let signature = signature();
        let function: FnAst = parse_quote! { #signature as bar use Environment };
        assert_eq!(function.signature, signature);
        assert!(function.type_var_env.is_some());
        assert!(function.name_override.is_some());

        let name_override = function.name_override.unwrap();
        assert_eq!(name_override.name_override.len(), 1);
        let first_part = name_override.name_override.get(0).unwrap();
        assert_eq!(first_part.to_string(), "bar");
    }

    #[test]
    fn parse_function_ast_renamed_with_macro_env() {
        let signature = signature();
        let function: FnAst = parse_quote! { #signature as bar use tvars!(tvar!('D')) };
        assert_eq!(function.signature, signature);
        assert!(function.type_var_env.is_some());
        assert!(function.name_override.is_some());

        let name_override = function.name_override.unwrap();
        assert_eq!(name_override.name_override.len(), 1);
        let first_part = name_override.name_override.get(0).unwrap();
        assert_eq!(first_part.to_string(), "bar");
    }
}
