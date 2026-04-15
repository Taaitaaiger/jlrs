//! `as <export_name>`

use quote::ToTokens;
use syn::{Ident, Token, parse::Parse, punctuated::Punctuated};

#[derive(Clone)]
pub struct AsAst {
    pub as_token: Token![as],
    pub name_override: Punctuated<Ident, Token![.]>,
    pub exclamation_mark_token: Option<Token![!]>,
}

impl Parse for AsAst {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let as_token = input.parse()?;
        let name_override = Punctuated::parse_separated_nonempty(input)?;
        let exclamation_mark_token = input.parse()?;

        Ok(AsAst {
            as_token,
            name_override,
            exclamation_mark_token,
        })
    }
}

impl ToTokens for AsAst {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.as_token.to_tokens(tokens);
        self.name_override.to_tokens(tokens);
        self.exclamation_mark_token.to_tokens(tokens);
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use crate::ast::raw::as_ast::AsAst;

    #[test]
    fn parse_single() {
        let name: AsAst = parse_quote! { as Foo };
        assert_eq!(name.name_override.len(), 1);
        assert_eq!(name.name_override[0].to_string(), "Foo");
        assert!(name.exclamation_mark_token.is_none());
    }

    #[test]
    fn parse_single_with_exclamation() {
        let name: AsAst = parse_quote! { as Foo! };
        assert_eq!(name.name_override.len(), 1);
        assert_eq!(name.name_override[0].to_string(), "Foo");
        assert!(name.exclamation_mark_token.is_some());
    }

    #[test]
    fn parse_double() {
        let name: AsAst = parse_quote! { as Foo.Bar };
        assert_eq!(name.name_override.len(), 2);
        assert_eq!(name.name_override[0].to_string(), "Foo");
        assert_eq!(name.name_override[1].to_string(), "Bar");
        assert!(name.exclamation_mark_token.is_none());
    }

    #[test]
    fn parse_double_with_exclamation() {
        let name: AsAst = parse_quote! { as Foo.Bar! };
        assert_eq!(name.name_override.len(), 2);
        assert_eq!(name.name_override[0].to_string(), "Foo");
        assert_eq!(name.name_override[1].to_string(), "Bar");
        assert!(name.exclamation_mark_token.is_some());
    }
}
