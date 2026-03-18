//! `become <init_fn_name>`

use quote::ToTokens;
use syn::{
    Ident, Result, Token,
    parse::{Parse, ParseStream},
};

pub struct BecomeAst {
    pub become_token: Token![become],
    pub init_fn: Ident,
}

impl Parse for BecomeAst {
    fn parse(input: ParseStream) -> Result<Self> {
        let init_fn_token = input.parse()?;
        let init_fn = input.parse()?;

        Ok(BecomeAst {
            become_token: init_fn_token,
            init_fn,
        })
    }
}

impl ToTokens for BecomeAst {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.become_token.to_tokens(tokens);
        self.init_fn.to_tokens(tokens);
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::BecomeAst;

    #[test]
    fn parse_become() {
        let ast: BecomeAst = parse_quote! { become module_init_fn };
        assert_eq!(ast.init_fn.to_string(), "module_init_fn");
    }
}
